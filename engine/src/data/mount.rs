//! Internal implementation details related to mounting and unmounting files.
//!
//! Step 1 of a game load is building the virtual file system.

use std::{
	io::Cursor,
	path::{Path, PathBuf},
	sync::Arc,
};

use bevy::prelude::{info, warn};
use dashmap::DashMap;
use indexmap::IndexSet;
use parking_lot::Mutex;
use rayon::prelude::*;
use slotmap::SlotMap;
use zip::ZipArchive;

use crate::{
	data::{
		detail::{MountMetaIngest, Outcome, VfsKey},
		Mount, MountInfo, MountKind, MountMeta, MountPointError, VzsManifest, WadExtras,
	},
	utils::{io::*, path::PathExt},
	vzs, wad, VPath, VPathBuf,
};

use super::{
	vfs::{File, FileContent},
	Catalog, LoadTracker, MountError, MountErrorKind, MountFormat,
};

#[derive(Debug)]
pub(super) struct Context {
	tracker: Arc<LoadTracker>,
	/// Returning errors through the mounting call tree is somewhat
	/// inflexible, so pass an array down through the context instead.
	errors: Vec<Mutex<Vec<MountError>>>,
}

impl Context {
	#[must_use]
	pub(super) fn new(tracker: Option<Arc<LoadTracker>>, load_order_len: usize) -> Self {
		// Build a dummy tracker if none was given
		// to simplify the rest of the loading code.
		// Q: Branching might yield a speed increase compared to wasted atomic
		// operations, but is there reason to bother?
		let tracker = tracker.unwrap_or_else(|| Arc::new(LoadTracker::default()));
		tracker.set_mount_target(load_order_len);

		let mut errors = vec![];
		errors.resize_with(load_order_len, Mutex::default);

		Self { tracker, errors }
	}
}

/// Context relevant to one single load order item.
#[derive(Debug)]
pub(self) struct SubContext<'ctx> {
	pub(self) tracker: &'ctx Arc<LoadTracker>,
	pub(self) errors: &'ctx Mutex<Vec<MountError>>,
	pub(self) files: DashMap<VfsKey, File>,
}

impl SubContext<'_> {
	fn add_file(&self, file: File) {
		let path = file.path_raw().clone();
		let key = VfsKey::new(file.path());
		let parent_key = VfsKey::new(file.parent_path().unwrap());
		self.files.insert(key, file);

		let mut parent = match self.files.get_mut(&parent_key) {
			Some(p) => p,
			None => return, // This is a subtree root. It will have to be handled later.
		};

		let FileContent::Directory(children) = &mut parent.content else {
			unreachable!("Parent of {} is not a directory.", path.display());
		};

		children.insert(path);
	}
}

#[derive(Debug)]
struct Success {
	format: MountFormat,
	new_files: DashMap<VfsKey, File>,
	real_path: PathBuf,
	mount_point: VPathBuf,
}

#[derive(Debug)]
pub(super) struct Output {
	pub(super) errors: Vec<Vec<MountError>>,
	pub(super) tracker: Arc<LoadTracker>,
}

