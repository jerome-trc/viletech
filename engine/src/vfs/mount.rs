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
use zip::{read::ZipFile, ZipArchive};

use crate::{
	utils::{io::*, path::PathExt},
	wad, Outcome, SendTracker, VPath, VPathBuf,
};

use super::{
	error::{MountErrorKind, MountPointError},
	file::FileKey,
	File, MountError, MountFormat, VirtualFs,
};

#[derive(Debug)]
pub(super) struct Context {
	tracker: Arc<SendTracker>,
	/// Returning errors through the mounting call tree is somewhat
	/// inflexible, so pass an array down through the context instead.
	errors: Vec<Mutex<Vec<MountError>>>,
	/// For bypassing checks on reserved mount points.
	basedata: bool,
}

impl Context {
	#[must_use]
	pub(super) fn new(
		tracker: Option<Arc<SendTracker>>,
		load_order_len: usize,
		basedata: bool,
	) -> Self {
		// Build a dummy tracker if none was given to simplify later code.
		// Q: Branching might yield a speed increase compared to wasted atomic
		// operations, but is there reason to bother?
		let tracker = tracker.unwrap_or_else(|| Arc::new(SendTracker::default()));
		tracker.set_target(load_order_len);

		let mut errors = vec![];
		errors.resize_with(load_order_len, Mutex::default);

		Self {
			tracker,
			errors,
			basedata,
		}
	}
}

/// Context relevant to one single load order item.
#[derive(Debug)]
pub(self) struct SubContext<'ctx> {
	pub(self) tracker: &'ctx Arc<SendTracker>,
	pub(self) errors: &'ctx Mutex<Vec<MountError>>,
	pub(self) files: DashMap<FileKey, File>,
}

impl SubContext<'_> {
	fn add_file(&self, path: VPathBuf, file: File) {
		let parent_key: FileKey = path.parent().unwrap().into();
		let key: FileKey = path.into();
		self.files.insert(key.clone(), file);

		let mut parent = match self.files.get_mut(&parent_key) {
			Some(p) => p,
			None => return, // This is a subtree root. It will have to be handled later.
		};

		let File::Directory(children) = parent.value_mut() else {
			unreachable!("Parent of {} is not a directory.", key.display());
		};

		children.insert(key);
	}
}

#[derive(Debug)]
struct Success {
	format: MountFormat,
	new_files: DashMap<FileKey, File>,
	real_path: PathBuf,
	mount_point: VPathBuf,
}

type Output = Vec<Vec<MountError>>;

