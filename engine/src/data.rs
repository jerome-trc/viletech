//! Management of files, audio, graphics, levels, text, localization, and so on.

pub mod asset;
mod detail;
mod error;
mod ext;
mod interface;
mod mount;
mod pproc;
#[cfg(test)]
mod test;
mod vfs;

use std::{
	marker::PhantomData,
	path::Path,
	sync::{atomic, Arc, Weak},
};

use bevy_egui::egui;
use dashmap::{DashMap, DashSet};
use globset::Glob;
use indexmap::IndexMap;
use parking_lot::Mutex;
use rayon::prelude::*;
use regex::Regex;
use smallvec::SmallVec;

use crate::{lith, utils::path::PathExt, EditorNum, SpawnNum, VPath, VPathBuf};

pub use self::{asset::*, error::*, ext::*, interface::*, vfs::*};

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
/// A footnote on semantics: it is impossible to mount a file that's nested within
/// an archive. If `mymod.zip` contains `myothermod.vpk7`, there's no way to
/// register `myothermod` as a mount in the official sense. It's just a part of
/// `mymod`'s file tree.
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
	pub(self) files: IndexMap<VfsKey, File>, // Q: FNV hashing?
	/// The first element is always the engine's base data (ID `viletech`),
	/// but every following element is user-specified, including their order.
	pub(self) mounts: Vec<MountInfo>,
	pub(self) modules: Vec<lith::Module>,
	/// The "source of truth" for record pointers.
	pub(self) assets: DashMap<AssetKey, Arc<Record>>,
	/// Asset storage without namespacing. Thus, requesting `MAP01` returns
	/// the last element in the array behind that key, as doom.exe would if
	/// loading multiple WADs with similarly-named entries.
	pub(self) nicknames: DashMap<Box<str>, SmallVec<[Weak<Record>; 2]>>,
	/// See the key type's documentation for background details.
	/// Stored records always wrap a [`Blueprint`].
	///
	/// [`Blueprint`]: asset::Blueprint
	pub(self) editor_nums: DashMap<EditorNum, SmallVec<[Weak<Record>; 2]>>,
	/// See the key type's documentation for background details.
	/// Stored records always wrap a [`Blueprint`].
	///
	/// [`Blueprint`]: asset::Blueprint
	pub(self) spawn_nums: DashMap<SpawnNum, SmallVec<[Weak<Record>; 2]>>,
	// FNV/aHash for maps using small key types?
}

impl Catalog {
	/// This is an end-to-end function that reads physical files, fills out the
	/// VFS, and then post-processes the files to decompose them into assets.
	/// Much of the important things to know are in the documentation for
	/// [`LoadRequest`]. The range of possible errors is documented by
	/// [`MountError`].
	///
	/// Notes:
	/// - The order of pre-existing VFS entries and mounts is unchanged upon success.
	/// - Returned `LoadOutcome` objects parallel the given mount requests.
	/// - This function is atomic; if one load operation fails, all of them fail,
	/// and the catalog's state is left entirely unchanged.
	/// - Each load request is fulfilled in parallel using [`rayon`]'s global
	/// thread pool, but the caller thread itself gets blocked.
	#[must_use = "mounting may return errors which should be handled"]
	pub fn load<RP: AsRef<Path>, MP: AsRef<VPath>>(
		&mut self,
		request: LoadRequest<RP, MP>,
	) -> LoadOutcome {
		if request.paths.len() > (u8::MAX as usize) {
			unimplemented!("Loading more than 255 files/folders is unsupported.");
		}

		let new_mounts = self.mounts.len()..(self.mounts.len() + request.paths.len());

		let ctx = mount::Context {
			// Build a dummy tracker if none was given to avoid branching later
			// and simplify the rest of the loading code.
			tracker: request
				.tracker
				.unwrap_or_else(|| Arc::new(LoadTracker::default())),
			errors: Mutex::new(vec![]),
		};

		ctx.tracker
			.mount_target
			.store(request.paths.len() as u8, atomic::Ordering::SeqCst);

		// Note to reader: check `./mount.rs`.
		let m_output = self.mount(&request.paths, ctx);

		if m_output.any_errs() {
			return LoadOutcome::MountFail {
				errors: m_output.errors,
			};
		}

		let ctx = pproc::Context {
			tracker: m_output.tracker,
			orig_files_len: m_output.orig_files_len,
			orig_mounts_len: m_output.orig_mounts_len,
			added: DashSet::default(),
			new_mounts,
			errors: Mutex::new(vec![]),
		};

		let p_output = self.postproc(ctx);

		if p_output.any_errs() {
			LoadOutcome::PostProcFail {
				errors: p_output.errors,
			}
		} else {
			LoadOutcome::Ok {
				mount: m_output.errors,
				pproc: p_output.errors,
			}
		}
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
			let vpath = mount.virtual_path();

			self.files.retain(|_, entry| !entry.path.is_child_of(vpath));
		}