impl Catalog {
	pub(super) fn mount(
		&mut self,
		load_order: &Vec<(impl AsRef<Path>, impl AsRef<VPath>)>,
		mut ctx: Context,
	) -> Outcome<Output, Vec<Vec<MountError>>> {
		if ctx.tracker.is_cancelled() {
			return Outcome::Cancelled;
		}

		let mut outcomes = vec![];
		outcomes.resize_with(load_order.len(), || None);
		let outcomes = Mutex::new(outcomes);

		let reqs: Vec<(usize, &Path, &VPath)> = load_order
			.iter()
			.enumerate()
			.map(|(index, (real_path, mount_point))| {
				(index, real_path.as_ref(), mount_point.as_ref())
			})
			.collect();

		reqs.par_iter().for_each(|(i, real_path, mount_point)| {
			if ctx.tracker.is_cancelled() {
				// return Outcome::Cancelled;
				return;
			}

			let subctx = SubContext {
				tracker: &ctx.tracker,
				errors: &ctx.errors[*i],
				files: DashMap::default(),
			};

			let (real_path, mount_point) = match self.validate_mount_request(real_path, mount_point)
			{
				Ok(paths) => paths,
				Err(err) => {
					subctx.errors.lock().push(err);
					return;
				}
			};

			match self.mount_real_unknown(&subctx, &real_path, &mount_point) {
				Outcome::Ok(format) => {
					ctx.tracker.add_mount_progress(1);

					outcomes.lock()[*i] = Some(Success {
						format,
						new_files: subctx.files,
						real_path,
						mount_point,
					});
				}
				Outcome::Err(err) => {
					subctx.errors.lock().push(err);
				}
				Outcome::Cancelled => {}
				Outcome::None => unreachable!(),
			}
		});

		if ctx.tracker.is_cancelled() {
			return Outcome::Cancelled;
		}

		let outcomes = outcomes.into_inner();
		let failed = outcomes.iter().any(|outc| outc.is_none());
		let mut mounts = Vec::with_capacity(load_order.len());

		// Insert new entries into `self.files` if altogether successful.
		// Don't push mounts to `self.mounts` yet; we need to fully populate the
		// file tree so we have the context needed to resolve more metadata.
		if !failed {
			for Success {
				format,
				new_files,
				real_path,
				mount_point,
			} in outcomes.into_iter().map(|outc| outc.unwrap())
			{
				info!(
					"Mounted: \"{}\" -> \"{}\".",
					real_path.display(),
					mount_point.display(),
				);

				if format != MountFormat::Wad {
					super::vfs::sort_dirs_dashmap(&new_files);
				}

				ctx.tracker.add_to_prep_target(new_files.len());
				self.vfs.insert_dashmap(new_files, &mount_point);

				mounts.push(Mount {
					info: MountInfo {
						id: mount_point
							.file_stem()
							.unwrap()
							.to_str()
							.unwrap()
							.to_string(),
						format,
						kind: MountKind::Misc,
						meta: None,
						real_path: real_path.into_boxed_path(),
						virtual_path: mount_point.into_boxed_path(),
						vzscript: None,
					},
					objs: SlotMap::default(),
					extras: WadExtras::default(),
				});
			}
		}

		let errors: Vec<Vec<MountError>> = std::mem::take(&mut ctx.errors)
			.into_iter()
			.map(|mutex| mutex.into_inner())
			.collect();

		debug_assert_eq!(errors.len(), load_order.len());

		if failed {
			return Outcome::Err(errors);
		}

		// Register mounts; learn as much about them as possible in the process.

		for mut mount in mounts {
			mount.info.kind =
				self.resolve_mount_kind(mount.info.format(), mount.info.virtual_path());

			self.resolve_mount_metadata(&mut mount.info);
			self.mounts.push(mount);
		}

		Outcome::Ok(Output {
			errors,
			tracker: ctx.tracker,
		})
	}

	/// Mount functions dealing in real (i.e. non-archived) files funnel their
	/// operations through here. It returns the deduced format of the file in
	/// question only for the benefit of the top-level function whose job it is
	/// to build the [`MountInfo`].
	fn mount_real_unknown(
		&self,
		ctx: &SubContext,
		real_path: &Path,
		virt_path: &VPath,
	) -> Outcome<MountFormat, MountError> {
		let (format, bytes) = if real_path.is_dir() {
			(MountFormat::Directory, vec![])
		} else {
			let b = match std::fs::read(real_path) {
				Ok(b) => b,
				Err(err) => {
					return Outcome::Err(MountError {
						path: real_path.to_path_buf(),
						kind: MountErrorKind::FileRead(err),
					});
				}
			};

			let f = Self::resolve_file_format(&b, virt_path);
			(f, b)
		};

		let outcome = match format {
			MountFormat::PlainFile => {
				if let Some(leaf) = self.new_leaf_file(virt_path.to_path_buf(), bytes) {
					ctx.add_file(leaf);
				}

				Outcome::Ok(())
			}
			MountFormat::Directory => self.mount_dir(ctx, real_path, virt_path),
			MountFormat::Zip => self.mount_zip(ctx, virt_path, bytes),
			MountFormat::Wad => self.mount_wad(ctx, virt_path, bytes),
		};

		match outcome {
			Outcome::Ok(()) => Outcome::Ok(format),
			Outcome::Err(err) => Outcome::Err(err),
			Outcome::Cancelled => Outcome::Cancelled,
			Outcome::None => unreachable!(),
		}
	}

