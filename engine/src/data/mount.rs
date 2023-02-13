//! Internal implementation details related to mounting and unmounting files.
//!
//! Step 1 of a game load is building the virtual file system.

use std::{
	io::Cursor,
	path::{Path, PathBuf},
	sync::{atomic, Arc},
};

use log::{info, warn};
use parking_lot::Mutex;
use rayon::prelude::*;
use zip::ZipArchive;

use crate::{
	data::detail::MountMetaIngest,
	utils::{io::*, path::PathExt, string::version_from_string},
	wad, VPath, VPathBuf,
};

use super::{
	detail::VfsKey, Catalog, LoadTracker, Mount, MountError, MountFormat, MountInfo, MountKind,
	VirtFileKind, VirtualFile,
};

#[derive(Debug)]
pub(super) struct Context {
	pub(super) tracker: Arc<LoadTracker>,
}

#[derive(Debug)]
pub(super) struct Output {
	pub(super) results: Vec<Result<(), MountError>>,
	pub(super) orig_files_len: usize,
	pub(super) orig_mounts_len: usize,
	pub(super) tracker: Arc<LoadTracker>,
}

impl Output {
	#[must_use]
	pub(super) fn any_errs(&self) -> bool {
		self.results.iter().any(|res| res.is_err())
	}
}

#[derive(Debug)]
enum Outcome {
	Uninit,
	Err(MountError),
	Ok {
		format: MountFormat,
		new_files: Vec<VirtualFile>,
		real_path: PathBuf,
		mount_point: PathBuf,
	},
}

impl Catalog {
	#[must_use]
	pub(super) fn mount(
		&mut self,
		mount_reqs: &[(impl AsRef<Path>, impl AsRef<VPath>)],
		ctx: Context,
	) -> Output {
		// To enable atomicity, remember where `self.files` and `self.mounts` were.
		// Truncate back to them upon a failure
		let orig_files_len = self.files.len();
		let orig_mounts_len = self.mounts.len();

		let mut results = Vec::with_capacity(mount_reqs.len());
		results.resize_with(mount_reqs.len(), || Outcome::Uninit);

		let results = Mutex::new(results);

		let reqs: Vec<(usize, &Path, &VPath)> = mount_reqs
			.iter()
			.enumerate()
			.map(|(index, (real_path, mount_path))| {
				(index, real_path.as_ref(), mount_path.as_ref())
			})
			.collect();

		reqs.par_iter().for_each(|(index, real_path, mount_path)| {
			let index = *index;

			let (real_path, mount_point) =
				match self.validate_mount_request(real_path.as_ref(), mount_path.as_ref()) {
					Ok(paths) => paths,
					Err(err) => {
						results.lock()[index] = Outcome::Err(err);
						return;
					}
				};

			// Suppose the following two mount points are given:
			// - `/mygame`
			// - `/mygame/myothergame`
			// In two separate mount batches this is valid, but in one batch, it
			// can lead to a parent existence check passing when it shouldn't due
			// to a data race, or the parallel iterator only being given 1 thread
			if reqs
				.iter()
				.any(|(_, _, mpath)| mount_point.is_child_of(mpath))
			{
				results.lock()[index] = Outcome::Err(MountError::Remount);
				return;
			}

			match real_path.metadata() {
				Ok(metadata) => {
					ctx.tracker
						.mount_target
						.fetch_add(metadata.len(), atomic::Ordering::SeqCst);
				}
				Err(err) => {
					results.lock()[index] = Outcome::Err(MountError::Metadata(err));
					return;
				}
			};

			match self.mount_real_unknown(&ctx, &real_path, &mount_point) {
				Ok((new_files, format)) => {
					results.lock()[index] = Outcome::Ok {
						format,
						new_files,
						real_path,
						mount_point,
					};
				}
				Err(err) => {
					results.lock()[index] = Outcome::Err(err);
				}
			}
		});

		let results = results.into_inner();
		let failed = results.iter().any(|outp| matches!(outp, Outcome::Err(_)));
		let mut mounts = Vec::with_capacity(mount_reqs.len());

		// Push new entries into `self.files` if `!failed`
		// Don't push mounts to `self.mounts` yet; we need to fully populate the
		// file tree so we have the context needed to resolve more metadata
		let results: Vec<Result<(), MountError>> = results
			.into_iter()
			.map(|outp| match outp {
				Outcome::Uninit => {
					unreachable!("A VFS mount result was left uninitialized.");
				}
				Outcome::Ok {
					format,
					new_files,
					real_path,
					mount_point,
				} => {
					if failed {
						// See `Error::MountFallthrough`'s docs for rationale
						// on why we return `Ok` here
						return Ok(());
					}

					info!(
						"Mounted: \"{}\" -> \"{}\".",
						real_path.display(),
						mount_point.display(),
					);

					for new_file in new_files {
						let displaced = self.files.insert(VfsKey::new(&new_file.path), new_file);

						debug_assert!(
							displaced.is_none(),
							"A VFS mount operation displaced entry: {}",
							displaced.unwrap().path.display()
						);
					}

					mounts.push(Mount::new(MountInfo {
						id: mount_point
							.file_stem()
							.unwrap()
							.to_str()
							.unwrap()
							.to_string(),
						format,
						kind: MountKind::Misc,
						version: None,
						name: None,
						description: None,
						authors: Vec::default(),
						copyright: None,
						links: Vec::default(),
						real_path: real_path.into_boxed_path(),
						virtual_path: mount_point.into_boxed_path(),
						script_root: None,
					}));

					Ok(())
				}
				Outcome::Err(err) => Err(err),
			})
			.collect();

		debug_assert!(results.len() == mount_reqs.len());

		let ret = Output {
			results,
			orig_files_len,
			orig_mounts_len,
			tracker: ctx.tracker,
		};

		// Time for some housekeeping. If we've failed, clean up after ourselves.
		if failed {
			self.load_fail_cleanup(orig_files_len, orig_mounts_len);
			return ret;
		}

		// Tell directories about their children
		self.populate_dirs();

		// Register mounts; learn as much about them as possible in the process

		for mut mount in mounts {
			mount.info.kind =
				self.resolve_mount_kind(mount.info.format(), mount.info.virtual_path());

			self.resolve_mount_metadata(&mut mount.info);
			self.mounts.push(mount);
		}

		ret
	}