impl VirtualFs {
	pub(super) fn mount_impl(
		&mut self,
		load_order: &[(impl AsRef<Path>, impl AsRef<VPath>)],
		mut ctx: Context,
	) -> Outcome<Output, Output> {
		if ctx.tracker.is_cancelled() {
			return Outcome::Cancelled;
		}

		let mut successes = vec![];
		successes.resize_with(load_order.len(), || None);
		let successes = Mutex::new(successes);

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

			let (real_path, mount_point) =
				match self.validate_mount_request(&ctx, real_path, mount_point) {
					Ok(paths) => paths,
					Err(err) => {
						subctx.errors.lock().push(err);
						return;
					}
				};

			match self.mount_real_unknown(&subctx, &real_path, &mount_point) {
				Outcome::Ok(format) => {
					ctx.tracker.add_to_progress(1);

					successes.lock()[*i] = Some(Success {
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

		let successes = successes.into_inner();
		let failed = successes.iter().any(|s| s.is_none());

		if failed {
			let errors: Vec<Vec<MountError>> = std::mem::take(&mut ctx.errors)
				.into_iter()
				.map(|mutex| mutex.into_inner())
				.collect();
			ctx.tracker.finish();
			return Outcome::Err(errors);
		}

		let mut ret = vec![];

		for (
			i,
			Success {
				format,
				new_files,
				real_path,
				mount_point,
			},
		) in successes.into_iter().map(|s| s.unwrap()).enumerate()
		{
			info!(
				"Mounted: \"{}\" -> \"{}\".",
				real_path.display(),
				mount_point.display(),
			);

			if format != MountFormat::Wad {
				new_files.par_iter_mut().for_each(|mut kvp| {
					if let File::Directory(children) = kvp.value_mut() {
						children.par_sort_unstable();
					}
				});
			}

			for (key, new_file) in new_files {
				match self.files.entry(key) {
					indexmap::map::Entry::Occupied(occu) => panic!(
						"A VFS bulk insertion displaced entry: {}",
						occu.key().display(),
					),
					indexmap::map::Entry::Vacant(vacant) => {
						vacant.insert(new_file);
					}
				}
			}

			let subtree_parent_path = mount_point.parent().unwrap();
			let subtree_parent = self.files.get_mut(subtree_parent_path).unwrap();

			if let File::Directory(children) = subtree_parent {
				children.insert(mount_point.into());
				children.par_sort_unstable();
			} else {
				unreachable!()
			}

			let errors = std::mem::take(&mut ctx.errors[i]);
			ret.push(errors.into_inner());
		}

		Outcome::Ok(ret)
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
				ctx.add_file(virt_path.to_path_buf(), File::new_leaf(bytes));
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

		ctx.add_file(
			virt_path.to_path_buf(),
			File::Directory(indexmap::indexset! {}),
		);

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
		fn maybe_map_marker(kvp: &(VPathBuf, File)) -> bool {
			let path_str = kvp.0.to_string_lossy();
			let fpfx = path_str.as_ref().split('.').next().unwrap();
			// FIXME: FraggleScript occupies markers, so these may not be empty.
			kvp.1.is_empty() && !is_map_component(fpfx)
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

		files.push((
			virt_path.to_path_buf(),
			File::Directory(indexmap::indexset! {}),
		));

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
				prev.1 = File::Directory(IndexSet::with_capacity(10));
				mapfold = Some(index - 1);
			} else if !is_map_component(name.as_str()) {
				mapfold = None;
			}

			let mut child_path = if let Some(entry_idx) = mapfold {
				let fname = files[entry_idx].0.file_name().unwrap().to_string_lossy();
				// virt_path currently: `/mount_point`
				let mut cpath = virt_path.join(VPath::new(fname.as_ref()));
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

			if let Some(pos) = files.iter().position(|(path, _)| {
				<VPathBuf as AsRef<VPath>>::as_ref(path)
					== <VPathBuf as AsRef<VPath>>::as_ref(&child_path)
			}) {
				if !files[pos].1.is_dir() {
					files.insert(
						pos,
						(
							files[pos].0.clone(),
							File::Directory(indexmap::indexset! {}),
						),
					);

					let prev_path = std::mem::take(&mut files[pos + 1].0);

					let mut prev_path = prev_path.to_path_buf();
					prev_path.push("000");

					files[pos + 1].0 = prev_path;
					child_path.push("001");
				} else {
					let num_children = files
						.iter()
						.filter(|(vp, _)| vp.parent().unwrap() == child_path)
						.count();

					child_path.push(format!("{num_children:03}"));
				}
			}

			files.push((child_path, File::new_leaf(bytes)));
		}

		for (p, file) in files {
			ctx.add_file(p, file);
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

		let base_zip = match zip {
			Ok(z) => z,
			Err(err) => return Outcome::Err(err),
		};

		ctx.add_file(
			virt_path.to_path_buf(),
			File::Directory(indexmap::indexset! {}),
		);

		(0..base_zip.len()).par_bridge().try_for_each(|i| {
			let mut zip = base_zip.clone();

			if ctx.tracker.is_cancelled() {
				return None;
			}

			let zf = match zip.by_index(i) {
				Ok(e) => e,
				Err(err) => {
					ctx.errors.lock().push(MountError {
						path: virt_path.to_path_buf(),
						kind: MountErrorKind::ZipFileGet(i, err),
					});

					return Some(());
				}
			};

			let zfpath = match zf.enclosed_name() {
				Some(p) => p,
				None => {
					ctx.errors.lock().push(MountError {
						path: virt_path.to_path_buf(),
						kind: MountErrorKind::ZipFileName(zf.name().to_string()),
					});

					return Some(());
				}
			};

			let vpath = [virt_path, zfpath].iter().collect();

			if zf.is_file() {
				self.mount_zip_leaf(ctx, virt_path, zf, vpath);
			} else {
				ctx.add_file(vpath, File::Directory(indexmap::indexset! {}));
			}

			Some(())
		});

		if ctx.tracker.is_cancelled() {
			return Outcome::Cancelled;
		}

		Outcome::Ok(())
	}

	fn mount_zip_leaf(
		&self,
		ctx: &SubContext,
		parent_path: &VPath,
		mut zf: ZipFile,
		vpath: VPathBuf,
	) {
		let size = zf.size();

		let mut bytes = Vec::with_capacity(size as usize);

		match std::io::copy(&mut zf, &mut bytes) {
			Ok(count) => {
				if count != size {
					ctx.errors.lock().push(MountError {
						path: parent_path.to_path_buf(),
						kind: MountErrorKind::ZipFileRead {
							name: zf
								.enclosed_name()
								.unwrap_or_else(|| Path::new(zf.name()))
								.to_path_buf(),
							err: None,
						},
					});

					return;
				}
			}
			Err(err) => {
				ctx.errors.lock().push(MountError {
					path: parent_path.to_path_buf(),
					kind: MountErrorKind::ZipFileRead {
						name: zf
							.enclosed_name()
							.unwrap_or_else(|| Path::new(zf.name()))
							.to_path_buf(),
						err: Some(err),
					},
				});

				return;
			}
		};

		ctx.add_file(vpath, File::new_leaf(bytes));
	}

	// Details /////////////////////////////////////////////////////////////////

	fn validate_mount_request(
		&self,
		ctx: &Context,
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

		self.mount_path_valid(ctx, mount_path)?;

		// Ensure mount point path has a parent path.

		let mount_point_parent = match mount_path.parent() {
			Some(p) => VPathBuf::from("/").join(p),
			None => {
				return Err(MountError {
					path: mount_path.to_path_buf(),
					kind: MountErrorKind::ParentlessMountPoint,
				});
			}
		};

		// Ensure nothing already exists at end of mount point.

		if self.contains(mount_path) {
			return Err(MountError {
				path: mount_path.to_path_buf(),
				kind: MountErrorKind::Remount,
			});
		}

		// Ensure mount point parent exists.

		if !self.contains(&mount_point_parent) {
			return Err(MountError {
				path: mount_path.to_path_buf(),
				kind: MountErrorKind::MountParentNotFound(mount_point_parent),
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
	/// - Not be reserved (if `ctx.basedata` is `false`).
	fn mount_path_valid(&self, ctx: &Context, path: &VPath) -> Result<(), MountError> {
		path.to_str().ok_or_else(|| MountError {
			path: path.to_path_buf(),
			kind: MountErrorKind::InvalidMountPoint(MountPointError::InvalidUtf8),
		})?;

		if ctx.basedata {
			return Ok(());
		}

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
					let comp_str = os_str.to_string_lossy();

					if self
						.config
						.reserved_mount_points
						.iter()
						.any(|rmp| rmp.eq_ignore_ascii_case(comp_str.as_ref()))
					{
						return Err(MountError {
							path: path.to_path_buf(),
							kind: MountErrorKind::InvalidMountPoint(MountPointError::Reserved),
						});
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
}