	fn mount_dir(
		&self,
		ctx: &SubContext,
		real_path: &Path,
		virt_path: &VPath,
	) -> Outcome<(), MountError> {
		let dir_iter = match std::fs::read_dir(real_path) {
			Ok(r) => r.filter_map(|res| match res {
				Ok(r) => Some(r),
				Err(_) => None,
			}),
			Err(err) => {
				return Outcome::Err(MountError {
					path: real_path.to_path_buf(),
					kind: MountErrorKind::DirectoryRead(err),
				});
			}
		};

		ctx.add_file(File::new_dir(virt_path.to_path_buf()));

		for entry in dir_iter {
			if ctx.tracker.is_cancelled() {
				return Outcome::Cancelled;
			}

			let ftype = match entry.file_type() {
				Ok(ft) => ft,
				Err(err) => {
					ctx.errors.lock().push(MountError {
						path: entry.path(),
						kind: MountErrorKind::FileType(err),
					});

					continue;
				}
			};

			if ftype.is_symlink() {
				continue;
			}

			let fname_os = entry.file_name();
			let fname_cow = fname_os.to_string_lossy();
			let filename = fname_cow.as_ref();

			let de_real_path = entry.path();
			let de_virt_path: PathBuf = [virt_path, Path::new(filename)].iter().collect();

			if let Outcome::Err(err) = self.mount_real_unknown(ctx, &de_real_path, &de_virt_path) {
				return Outcome::Err(err);
			}
		}

		Outcome::Ok(())
	}

	fn mount_wad(
		&self,
		ctx: &SubContext,
		virt_path: &VPath,
		bytes: Vec<u8>,
	) -> Outcome<(), MountError> {
		#[rustfmt::skip]
		const MAP_COMPONENTS: &[&str] = &[
			"blockmap",
			"linedefs",
			"nodes",
			"reject",
			"sectors",
			"segs",
			"sidedefs",
			"ssectors",
			"things",
			"vertexes",
			// UDMF
			"behavior",
			"dialogue",
			"scripts",
			"textmap",
			"znodes",
		];

		#[must_use]
		fn is_map_component(name: &str) -> bool {
			MAP_COMPONENTS.iter().any(|s| s.eq_ignore_ascii_case(name))
		}

		#[must_use]
		fn maybe_map_marker(file: &File) -> bool {
			file.is_empty() && !is_map_component(file.file_prefix())
		}

		if ctx.tracker.is_cancelled() {
			return Outcome::Cancelled;
		}

		let wad = wad::parse_wad(bytes).map_err(|err| MountError {
			path: virt_path.to_path_buf(),
			kind: MountErrorKind::Wad(err),
		});

		let wad = match wad {
			Ok(w) => w,
			Err(err) => return Outcome::Err(err),
		};

		let mut files = Vec::with_capacity(wad.len() + 1);
		files.push(File::new_dir(virt_path.to_path_buf()));

		let mut dissolution = wad.dissolve();
		let mut index = 0_usize;
		let mut mapfold: Option<usize> = None;

		for (bytes, name) in dissolution.drain(..) {
			index += 1;

			if ctx.tracker.is_cancelled() {
				return Outcome::Cancelled;
			}

			// If the previous entry was some kind of marker, and we're just now
			// looking at some kind of map component (UDMF spec makes it easier
			// to discern), then in all likelihood the previous WAD entry was a
			// map marker, and needs to be treated like a directory. Future WAD
			// entries until the next map marker or non-map component will be made
			// children of that folder.
			if maybe_map_marker(&files[index - 1]) && is_map_component(&name) {
				let prev = files.get_mut(index - 1).unwrap();
				prev.content = FileContent::Directory(IndexSet::with_capacity(10));
				mapfold = Some(index - 1);
			} else if !is_map_component(name.as_str()) {
				mapfold = None;
			}

			let mut child_path = if let Some(entry_idx) = mapfold {
				// virt_path currently: `/mount_point`
				let mut cpath = virt_path.join(files[entry_idx].file_name());
				// cpath currently: `/mount_point/MAP01`
				cpath.push(&name);
				// cpath currently: `/mount_point/MAP01/THINGS`
				cpath
			} else {
				virt_path.join(&name)
			};

			// What if a WAD contains two entries with the same name?
			// For example, DOOM2.WAD has two identical `SW18_7` entries, and
			// Angelic Aviary 1.0 has several `DECORATE` lumps.
			// Roll them together into virtual directories. The end result is:
			// /Angelic Aviary 1.0
			//		/DECORATE
			// 			/000
			// 			/001
			// 			/002
			// ...and so on.

			if let Some(pos) = files
				.iter()
				.position(|e| e.path.as_ref() == <VPathBuf as AsRef<VPath>>::as_ref(&child_path))
			{
				if !files[pos].is_dir() {
					files.insert(pos, File::new_dir(files[pos].path.to_path_buf()));

					let prev_path =
						std::mem::replace(&mut files[pos + 1].path, PathBuf::default().into());

					let mut prev_path = prev_path.to_path_buf();
					prev_path.push("000");

					files[pos + 1].path = prev_path.into();
					child_path.push("001");
				} else {
					let num_children = files
						.iter()
						.filter(|vf| vf.parent_path().unwrap() == child_path)
						.count();

					child_path.push(format!("{num_children:03}"));
				}
			}

			if let Some(leaf) = self.new_leaf_file(child_path, bytes) {
				files.push(leaf);
			}
		}

		for file in files {
			ctx.add_file(file);
		}

		Outcome::Ok(())
	}

