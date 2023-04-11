//! Internal implementation details related to mounting and unmounting files.
//!
//! Step 1 of a game load is building the virtual file system.

use std::{
	io::Cursor,
	path::{Path, PathBuf},
	sync::Arc,
};

use bevy::prelude::{info, warn};
use parking_lot::Mutex;
use rayon::prelude::*;
use slotmap::SlotMap;
use zip::ZipArchive;

use crate::{
	data::{detail::MountMetaIngest, Mount, MountErrorKind, MountMeta, MountPointError},
	utils::{io::*, path::PathExt},
	wad, VPath, VPathBuf,
};

use super::{
	detail::{Outcome, VfsKey},
	Catalog, File, FileKind, LoadTracker, MountError, MountFormat, MountInfo, MountKind,
};

#[derive(Debug)]
pub(super) struct Context {
	tracker: Arc<LoadTracker>,
	/// Returning errors through the mounting call tree is somewhat
	/// inflexible, so pass an array down through the context instead.
	errors: Mutex<Vec<Vec<MountError>>>,
}

impl Context {
	#[must_use]
	pub(super) fn new(tracker: Option<Arc<LoadTracker>>) -> Self {
		Self {
			// Build a dummy tracker if none was given to avoid branching later
			// and simplify the rest of the loading code.
			tracker: tracker.unwrap_or_else(|| Arc::new(LoadTracker::default())),
			errors: Mutex::new(vec![]),
		}
	}
}

#[derive(Debug)]
pub(super) struct Output {
	pub(super) errors: Vec<Vec<MountError>>,
	pub(super) tracker: Arc<LoadTracker>,
}

#[derive(Debug)]
struct Success {
	format: MountFormat,
	new_files: Vec<File>,
	real_path: PathBuf,
	mount_point: PathBuf,
}