	/// Mount functions dealing in real (i.e. non-archived) files funnel their
	/// operations through here. It returns the deduced format of the file in
	/// question only for the benefit of the top-level function whose job it is
	/// to build the [`MountInfo`].
	fn mount_real_unknown(
		&self,
		ctx: &Context,
		real_path: &Path,
		virt_path: &VPath,
	) -> Result<(Vec<VirtualFile>, MountFormat), MountError> {
		let (format, bytes) = if real_path.is_dir() {
			(MountFormat::Directory, vec![])
		} else {
			let b = match std::fs::read(real_path) {
				Ok(b) => b,
				Err(err) => {
					return Err(MountError::FileRead(err));
				}
			};

			let f = Self::resolve_file_format(&b, virt_path);
			(f, b)
		};

		let res = match format {
			MountFormat::PlainFile => {
				if let Some(leaf) = self.new_leaf_file(virt_path.to_path_buf(), bytes) {
					ctx.tracker.add_mount_progress(leaf.byte_len() as u64);
					Ok(vec![leaf])
				} else {
					Ok(vec![])
				}
			}
			MountFormat::Directory => self.mount_dir(ctx, real_path, virt_path),
			MountFormat::Zip => self.mount_zip(ctx, virt_path, bytes),
			MountFormat::Wad => self.mount_wad(ctx, virt_path, bytes),
		};

		match res {
			Ok(new_files) => Ok((new_files, format)),
			Err(err) => Err(err),
		}
	}