	fn mount_zip(
		&self,
		ctx: &SubContext,
		virt_path: &VPath,
		bytes: Vec<u8>,
	) -> Outcome<(), MountError> {
		let cursor = Cursor::new(&bytes);

		let zip = ZipArchive::new(cursor).map_err(|err| MountError {
			path: virt_path.to_path_buf(),
			kind: MountErrorKind::ZipArchiveRead(err),
		});

		let mut zip = match zip {
			Ok(z) => z,
			Err(err) => return Outcome::Err(err),
		};

		ctx.add_file(File::new_dir(virt_path.to_path_buf()));

		// First pass creates a directory structure.

		for i in 0..zip.len() {
			if ctx.tracker.is_cancelled() {
				return Outcome::Cancelled;
			}

			let zfile = match zip.by_index(i) {
				Ok(z) => {
					if !z.is_dir() {
						continue;
					} else {
						z
					}
				}
				Err(err) => {
					ctx.errors.lock().push(MountError {
						path: virt_path.to_path_buf(),
						kind: MountErrorKind::ZipFileGet(i, err),
					});

					continue;
				}
			};

			let zfpath = match zfile.enclosed_name() {
				Some(p) => p,
				None => {
					ctx.errors.lock().push(MountError {
						path: virt_path.to_path_buf(),
						kind: MountErrorKind::ZipFileName(zfile.name().to_string()),
					});

					continue;
				}
			};

			let zdir_virt_path: PathBuf = [virt_path, zfpath].iter().collect();
			ctx.add_file(File::new_dir(zdir_virt_path));
		}

		// Second pass covers leaf nodes.

		for i in 0..zip.len() {
			if ctx.tracker.is_cancelled() {
				return Outcome::Cancelled;
			}

			let mut zfile = match zip.by_index(i) {
				Ok(z) => {
					if z.is_dir() {
						continue;
					} else {
						z
					}
				}
				Err(err) => {
					ctx.errors.lock().push(MountError {
						path: virt_path.to_path_buf(),
						kind: MountErrorKind::ZipFileGet(i, err),
					});

					continue;
				}
			};

			let zfsize = zfile.size();
			let mut bytes = Vec::<u8>::with_capacity(zfsize.try_into().unwrap());

			match std::io::copy(&mut zfile, &mut bytes) {
				Ok(count) => {
					if count != zfsize {
						ctx.errors.lock().push(MountError {
							path: virt_path.to_path_buf(),
							kind: MountErrorKind::ZipFileRead {
								name: zfile
									.enclosed_name()
									.unwrap_or_else(|| Path::new(zfile.name()))
									.to_path_buf(),
								err: None,
							},
						});

						continue;
					}
				}
				Err(err) => {
					ctx.errors.lock().push(MountError {
						path: virt_path.to_path_buf(),
						kind: MountErrorKind::ZipFileRead {
							name: zfile
								.enclosed_name()
								.unwrap_or_else(|| Path::new(zfile.name()))
								.to_path_buf(),
							err: Some(err),
						},
					});

					continue;
				}
			};

			let zfpath = match zfile.enclosed_name() {
				Some(p) => p,
				None => {
					ctx.errors.lock().push(MountError {
						path: virt_path.to_path_buf(),
						kind: MountErrorKind::ZipFileName(zfile.name().to_string()),
					});

					continue;
				}
			};

			let zf_virt_path: VPathBuf = [virt_path, zfpath].iter().collect();

			if let Some(leaf) = self.new_leaf_file(zf_virt_path, bytes) {
				ctx.add_file(leaf);
			}
		}

		Outcome::Ok(())
	}

	// Utility /////////////////////////////////////////////////////////////////