impl Catalog {
	pub(super) fn mount(
		&mut self,
		mount_reqs: &[(impl AsRef<Path>, impl AsRef<VPath>)],
		mut ctx: Context,
	) -> Outcome<Output, Vec<Vec<MountError>>> {
		ctx.tracker.set_mount_target(mount_reqs.len());
		ctx.errors.lock().resize_with(mount_reqs.len(), Vec::new);

		let mut outcomes = Vec::with_capacity(mount_reqs.len());
		outcomes.resize_with(mount_reqs.len(), || None);
		let outcomes = Mutex::new(outcomes);

		let reqs: Vec<(usize, &Path, &VPath)> = mount_reqs
			.iter()
			.enumerate()
			.map(|(index, (real_path, mount_path))| {
				(index, real_path.as_ref(), mount_path.as_ref())
			})
			.collect();

		if ctx.tracker.is_cancelled() {
			return Outcome::Cancelled;
		}

		reqs.par_iter().for_each(|(index, real_path, mount_path)| {
			let index = *index;

			let (real_path, mount_point) =
				match self.validate_mount_request(real_path.as_ref(), mount_path.as_ref()) {
					Ok(paths) => paths,
					Err(err) => {
						ctx.errors.lock()[index].push(err);
						return;
					}
				};

			// Suppose the following two mount points are given:
			// - `/mygame`
			// - `/mygame/myothergame`
			// In two separate mount batches this is valid, but in one batch, it
			// can lead to a parent existence check passing when it shouldn't due
			// to a data race, or the parallel iterator only being given 1 thread.
			if reqs
				.iter()
				.any(|(_, _, mpath)| mount_point.is_child_of(mpath))
			{
				ctx.errors.lock()[index].push(MountError {
					path: real_path,
					kind: MountErrorKind::Remount,
				});

				return;
			}

			match self.mount_real_unknown(&real_path, &mount_point) {
				Ok((new_files, format)) => {
					ctx.tracker.add_mount_progress(1);

					outcomes.lock()[index] = Some(Success {
						format,
						new_files,
						real_path,
						mount_point,
					});
				}
				Err(err) => {
					ctx.errors.lock()[index].push(err);
				}
			}
		});

		if ctx.tracker.is_cancelled() {
			return Outcome::Cancelled;
		}

		let outcomes = outcomes.into_inner();
		let failed = outcomes.iter().any(|outc| outc.is_none());
		let mut mounts = Vec::with_capacity(mount_reqs.len());

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

				ctx.tracker.set_pproc_target(new_files.len());

				for new_file in new_files {
					let displaced = self.files.insert(VfsKey::new(&new_file.path), new_file);

					debug_assert!(
						displaced.is_none(),
						"A VFS mount operation displaced entry: {}",
						displaced.unwrap().path.display()
					);
				}

				mounts.push(Mount {
					assets: SlotMap::default(),
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
						script_root: None,
					},
				});
			}
		}

		let mut errors = std::mem::replace(&mut ctx.errors, Mutex::new(vec![])).into_inner();

		debug_assert_eq!(errors.len(), mount_reqs.len());

		for subvec in &mut errors {
			subvec.sort_by(|err1, err2| err1.path.cmp(&err2.path));
		}

		// Time for some housekeeping. If we've failed, clean up after ourselves.
		if failed {
			return Outcome::Err(errors);
		}

		// Tell directories about their children.
		self.populate_dirs();

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
		real_path: &Path,
		virt_path: &VPath,
	) -> Result<(Vec<File>, MountFormat), MountError> {
		let (format, bytes) = if real_path.is_dir() {
			(MountFormat::Directory, vec![])
		} else {
			let b = match std::fs::read(real_path) {
				Ok(b) => b,
				Err(err) => {
					return Err(MountError {
						path: real_path.to_path_buf(),
						kind: MountErrorKind::FileRead(err),
					});
				}
			};

			let f = Self::resolve_file_format(&b, virt_path);
			(f, b)
		};

		let res = match format {
			MountFormat::PlainFile => {
				if let Some(leaf) = self.new_leaf_file(virt_path.to_path_buf(), bytes) {
					Ok(vec![leaf])
				} else {
					Ok(vec![])
				}
			}
			MountFormat::Directory => self.mount_dir(real_path, virt_path),
			MountFormat::Zip => self.mount_zip(virt_path, bytes),
			MountFormat::Wad => self.mount_wad(virt_path, bytes),
		};

		match res {
			Ok(new_files) => Ok((new_files, format)),
			Err(err) => Err(err),
		}
	}

	fn mount_dir(&self, real_path: &Path, virt_path: &VPath) -> Result<Vec<File>, MountError> {
		let mut ret = Vec::default();

		let dir_iter = match std::fs::read_dir(real_path) {
			Ok(r) => r.filter_map(|res| match res {
				Ok(r) => Some(r),
				Err(_) => None,
			}),
			Err(err) => {
				return Err(MountError {
					path: real_path.to_path_buf(),
					kind: MountErrorKind::DirectoryRead(err),
				});
			}
		};

		ret.push(Self::new_dir(virt_path.to_path_buf()));

		for entry in dir_iter {
			let ftype = match entry.file_type() {
				Ok(ft) => ft,
				Err(err) => {
					warn!(
						"Skipping mounting directory entry of unknown type: {}\r\n\
						File type acquiry error: {err}",
						entry.path().display(),
					);
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

			match self.mount_real_unknown(&de_real_path, &de_virt_path) {
				Ok((mut new_files, _)) => {
					ret.append(&mut new_files);
				}
				Err(err) => return Err(err),
			}
		}

		ret[1..].par_sort_by(File::cmp_name);

		Ok(ret)
	}

	fn mount_wad(&self, virt_path: &VPath, bytes: Vec<u8>) -> Result<Vec<File>, MountError> {
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

		let wad = wad::parse_wad(bytes).map_err(|err| MountError {
			path: virt_path.to_path_buf(),
			kind: MountErrorKind::Wad(err),
		})?;

		let mut ret = Vec::with_capacity(wad.len() + 1);
		let mut dissolution = wad.dissolve();
		ret.push(Self::new_dir(virt_path.to_path_buf()));
		let mut index = 0_usize;
		let mut mapfold: Option<usize> = None;

		for (bytes, name) in dissolution.drain(..) {
			index += 1;

			// If the previous entry was some kind of marker, and we're just now
			// looking at some kind of map component (UDMF spec makes it easier
			// to discern), then in all likelihood the previous WAD entry was a
			// map marker, and needs to be treated like a directory. Future WAD
			// entries until the next map marker or non-map component will be made
			// children of that folder.
			if ret[index - 1].is_empty()
				&& (is_map_component(&name) || name.eq_ignore_ascii_case("TEXTMAP"))
			{
				let prev_index = ret.len() - 1;
				let prev = ret.get_mut(prev_index).unwrap();
				prev.kind = FileKind::Directory(Vec::with_capacity(10));
				mapfold = Some(index - 1);
			} else if !is_map_component(name.as_str()) {
				mapfold = None;
			}

			let mut child_path = if let Some(entry_idx) = mapfold {
				// virt_path currently: `/mount_point`
				let mut cpath = virt_path.join(ret[entry_idx].file_name());
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
			// - /DECORATE
			// -	/000
			// - 	/001
			// - 	/002
			// ...and so on.

			if let Some(pos) = ret
				.iter()
				.position(|e| e.path.as_ref() == <VPathBuf as AsRef<VPath>>::as_ref(&child_path))
			{
				if !ret[pos].is_dir() {
					ret.insert(pos, Self::new_dir(ret[pos].path.to_path_buf()));

					let prev_path = std::mem::replace(
						&mut ret[pos + 1].path,
						VPathBuf::default().into_boxed_path(),
					);

					let mut prev_path = VPathBuf::from(prev_path);
					prev_path.push("000");

					ret[pos + 1].path = prev_path.into_boxed_path();
					child_path.push("001");
				} else {
					let num_children = ret
						.iter()
						.filter(|vf| vf.parent_path().unwrap() == child_path)
						.count();

					child_path.push(format!("{num_children:03}"));
				}
			}

			if let Some(leaf) = self.new_leaf_file(child_path, bytes) {
				ret.push(leaf);
			}
		}

		Ok(ret)
	}

	fn mount_zip(&self, virt_path: &VPath, bytes: Vec<u8>) -> Result<Vec<File>, MountError> {
		let cursor = Cursor::new(&bytes);

		let mut zip = ZipArchive::new(cursor).map_err(|err| MountError {
			path: virt_path.to_path_buf(),
			kind: MountErrorKind::Zip(err),
		})?;

		let mut ret = Vec::with_capacity(zip.len() + 1);
		ret.push(Self::new_dir(virt_path.to_path_buf()));

		// First pass creates a directory structure.

		for i in 0..zip.len() {
			let zfile = match zip.by_index(i) {
				Ok(z) => {
					if !z.is_dir() {
						continue;
					} else {
						z
					}
				}
				Err(err) => {
					warn!(
						"Skipping malformed entry in zip archive: {}\r\n\t\
						Error: {err}",
						virt_path.display(),
					);
					continue;
				}
			};

			let zfpath = match zfile.enclosed_name() {
				Some(p) => p,
				None => {
					warn!(
						"A zip file entry contains an unsafe path at index: {i}\r\n\t\
						Zip file virtual path: {}",
						virt_path.display()
					);
					continue;
				}
			};

			let zdir_virt_path: PathBuf = [virt_path, zfpath].iter().collect();
			ret.push(Self::new_dir(zdir_virt_path));
		}

		// Second pass covers leaf nodes.

		for i in 0..zip.len() {
			let mut zfile = match zip.by_index(i) {
				Ok(z) => {
					if z.is_dir() {
						continue;
					} else {
						z
					}
				}
				Err(err) => {
					warn!(
						"Skipping malformed entry in zip archive: {}\r\n\
						Error: {err}",
						virt_path.display(),
					);
					continue;
				}
			};

			let zfsize = zfile.size();
			let mut bytes = Vec::<u8>::with_capacity(zfsize.try_into().unwrap());

			match std::io::copy(&mut zfile, &mut bytes) {
				Ok(count) => {
					if count != zfsize {
						warn!(
							"Failed to read all bytes of zip file entry: {}\r\n\
							Zip file virtual path: {}",
							zfile.enclosed_name().unwrap().display(),
							virt_path.display()
						);
						continue;
					}
				}
				Err(err) => {
					warn!(
						"Failed to read zip file entry: {}\r\nZip file virtual path: {}\r\n\
						Error: {err}",
						zfile.enclosed_name().unwrap().display(),
						virt_path.display(),
					);
					continue;
				}
			};

			let zfpath = match zfile.enclosed_name() {
				Some(p) => p,
				None => {
					warn!(
						"A zip file entry contains an unsafe path at index: {i}\r\n\
						Zip file virtual path: {}",
						virt_path.display()
					);
					continue;
				}
			};

			let zf_virt_path: VPathBuf = [virt_path, zfpath].iter().collect();

			if let Some(leaf) = self.new_leaf_file(zf_virt_path, bytes) {
				ret.push(leaf);
			}
		}

		ret[1..].par_sort_by(File::cmp_name);

		Ok(ret)
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
		let kind: FileKind = if bytes.is_empty() {
			FileKind::Empty
		} else {
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

					FileKind::Text(string.into_boxed_str())
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

					FileKind::Binary(slice)
				}
			}
		};

		Some(File {
			path: virt_path.into_boxed_path(),
			kind,
		})
	}

	#[must_use]
	fn new_dir(virt_path: VPathBuf) -> File {
		File {
			path: virt_path.into_boxed_path(),
			kind: FileKind::Directory(Vec::default()),
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

		if self.file_exists(mount_path) {
			return Err(MountError {
				path: mount_path.to_path_buf(),
				kind: MountErrorKind::Remount,
			});
		}

		// Ensure mount point parent exists.

		if !self.file_exists(mount_point_parent) {
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
	fn resolve_mount_kind(
		&self,
		format: MountFormat,
		virtual_path: impl AsRef<VPath>,
	) -> MountKind {
		if format == MountFormat::Wad {
			return MountKind::Wad;
		}

		let fref = self
			.get_file(virtual_path)
			.expect("`resolve_mount_kind` received an invalid virtual path.");

		if fref.is_leaf() {
			return MountKind::Misc;
		}

		// Heuristics have a precedence hierarchy, so use multiple passes.

		if fref
			.children()
			.any(|child| child.file_name().eq_ignore_ascii_case("meta.toml") && child.is_text())
		{
			return MountKind::VileTech;
		}

		const ZDOOM_LUMP_FILE_STEMS: &[&str] = &[
			"cvarinfo", "decorate", "gldefs", "menudef", "modeldef", "sndinfo", "zmapinfo",
			"zscript",
		];

		if fref.children().any(|child| {
			let fstem = child.file_stem();
			ZDOOM_LUMP_FILE_STEMS
				.iter()
				.any(|&constant| fstem.eq_ignore_ascii_case(constant))
		}) {
			return MountKind::ZDoom;
		}

		if fref.children().any(|child| {
			let fstem = child.file_stem();
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
		let meta_file = self.get_file(&meta_path).unwrap();

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
		info.script_root = ingest.script_root.map(|vpb| vpb.into_boxed_path());

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
