//! Management of files, audio, graphics, levels, text, localization, and so on.

pub mod asset;
mod detail;
mod error;
mod ext;
mod file;
mod interface;
mod mount;
mod pproc;
#[cfg(test)]
mod test;

use std::{path::Path, sync::Arc};

use globset::Glob;
use indexmap::IndexMap;
use rayon::prelude::*;
use regex::Regex;

use crate::{utils::path::PathExt, ShortId, VPath, VPathBuf};

pub use asset::*;
pub use error::{
	Asset as AssetError, Load as LoadError, Mount as MountError, PostProc as PostProcError,
	Vfs as VfsError,
};
pub use ext::*;
pub use file::*;
pub use interface::*;

use self::detail::{AssetKey, Config, VfsKey};

/// The data catalog is the heart of file and asset management in VileTech.
/// "Physical" files are "mounted" into one cohesive virtual file system (VFS)
/// tree that makes it easy for all other parts of the engine to access any given
/// unit of data, without exposing any details of the user's real underlying machine.
///
/// A mounted file or directory has the same tree structure in the virtual FS as
/// in the physical one, although binary files are converted into more useful
/// forms (e.g. decoding sounds and images) if their format can be easily identified.
/// Otherwise, they're left as-is.
///
/// Any given unit of data or [`Asset`] is stored in a [`Record`] and kept behind
/// an [`Arc`], allowing other parts of the engine to take out high-speed
/// [`Handle`]s to something and safely access it lock-free.
///
/// Some miscellaneous notes on semantics:
/// - It's impossible to mount a file that's nested within an archive. If
/// `mymod.zip` contains `myothermod.zip`, there's no way to register `myothermod`
/// as a mount in the official sense. It's just a part of `mymod`'s file tree.
/// - If, for example, a zip file is mounted, and within that zip is a WAD, the
/// WAD is not considered a "mount" like the zip.
#[derive(Debug)]
pub struct Catalog {
	pub(self) config: Config,
	/// Element 0 is always the root node, under virtual path `/`.
	///
	/// The choice to use an `IndexMap` here is very deliberate.
	/// - Directory contents can be stored in an alphabetically-sorted way.
	/// - Ordering is preserved for WAD entries.
	/// - Exact-path lookups are fast.
	/// - Memory contiguity means that linear searches are non-pessimized.
	/// - If a load fails, restoring the previous state is simple truncation.
	pub(self) files: IndexMap<VfsKey, VirtualFile>, // Q: FNV hashing?
	/// The first element is always the engine's base data (ID `viletech`),
	/// but every following element is user-specified, including their order.
	pub(self) mounts: Vec<Mount>,
}

impl Catalog {
	/// This is an end-to-end function that reads physical files, fills out the
	/// VFS, and then post-processes the files to decompose them into assets.
	/// Much of the important things to know are in the documentation for
	/// [`LoadRequest`]. The range of possible errors is documented by
	/// [`MountError`].
	///
	/// Notes:
	/// - The order of pre-existing entries and mounts is unchanged upon success.
	/// - Returned errors parallel the given mount requests.
	/// - This function is atomic; if one mount operation fails, all of them fail,
	/// and the VFS's state is left entirely unchanged.
	/// - Each mount request is fulfilled in parallel using [`rayon`]'s global
	/// thread pool, but the caller thread itself gets blocked.
	#[must_use = "mounting may return errors which should be handled"]
	pub fn load<RP: AsRef<Path>, MP: AsRef<VPath>>(
		&mut self,
		request: LoadRequest<RP, MP>,
	) -> Vec<Result<(), LoadError>> {
		let ctx = mount::Context {
			// Build a dummy tracker if none was given to avoid branching later
			// and simplify the rest of the loading code
			tracker: request
				.tracker
				.unwrap_or_else(|| Arc::new(LoadTracker::default())),
		};

		// Note to reader: check ./mount.rs
		let output = self.mount(request.paths, ctx);

		if output.any_errs() {
			return output
				.results
				.into_iter()
				.map(|res| match res {
					Ok(()) => Ok(()),
					Err(err) => Err(LoadError::Mount(err)),
				})
				.collect();
		}

		let ctx = pproc::Context {
			project: request.project,
			tracker: output.tracker,
			orig_files_len: output.orig_files_len,
			orig_mounts_len: output.orig_mounts_len,
		};

		let output = self.postproc(ctx);

		output
			.results
			.into_iter()
			.map(|res| match res {
				Ok(()) => Ok(()),
				Err(err) => Err(LoadError::PostProc(err)),
			})
			.collect()
	}

	/// Like [`Self::load`], but performs no file culling, asset loading, or LithScript
	/// compilation. Used for preparing the engine's base data, and for testing.
	///
	/// See the aforementioned function's docs; most of the same caveats apply.
	/// Additionally, see [`LoadRequest`] to better understand the `mounts` parameter.
	pub fn load_simple(
		&mut self,
		mounts: &[(impl AsRef<Path>, impl AsRef<Path>)],
	) -> Vec<Result<(), MountError>> {
		let ctx = mount::Context {
			// Build a dummy tracker if none was given to avoid branching later
			// and simplify the rest of the loading code
			tracker: Arc::new(LoadTracker::default()),
		};

		// Note to reader: check ./mount.rs
		self.mount(mounts, ctx).results
	}

	/// Keep the first `len` mounts. Remove the rest, along their files.
	/// If `len` is greater than the number of mounts, this function is a no-op.
	pub fn truncate(&mut self, len: usize) {
		if len == 0 {
			self.files.clear();
			self.mounts.clear();
			return;
		} else if len >= self.mounts.len() {
			return;
		}

		for mount in self.mounts.drain(len..) {
			let vpath = mount.info.virtual_path();

			self.files.retain(|_, entry| !entry.path.is_child_of(vpath));
		}

		self.clear_dirs();
		self.populate_dirs();
	}