	/// Returns a new [empty], [text], or [binary] file, depending on `bytes`,
	/// but only if the VFS size limit configuration permits it. If the file is
	/// oversize, log a warning and return `None`.
	///
	/// [empty]: FileKind::Empty
	/// [text]: FileKind::Text
	/// [binary]: FileKind::Binary
	#[must_use]
	fn new_leaf_file(&self, virt_path: VPathBuf, bytes: Vec<u8>) -> Option<File> {
		if bytes.is_empty() {
			return Some(File::new_empty(virt_path));
		}

		match String::from_utf8(bytes) {
			Ok(string) => {
				if string.len() > self.config.text_size_limit {
					warn!(
						"File is too big to be mounted: {p}\r\n\t\
						Size: {sz}\r\n\t\
						Maximum allowed for text files: {max}",
						p = virt_path.display(),
						sz = string.len(),
						max = self.config.text_size_limit
					);

					return None;
				}

				Some(File::new_text(virt_path, string.into_boxed_str()))
			}
			Err(err) => {
				let slice = err.into_bytes().into_boxed_slice();

				if slice.len() > self.config.bin_size_limit {
					warn!(
						"File is too big to be mounted: {p}\r\n\t\
						Size: {sz}\r\n\t\
						Maximum allowed for non-text files: {max}",
						p = virt_path.display(),
						sz = slice.len(),
						max = self.config.bin_size_limit
					);

					return None;
				}

				Some(File::new_binary(virt_path, slice))
			}
		}
	}

	fn validate_mount_request(
		&self,
		real_path: &Path,
		mount_path: &VPath,
	) -> Result<(PathBuf, VPathBuf), MountError> {
		let real_path = match real_path.canonicalize() {
			Ok(canon) => canon,
			Err(err) => {
				return Err(MountError {
					path: real_path.to_path_buf(),
					kind: MountErrorKind::Canonicalization(err),
				});
			}
		};

		if !real_path.exists() {
			return Err(MountError {
				path: real_path,
				kind: MountErrorKind::FileNotFound,
			});
		}

		if real_path.is_symlink() {
			return Err(MountError {
				path: real_path,
				kind: MountErrorKind::MountSymlink,
			});
		}

		if real_path.is_hidden() {
			return Err(MountError {
				path: real_path,
				kind: MountErrorKind::MountHidden,
			});
		}

		// Ensure mount point has no invalid characters, isn't reserved, etc.

		self.mount_path_valid(mount_path)?;

		// Ensure mount point path has a parent path.

		let mount_point_parent = match mount_path.parent() {
			Some(p) => p,
			None => {
				return Err(MountError {
					path: mount_path.to_path_buf(),
					kind: MountErrorKind::ParentlessMountPoint,
				});
			}
		};

		// Ensure nothing already exists at end of mount point.

		if self.vfs.contains(mount_path) {
			return Err(MountError {
				path: mount_path.to_path_buf(),
				kind: MountErrorKind::Remount,
			});
		}

		// Ensure mount point parent exists.

		if !self.vfs.contains(mount_point_parent) {
			return Err(MountError {
				path: mount_path.to_path_buf(),
				kind: MountErrorKind::MountParentNotFound(mount_point_parent.to_path_buf()),
			});
		}

		// All checks passed.

		let mut mount_point = VPathBuf::new();

		if !mount_path.starts_with("/") {
			mount_point.push("/");
		}

		mount_point.push(mount_path);

		Ok((real_path, mount_point))
	}

	/// A mount path must:
	/// - Be valid UTF-8.
	/// - Contain no relative components (`.` or `..`).
	/// There is also the following context-sensitive check:
	/// - If nothing has been mounted yet, assert that the file name at the end
	/// of the mount path is the base data's file name.
	/// - If the base data has already been mounted, check that none of the
	/// components in the mount path are a reserved name (ASCII case ignored).
	fn mount_path_valid(&self, path: &VPath) -> Result<(), MountError> {
		path.to_str().ok_or_else(|| MountError {
			path: path.to_path_buf(),
			kind: MountErrorKind::InvalidMountPoint(MountPointError::InvalidUtf8),
		})?;

		#[cfg(not(test))]
		if self.mounts.is_empty() {
			assert!(
				path.ends_with(crate::BASEDATA_ID),
				"Engine base data is being mounted late."
			);

			return Ok(());
		}

		const RESERVED: &[&str] = &[
			"vile", "viletec", "vt", "vtec", "vtech", "viletech", "vzs", "vzscript", "zs",
			"zscript",
		];

		for comp in path.components() {
			match comp {
				std::path::Component::Prefix(_) => {
					unreachable!("A Windows path prefix wasn't filtered out of a mount point.")
				}
				std::path::Component::RootDir => {} // OK
				std::path::Component::CurDir | std::path::Component::ParentDir => {
					return Err(MountError {
						path: path.to_path_buf(),
						kind: MountErrorKind::InvalidMountPoint(MountPointError::Relative),
					});
				}
				std::path::Component::Normal(os_str) => {
					let s = os_str.to_str().unwrap();

					for reserved in RESERVED {
						if s.contains(reserved) {
							return Err(MountError {
								path: path.to_path_buf(),
								kind: MountErrorKind::InvalidMountPoint(MountPointError::Reserved),
							});
						}
					}
				}
			}
		}

		Ok(())
	}