	fn mount_dir(
		&self,
		ctx: &Context,
		real_path: &Path,
		virt_path: &VPath,
	) -> Result<Vec<VirtualFile>, MountError> {
		let mut ret = Vec::default();

		let dir_iter = match std::fs::read_dir(real_path) {
			Ok(r) => r.filter_map(|res| match res {
				Ok(r) => Some(r),
				Err(_) => None,
			}),
			Err(err) => {
				return Err(MountError::DirectoryRead(err));
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

			match self.mount_real_unknown(ctx, &de_real_path, &de_virt_path) {
				Ok((mut new_files, _)) => {
					ret.append(&mut new_files);
				}
				Err(err) => return Err(err),
			}
		}

		ret[1..].par_sort_by(VirtualFile::cmp_name);

		Ok(ret)
	}

	fn mount_wad(
		&self,
		ctx: &Context,
		virt_path: &VPath,
		bytes: Vec<u8>,
	) -> Result<Vec<VirtualFile>, MountError> {
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

		let wad = wad::parse_wad(bytes).map_err(MountError::Wad)?;
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
			// children of that folder
			if ret[index - 1].is_empty()
				&& (is_map_component(&name) || name.eq_ignore_ascii_case("TEXTMAP"))
			{
				let prev_index = ret.len() - 1;
				let prev = ret.get_mut(prev_index).unwrap();
				prev.kind = VirtFileKind::Directory(Vec::with_capacity(10));
				mapfold = Some(index - 1);
			} else if !is_map_component(name.as_str()) {
				mapfold = None;
			}

			let child_path = if let Some(entry_idx) = mapfold {
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
			// (e.g. DOOM2.WAD has two identical `SW18_7` entries)
			// In this case, the last entry clobbers the previous ones

			if let Some(pos) = ret
				.iter()
				.position(|e| e.path.as_ref() == <VPathBuf as AsRef<VPath>>::as_ref(&child_path))
			{
				info!("Overwriting WAD entry: {}", ret[pos].path_str());
				ret.remove(pos);
				index -= 1;
			}

			if let Some(leaf) = self.new_leaf_file(child_path, bytes) {
				ctx.tracker.add_mount_progress(leaf.byte_len() as u64);
				ret.push(leaf);
			}
		}

		Ok(ret)
	}

	fn mount_zip(
		&self,
		ctx: &Context,
		virt_path: &VPath,
		bytes: Vec<u8>,
	) -> Result<Vec<VirtualFile>, MountError> {
		let cursor = Cursor::new(&bytes);
		let mut zip = ZipArchive::new(cursor).map_err(MountError::Zip)?;

		let mut ret = Vec::with_capacity(zip.len() + 1);
		ret.push(Self::new_dir(virt_path.to_path_buf()));

		// First pass creates a directory structure

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

		// Second pass covers leaf nodes

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
				ctx.tracker.add_mount_progress(leaf.byte_len() as u64);
				ret.push(leaf);
			}
		}

		ret[1..].par_sort_by(VirtualFile::cmp_name);