	#[must_use]
	pub fn get_file(&self, path: impl AsRef<VPath>) -> Option<FileRef> {
		self.files.get(&VfsKey::new(path)).map(|file| FileRef {
			catalog: self,
			file,
		})
	}

	/// Note that `T` here is a filter on the type that comes out of the lookup,
	/// rather than an assertion that the asset under `id` is that type, so this
	/// returns an `Option` rather than a [`Result`].
	#[must_use]
	pub fn get_asset<A: Asset>(&self, id: &str) -> Option<Arc<Record>> {
		let key = AssetKey::new::<A>(id);

		self.mounts
			.par_iter()
			.find_map_any(|mount| mount.assets.get(&key))
			.map(|kvp| kvp.value().clone())
	}

	#[must_use]
	pub fn file_exists(&self, path: impl AsRef<VPath>) -> bool {
		let key = VfsKey::new(path);
		self.files.contains_key(&key)
	}

	pub fn all_files(&self) -> impl Iterator<Item = FileRef> {
		self.files.iter().map(|(_, file)| FileRef {
			catalog: self,
			file,
		})
	}

	/// Note that WAD files will be yielded out of their original order, and
	/// all other files will not exhibit the alphabetical sorting with which
	/// they are internally stored.
	#[must_use = "iterators are lazy and do nothing unless consumed"]
	pub fn all_files_par(&self) -> impl ParallelIterator<Item = FileRef> {
		self.all_files().par_bridge()
	}

	pub fn get_files_glob(&self, pattern: Glob) -> impl Iterator<Item = FileRef> {
		let glob = pattern.compile_matcher();

		self.files.iter().filter_map(move |(_, file)| {
			if glob.is_match(&file.path) {
				Some(FileRef {
					catalog: self,
					file,
				})
			} else {
				None
			}
		})
	}

	/// Note that WAD files will be yielded out of their original order, and
	/// all other files will not exhibit the alphabetical sorting with which
	/// they are internally stored.
	#[must_use = "iterators are lazy and do nothing unless consumed"]
	pub fn get_files_glob_par(&self, pattern: Glob) -> impl ParallelIterator<Item = FileRef> {
		self.get_files_glob(pattern).par_bridge()
	}

	pub fn get_files_regex(&self, pattern: Regex) -> impl Iterator<Item = FileRef> {
		self.files.iter().filter_map(move |(_, file)| {
			if pattern.is_match(file.path_str()) {
				Some(FileRef {
					catalog: self,
					file,
				})
			} else {
				None
			}
		})
	}

	/// Note that WAD files will be yielded out of their original order, and
	/// all other files will not exhibit the alphabetical sorting with which
	/// they are internally stored.
	#[must_use = "iterators are lazy and do nothing unless consumed"]
	pub fn get_files_regex_par(&self, pattern: Regex) -> impl ParallelIterator<Item = FileRef> {
		self.get_files_regex(pattern).par_bridge()
	}

	/// Finds the last-loaded asset by a given ID and type.
	#[must_use]
	pub fn get_asset_shortid<A: Asset>(&self, shortid: ShortId) -> Option<Arc<Record>> {
		self.mounts
			.par_iter()
			.find_map_last(|mount| mount.shortid_map.get(&shortid))
			.map(|kvp| {
				kvp.value()
					.upgrade()
					.expect("A dangling short-ID weak pointer wasn't garbage-collected.")
			})
	}

	/// Allow altering a single asset record in place.
	///
	/// Returns [`AssetError::NotFound`] if no records of a matching ID and type
	/// are found; returns `Ok(None)` if the record has one or more [`Handle`]s
	/// pointing to it.
	pub fn try_mutate<A: Asset>(&mut self, id: &str) -> Result<asset::RefMut, AssetError> {
		let key = AssetKey::new::<A>(id);

		match self
			.mounts
			.par_iter_mut()
			.find_map_last(|ns| ns.assets.get_mut(&key))
		{
			Some(entry) => {
				let strong_count = Arc::strong_count(entry.value());

				if strong_count > 1 {
					Err(AssetError::Immutable(strong_count - 1))
				} else {
					Ok(asset::RefMut(entry))
				}
			}
			None => Err(AssetError::NotFound(id.to_string())),
		}
	}

	#[must_use]
	pub fn mounts(&self) -> &[Mount] {
		&self.mounts
	}

	#[must_use]
	pub fn config_get(&self) -> ConfigGet {
		ConfigGet(self)
	}

	#[must_use]
	pub fn config_set(&mut self) -> ConfigSet {
		ConfigSet(self)
	}

	/// The returned value reflects only the footprint of the content of the
	/// virtual files themselves; the size of the data structures isn't included,
	/// since it's trivial next to the size of large text files and binary blobs.
	#[must_use]
	pub fn vfs_mem_usage(&self) -> usize {
		self.files
			.par_values()
			.fold(|| 0_usize, |acc, file| acc + file.byte_len())
			.sum()
	}
}

impl Default for Catalog {
	fn default() -> Self {
		let root = VirtualFile {
			path: VPathBuf::from("/").into_boxed_path(),
			kind: VirtFileKind::Directory(Vec::default()),
		};

		let key = VfsKey::new(&root.path);

		Self {
			config: Config::default(),
			files: indexmap::indexmap! { key => root },
			mounts: vec![],
		}
	}
}

// [Rat] If you're reading this, congratulations! You've found something special.
// This module sub-tree is, historically speaking, the most tortured code in VileTech.
// The Git history doesn't even reflect half of the reworks the VFS has undergone.