	#[must_use]
	fn resolve_file_format(bytes: &[u8], virt_path: &VPath) -> MountFormat {
		match is_valid_wad(bytes, bytes.len().try_into().unwrap()) {
			Ok(is_wad) => {
				if is_wad {
					return MountFormat::Wad;
				}
			}
			Err(err) => {
				warn!(
					"Failed to determine if file is a WAD: {}\r\n\t\
					Error: {err}\r\n\t\
					It will likely be treated as an unknown file.",
					virt_path.display()
				);
			}
		}

		if is_zip(bytes) {
			return MountFormat::Zip;
		}

		MountFormat::PlainFile
	}

	/// Assumes that `self.files` has been fully populated.
	#[must_use]
	fn resolve_mount_kind(
		&self,
		format: MountFormat,
		virtual_path: impl AsRef<VPath>,
	) -> MountKind {
		if format == MountFormat::Wad {
			return MountKind::Wad;
		}

		let fref = self
			.vfs
			.get(virtual_path)
			.expect("`resolve_mount_kind` received an invalid virtual path.");

		if fref.is_leaf() {
			return MountKind::Misc;
		}

		// Heuristics have a precedence hierarchy, so use multiple passes.

		if fref
			.children()
			.unwrap()
			.any(|child| child.file_name().eq_ignore_ascii_case("meta.toml") && child.is_text())
		{
			return MountKind::VileTech;
		}

		const ZDOOM_FILE_PFXES: &[&str] = &[
			"cvarinfo", "decorate", "gldefs", "menudef", "modeldef", "sndinfo", "zmapinfo",
			"zscript",
		];

		if fref.children().unwrap().any(|child| {
			let pfx = child.file_prefix();

			ZDOOM_FILE_PFXES
				.iter()
				.any(|&constant| pfx.eq_ignore_ascii_case(constant))
		}) {
			return MountKind::ZDoom;
		}

		if fref.children().unwrap().any(|child| {
			let fstem = child.file_prefix();
			fstem.eq_ignore_ascii_case("edfroot") || fstem.eq_ignore_ascii_case("emapinfo")
		}) {
			return MountKind::Eternity;
		}

		unreachable!("All mount kind resolution heuristics failed.")
	}

	/// Parses a meta.toml if one exists. Otherwise, make a best-possible effort
	/// to deduce some metadata. Assumes that `self.files` has been fully populated.
	fn resolve_mount_metadata(&self, info: &mut MountInfo) {
		debug_assert!(!info.id.is_empty());

		if info.kind != MountKind::VileTech {
			// Q: Should we bother trying to infer the mount's version?
			return;
		}

		let meta_path = info.virtual_path().join("meta.toml");
		let meta_file = self.vfs.get(&meta_path).unwrap();

		let ingest: MountMetaIngest = match toml::from_str(meta_file.read_str()) {
			Ok(toml) => toml,
			Err(err) => {
				warn!(
					"Invalid meta.toml file: {p}\r\n\t\
					Details: {err}\r\n\t\
					This mount's metadata may be incomplete.",
					p = meta_path.display()
				);

				return;
			}
		};

		info.id = ingest.id;

		if let Some(mnf) = ingest.vzscript {
			let version = match mnf.version.parse::<vzs::Version>() {
				Ok(v) => v,
				Err(err) => {
					warn!(
						"Invalid `vzscript` table in meta.toml file: {p}\r\n\t\
						Details: {err}\r\n\t\
						This mount's metadata may be incomplete.",
						p = meta_path.display()
					);

					return;
				}
			};

			info.vzscript = Some(VzsManifest {
				root_dir: mnf.folder,
				namespace: mnf.namespace,
				version,
			});
		}

		info.meta = Some(Box::new(MountMeta {
			version: ingest.version,
			name: ingest.name,
			description: ingest.description,
			authors: ingest.authors,
			copyright: ingest.copyright,
			links: ingest.links,
		}));
	}
}
