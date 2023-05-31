//! All the methods involved in building the virtual file tree.

use std::{
	collections::hash_map,
	path::{Path, PathBuf},
	sync::Arc,
};

use dashmap::DashMap;
use parking_lot::Mutex;
use rayon::prelude::*;
use util::{path::PathExt, Outcome, SendTracker};

use crate::{
	error::{MountError, MountErrorKind, MountPointError},
	file::{self, File},
	FileKey, MountFormat, MountInfo, VPath, VPathBuf, VirtualFs,
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

		let file::Content::Directory(children) = &mut parent.value_mut().content else {
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
			#[cfg(feature = "bevy")]
			bevy::prelude::info!(
				"Mounted: \"{}\" -> \"{}\".",
				real_path.display(),
				mount_point.display(),
			);

			if format != MountFormat::Wad {
				new_files.par_iter_mut().for_each(|mut kvp| {
					if let file::Content::Directory(children) = &mut kvp.value_mut().content {
						children.par_sort_unstable();
					}
				});
			}

			for (key, new_file) in new_files {
				match self.files.entry(key) {
					hash_map::Entry::Occupied(occu) => panic!(
						"A VFS bulk insertion displaced entry: {}",
						occu.key().display(),
					),
					hash_map::Entry::Vacant(vacant) => {
						vacant.insert(new_file);
					}
				}
			}

			let subtree_parent_path = mount_point.parent().unwrap();
			let subtree_parent = self.files.get_mut(subtree_parent_path).unwrap();

			if let file::Content::Directory(children) = &mut subtree_parent.content {
				children.insert(mount_point.clone().into());
				children.par_sort_unstable();
			} else {
				unreachable!()
			}

			let errors = std::mem::take(&mut ctx.errors[i]);
			ret.push(errors.into_inner());

			self.mounts.push(MountInfo {
				id: mount_point
					.file_stem()
					.unwrap()
					.to_str()
					.unwrap()
					.to_string(),
				format,
				real_path,
				mount_point,
			});
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
		todo!()
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
		match util::io::is_valid_wad(bytes, bytes.len().try_into().unwrap()) {
			Ok(is_wad) => {
				if is_wad {
					return MountFormat::Wad;
				}
			}
			Err(err) => {
				#[cfg(feature = "bevy")]
				bevy::prelude::warn!(
					"Failed to determine if file is a WAD: {}\r\n\t\
					Error: {err}\r\n\t\
					It will likely be treated as an unknown file.",
					virt_path.display()
				);
			}
		}

		if util::io::is_zip(bytes) {
			return MountFormat::Zip;
		}

		MountFormat::PlainFile
	}
}