		Ok(ret)
	}

	// Utility /////////////////////////////////////////////////////////////////

	/// Returns a new [empty], [text], or [binary] file, depending on `bytes`,
	/// but only if the VFS size limit configuration permits it. If the file is
	/// oversize, log a warning and return `None`.
	///
	/// [empty]: VirtFileKind::Empty
	/// [text]: VirtFileKind::Text
	/// [binary]: VirtFileKind::Binary
	#[must_use]
	fn new_leaf_file(&self, virt_path: VPathBuf, bytes: Vec<u8>) -> Option<VirtualFile> {
		let kind: VirtFileKind = if bytes.is_empty() {
			VirtFileKind::Empty
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

					VirtFileKind::Text(string.into_boxed_str())
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

					VirtFileKind::Binary(slice)
				}
			}
		};

		Some(VirtualFile {
			path: virt_path.into_boxed_path(),
			kind,
		})
	}

	#[must_use]
	fn new_dir(virt_path: VPathBuf) -> VirtualFile {
		VirtualFile {
			path: virt_path.into_boxed_path(),
			kind: VirtFileKind::Directory(Vec::default()),
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
				return Err(MountError::Canonicalization(err));
			}
		};

		if !real_path.exists() {
			return Err(MountError::FileNotFound(real_path));
		}

		if real_path.is_symlink() {
			return Err(MountError::MountSymlink);
		}

		if real_path.is_hidden() {
			return Err(MountError::MountHidden);
		}

		// Ensure mount point has no invalid characters, isn't reserved, etc.

		self.mount_path_valid(mount_path)?;

		// Ensure mount point path has a parent path

		let mount_point_parent = match mount_path.parent() {
			Some(p) => p,
			None => {
				return Err(MountError::ParentlessMountPoint);
			}
		};

		// Ensure nothing already exists at end of mount point

		if self.file_exists(mount_path) {
			return Err(MountError::Remount);
		}

		// Ensure mount point parent exists

		if !self.file_exists(mount_point_parent) {
			return Err(MountError::MountParentNotFound(
				mount_point_parent.to_path_buf(),
			));
		}

		// All checks passed

		let mut mount_point = VPathBuf::new();

		if !mount_path.starts_with("/") {
			mount_point.push("/");
		}

		mount_point.push(mount_path);

		Ok((real_path, mount_point))
	}

	/// A mount path must:
	/// - Be valid UTF-8.
	/// - Be comprised only of ASCII alphanumerics, dashes, underscores,
	/// and forward slashes.
	/// - Not start with a `.` character.
	/// There is also the following context-sensitive check:
	/// - If nothing has been mounted yet, assert that the file name at the end
	/// of the mount path is the base data's file name.
	/// - If the base data has already been mounted, check that none of the
	/// components in the mount path are a reserved name (ASCII case ignored).
	fn mount_path_valid(&self, path: &VPath) -> Result<(), MountError> {
		let string = match path.to_str() {
			Some(s) => s,
			None => {
				return Err(MountError::InvalidMountPoint(
					path.to_path_buf(),
					"Path is not valid UTF-8.",
				));
			}
		};

		if string.contains(|c: char| {
			!(c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '/' || c == '.')
		}) {
			return Err(MountError::InvalidMountPoint(
				path.to_path_buf(),
				"Path contains a character that isn't an ASCII letter/number, \
				underscore, dash, period, or forward slash.",
			));
		}

		if string.starts_with('.') {
			return Err(MountError::InvalidMountPoint(
				path.to_path_buf(),
				"Path contains a component containing only `.` characters.",
			));
		}

		if self.mounts.is_empty() {
			assert!(
				path.ends_with(crate::BASEDATA_ID),
				"Engine base data is being mounted late."
			);

			return Ok(());
		}

		const RESERVED: &[&str] = &["vile", "viletec", "viletech", "lith", "lithscript"];

		for comp in path.components() {
			match comp {
				std::path::Component::Prefix(_) => {
					unreachable!("A Windows path prefix wasn't filtered out of a mount point.")
				}
				std::path::Component::RootDir => {} // OK
				std::path::Component::CurDir => {
					return Err(MountError::InvalidMountPoint(
						path.to_path_buf(),
						"Path contains an illegal `.` component.",
					));
				}
				std::path::Component::ParentDir => {
					return Err(MountError::InvalidMountPoint(
						path.to_path_buf(),
						"Path contains an illegal `..` component.",
					))
				}
				std::path::Component::Normal(os_str) => {
					let s = os_str.to_str().unwrap();

					for reserved in RESERVED {
						if s.contains(reserved) {
							return Err(MountError::InvalidMountPoint(
								path.to_path_buf(),
								"Path contains a component that's engine-reserved.",
							));
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

		// Heuristics have a precedence hierarchy, so use multiple passes

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

		if info.kind == MountKind::VileTech {
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
			info.version = ingest.version;
			info.name = ingest.name;
			info.description = ingest.description;
			info.authors = ingest.authors;
			info.copyright = ingest.copyright;
			info.links = ingest.links;
			info.script_root = ingest.script_root.map(|vpb| vpb.into_boxed_path());
			return;
		}

		// Try to infer the mount's version based on the mount point

		let mut id = info.id.clone();

		if let Some(vers) = version_from_string(&mut id) {
			info.version = Some(vers);
			info.name = Some(id);
		}
	}
}