		self.clear_dirs();
		self.populate_dirs();
		self.clean();
	}

	#[must_use]
	pub fn get_file(&self, path: impl AsRef<VPath>) -> Option<FileRef> {
		self.files.get(&VfsKey::new(path)).map(|file| FileRef {
			catalog: self,
			file,
		})
	}

	/// Note that `A` here is a filter on the type that comes out of the lookup,
	/// rather than an assertion that the asset under `id` is that type, so this
	/// returns an `Option` rather than a [`Result`].
	#[must_use]
	pub fn get_asset<A: Asset>(&self, id: &str) -> Option<AssetRef<A>> {
		let key = AssetKey::new::<A>(id);

		self.assets.get(&key).map(|kvp| AssetRef {
			catalog: self,
			kvp,
			phantom: PhantomData,
		})
	}

	/// Find an [`Actor`] [`Blueprint`] by a 16-bit editor number.
	/// The last blueprint assigned the given number is what gets returned.
	///
	/// [`Actor`]: crate::sim::actor::Actor
	#[must_use]
	pub fn bp_by_ednum(&self, num: EditorNum) -> Option<Handle<Blueprint>> {
		self.editor_nums.get(&num).map(|kvp| {
			let stack = kvp.value();

			let last = stack
				.last()
				.expect("Catalog cleanup missed an empty ed-num stack.");

			Handle::from(
				last.upgrade()
					.expect("Catalog cleanup missed a dangling ed-num weak pointer."),
			)
		})
	}

	/// Find an [`Actor`] [`Blueprint`] by a 16-bit spawn number.
	/// The last blueprint assigned the given number is what gets returned.
	///
	/// [`Actor`]: crate::sim::actor::Actor
	#[must_use]
	pub fn bp_by_spawnnum(&self, num: SpawnNum) -> Option<Handle<Blueprint>> {
		self.spawn_nums.get(&num).map(|kvp| {
			let stack = kvp.value();

			let last = stack
				.last()
				.expect("Catalog cleanup missed an empty spawn-num stack.");

			Handle::from(
				last.upgrade()
					.expect("Catalog cleanup missed a dangling spawn-num weak pointer."),
			)
		})
	}

	#[must_use]
	pub fn file_exists(&self, path: impl AsRef<VPath>) -> bool {
		self.files.contains_key(&VfsKey::new(path))
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

	#[must_use]
	pub fn last_asset_by_nick<A: Asset>(&self, nick: &str) -> Option<Handle<A>> {
		self.nicknames.get(nick).map(|kvp| {
			let stack = kvp.value();

			let last = stack
				.last()
				.expect("Catalog cleanup missed an empty nicknamed stack.");

			Handle::from(
				last.upgrade()
					.expect("Catalog cleanup missed a dangling nicknamed weak pointer."),
			)
		})
	}

	#[must_use]
	pub fn first_asset_by_nick<A: Asset>(&self, nick: &str) -> Option<Handle<A>> {
		self.nicknames.get(nick).map(|kvp| {
			let stack = kvp.value();

			let last = stack
				.last()
				.expect("Catalog cleanup missed an empty nicknamed stack.");

			Handle::from(
				last.upgrade()
					.expect("Catalog cleanup missed a dangling nicknamed weak pointer."),
			)
		})
	}

	#[must_use]
	pub fn mounts(&self) -> &[MountInfo] {
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

	/// Draw the egui-based developer/debug/diagnosis menu for the VFS.
	pub fn ui_vfs(&self, ctx: &egui::Context, ui: &mut egui::Ui) {
		self.ui_vfs_impl(ctx, ui);
	}

	pub fn ui_assets(&self, ctx: &egui::Context, ui: &mut egui::Ui) {
		self.ui_assets_impl(ctx, ui);
	}
}

impl Default for Catalog {
	fn default() -> Self {
		let root = File {
			path: VPathBuf::from("/").into_boxed_path(),
			kind: FileKind::Directory(vec![]),
		};

		let key = VfsKey::new(&root.path);

		Self {
			config: Config::default(),
			files: indexmap::indexmap! { key => root },
			mounts: vec![],
			modules: vec![],
			assets: DashMap::default(),
			nicknames: DashMap::default(),
			editor_nums: DashMap::default(),
			spawn_nums: DashMap::default(),
		}
	}
}

#[derive(Debug)]
pub enum LoadOutcome {
	/// One or more fatal errors prevented a successful VFS mount.
	MountFail {
		/// Every *new* mount gets a sub-vec, but that sub-vec may be empty.
		errors: Vec<Vec<MountError>>,
	},
	/// Mounting succeeeded, but one or more fatal errors
	/// prevented successful asset post-processing.
	PostProcFail {
		/// Every *new* mount gets a sub-vec, but that sub-vec may be empty.
		errors: Vec<Vec<PostProcError>>,
	},
	/// Loading was successful, but non-fatal errors or warnings may have arisen.
	Ok {
		/// Every *new* mount gets a sub-vec, but that sub-vec may be empty.
		mount: Vec<Vec<MountError>>,
		/// Every *new* mount gets a sub-vec, but that sub-vec may be empty.
		pproc: Vec<Vec<PostProcError>>,
	},
}

impl LoadOutcome {
	#[must_use]
	pub fn num_errs(&self) -> usize {
		match self {
			LoadOutcome::MountFail { errors } => {
				errors.iter().fold(0, |acc, subvec| acc + subvec.len())
			}
			LoadOutcome::PostProcFail { errors } => {
				errors.iter().fold(0, |acc, subvec| acc + subvec.len())
			}
			LoadOutcome::Ok { mount, pproc } => {
				mount.iter().fold(0, |acc, subvec| acc + subvec.len())
					+ pproc.iter().fold(0, |acc, subvec| acc + subvec.len())
			}
		}
	}
}

// (RAT) If you're reading this, congratulations! You've found something special.
// This module sub-tree is, historically speaking, the most tortured code in VileTech.
// The Git history doesn't even reflect half of the reworks the VFS has undergone.
