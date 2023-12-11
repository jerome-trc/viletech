//! Management of files, audio, graphics, levels, text, localization, and so on.

mod config;
pub mod dobj;
mod error;
mod gui;
mod prep;

#[cfg(test)]
mod test;

use std::{
	any::TypeId,
	hash::{Hash, Hasher},
	path::PathBuf,
	sync::Arc,
};

use bevy::prelude::Resource;
use bevy_egui::egui;
use dashmap::DashMap;
use parking_lot::{Mutex, RwLock};
use rayon::prelude::*;
use rustc_hash::FxHasher;
use smallvec::SmallVec;
use util::{EditorNum, Outcome, SendTracker, SpawnNum};
use vfs::VPathBuf;

use crate::vfs::{FileRef, MountError, MountInfo, MountOutcome, MountRequest, VirtualFs};

use self::{
	dobj::{Blueprint, DataRef, Datum, DatumStore},
	gui::DevGui,
};

pub use self::{config::*, error::*};

/// The data catalog is the heart of file and game data management in VileTech.
/// "Physical" files are "mounted" into one cohesive virtual file system (VFS)
/// tree that makes it easy for all other parts of the engine to access any given
/// unit of data, without exposing any details of the user's real underlying machine.
///
/// A mounted file or directory has the same tree structure in the virtual FS as
/// in the physical one, although binary files are converted into more useful
/// forms (e.g. decoding sounds and images) if their format can be easily identified.
/// Otherwise, they're left as-is.
///
/// Any given unit of data or [`Datum`] is stored behind an [`Arc`], allowing
/// other parts of the engine to take out high-speed [pointers] to something and
/// safely access it without passing through locks or casts.
///
/// The catalog works in phases; files considered "engine basedata" are always in
/// the VFS by necessity, but everything else is either in a "fully loaded" or
/// "unloaded" state.
///
/// A footnote on semantics: it is impossible to mount a file that's nested within
/// an archive. If `mymod.zip` contains `myothermod.vpk7`, there's no way to
/// register `myothermod` as a mount in the official sense. It's just a part of
/// `mymod`'s file tree.
///
/// [pointers]: dobj::Handle
#[derive(Debug, Resource)]
pub struct Catalog {
	config: Config,
	/// See [`Self::new`]; mounts given as `basedata` through that function are
	/// always present here.
	vfs: VirtualFs,
	dobjs: dashmap::ReadOnlyView<DatumKey, Arc<dyn DatumStore>>,
	/// Datum lookup table without namespacing. Thus, requesting `MAP01` returns
	/// the last element in the array behind that key, as doom.exe would if
	/// loading multiple WADs with similarly-named entries. Also contains names
	/// assigned via [`SNDINFO`](https://zdoom.org/wiki/SNDINFO).
	nicknames: dashmap::ReadOnlyView<DatumKey, SmallVec<[Arc<dyn DatumStore>; 2]>>,
	/// See the key type's documentation for background details.
	/// These are always backed by a [`Blueprint`]; they are only `dyn` for the
	/// benefit of [`DataRef`].
	editor_nums: dashmap::ReadOnlyView<EditorNum, SmallVec<[Arc<dyn DatumStore>; 2]>>,
	/// See the key type's documentation for background details.
	/// These are always backed by a [`Blueprint`]; they are only `dyn` for the
	/// benefit of [`DataRef`].
	spawn_nums: dashmap::ReadOnlyView<SpawnNum, SmallVec<[Arc<dyn DatumStore>; 2]>>,
	gui: DevGui,
	populated: bool,
	// Q: FNV/aHash for maps using small key types?
}

impl Catalog {
	/// Each item in `basedata` is a combination of a real path and mount point.
	/// These will be mounted onto the VFS permanently but will need to be loaded
	/// in full along with everything else in a load request.
	///
	/// Panics if mounting the basedata fails for any reason.
	#[must_use]
	pub fn new(basedata: impl IntoIterator<Item = (PathBuf, VPathBuf)>) -> Self {
		let mut ret = Self {
			config: Config::default(),
			vfs: VirtualFs::default(),
			dobjs: DashMap::default().into_read_only(),
			nicknames: DashMap::default().into_read_only(),
			editor_nums: DashMap::default().into_read_only(),
			spawn_nums: DashMap::default().into_read_only(),
			gui: DevGui::default(),
			populated: false,
		};

		let mut load_order = vec![];

		for pair in basedata {
			ret.config.basedata.push(pair.0.clone());
			load_order.push(pair);
		}

		match ret.vfs.mount(MountRequest {
			load_order,
			tracker: None,
			basedata: true,
		}) {
			MountOutcome::Ok(errors) => {
				debug_assert!(
					errors.iter().all(|errs| errs.is_empty()),
					"unexpected non-fatal errors during basedata mount: {}",
					{
						let mut msg = String::default();

						for subvec in errors {
							msg.push_str("\r\n\r\n");

							for err in subvec {
								msg.push_str(&err.to_string());
							}
						}

						msg
					}
				);
			}
			MountOutcome::Errs(errs) => panic!("basedata mount failed: {}", {
				let mut msg = String::default();

				for subvec in errs {
					msg.push_str("\r\n\r\n");

					for err in subvec {
						msg.push_str(&err.to_string());
					}
				}

				msg
			}),
			MountOutcome::Cancelled => unreachable!(),
			MountOutcome::NoOp => {} // No basedata; this is valid.
		};

		ret
	}

	/// This is an end-to-end function that reads physical files, fills out the
	/// VFS, and then processes the files to decompose them into data objects.
	/// Much of the important things to know are in the documentation for
	/// [`LoadRequest`]. The range of possible errors is documented by
	/// [`MountError`] and [`PrepError`].
	///
	/// Notes:
	/// - This can only be called on a newly-created catalog or one which has had
	/// [`Self::clear`] called on it. Otherwise, a panic will occur.
	/// - Each load request is fulfilled in parallel using [`rayon`]'s global
	/// thread pool, but the caller thread itself gets blocked.
	pub fn load(&mut self, request: LoadRequest) -> LoadOutcome {
		assert!(
			!self.populated,
			"attempted to load a game to an already-populated `Catalog`."
		);

		if request.mount.load_order.is_empty() {
			return LoadOutcome::NoOp;
		}

		let prev_file_count = self.vfs.file_count();

		let mnt_errs = match self.vfs.mount(request.mount) {
			MountOutcome::Ok(errors) => errors,
			MountOutcome::Errs(errors) => return LoadOutcome::MountFail { errors },
			MountOutcome::Cancelled => return LoadOutcome::Cancelled,
			MountOutcome::NoOp => unreachable!(),
		};

		let prep_tracker = request
			.tracker
			.unwrap_or_else(|| Arc::new(SendTracker::default()));

		prep_tracker.set_target(self.vfs.file_count() - prev_file_count);

		let p_ctx = prep::Context::new(prep_tracker, self.vfs.mounts().len());

		// Note to reader: check `./prep.rs`.
		match self.prep(p_ctx) {
			Outcome::Ok(output) => {
				self.populated = true;

				LoadOutcome::Ok {
					mount: mnt_errs,
					prep: output,
				}
			}
			Outcome::Err(errors) => LoadOutcome::PrepFail { errors },
			Outcome::Cancelled => LoadOutcome::Cancelled,
			Outcome::None => unreachable!(),
		}
	}

	pub fn clear(&mut self) {
		self.vfs.truncate(self.config.basedata.len());

		let dobjs =
			std::mem::replace(&mut self.dobjs, DashMap::default().into_read_only()).into_inner();
		dobjs.clear();
		self.dobjs = dobjs.into_read_only();

		let nicknames = std::mem::replace(&mut self.nicknames, DashMap::default().into_read_only())
			.into_inner();
		nicknames.clear();
		self.nicknames = nicknames.into_read_only();

		let editor_nums =
			std::mem::replace(&mut self.editor_nums, DashMap::default().into_read_only())
				.into_inner();
		editor_nums.clear();
		self.editor_nums = editor_nums.into_read_only();

		let spawn_nums =
			std::mem::replace(&mut self.spawn_nums, DashMap::default().into_read_only())
				.into_inner();
		spawn_nums.clear();
		self.spawn_nums = spawn_nums.into_read_only();

		self.populated = false;
	}

	/// Note that `D` here is a filter on the type that comes out of the lookup,
	/// rather than an assertion that the datum under `id` is that type, so this
	/// returns an `Option` rather than a [`Result`].
	#[must_use]
	pub fn get<D: Datum>(&self, id: &str) -> Option<DataRef<D>> {
		let key = DatumKey::new::<D>(id);
		self.dobjs.get(&key).map(|arc| DataRef::new(self, arc))
	}

	/// Find an [actor] [`Blueprint`] by a 16-bit editor number.
	/// The last blueprint assigned the given number is what gets returned.
	///
	/// [actor]: crate::sim::actor
	#[must_use]
	pub fn bp_by_ednum(&self, num: EditorNum) -> Option<DataRef<Blueprint>> {
		let Some(stack) = self.editor_nums.get(&num) else {
			return None;
		};

		let arc = stack
			.last()
			.expect("catalog cleanup missed an empty ed-num stack");

		Some(DataRef::new(self, arc))
	}

	/// Find an [actor] [`Blueprint`] by a 16-bit spawn number.
	/// The last blueprint assigned the given number is what gets returned.
	///
	/// [actor]: crate::sim::actor
	#[must_use]
	pub fn bp_by_spawnnum(&self, num: SpawnNum) -> Option<DataRef<Blueprint>> {
		let Some(stack) = self.spawn_nums.get(&num) else {
			return None;
		};

		let arc = stack
			.last()
			.expect("catalog cleanup missed an empty spawn-num stack");

		Some(DataRef::new(self, arc))
	}

	#[must_use]
	pub fn last_by_nick<D: Datum>(&self, nick: &str) -> Option<DataRef<D>> {
		let key = DatumKey::new::<D>(nick);
		let Some(stack) = self.nicknames.get(&key) else {
			return None;
		};

		let arc = stack
			.last()
			.expect("catalog cleanup missed an empty nickname stack");

		Some(DataRef::new(self, arc))
	}

	#[must_use]
	pub fn first_by_nick<D: Datum>(&self, nick: &str) -> Option<DataRef<D>> {
		let key = DatumKey::new::<D>(nick);
		let Some(stack) = self.nicknames.get(&key) else {
			return None;
		};

		let arc = stack
			.first()
			.expect("catalog cleanup missed an empty nickname stack");

		Some(DataRef::new(self, arc))
	}

	#[must_use]
	pub fn vfs(&self) -> &VirtualFs {
		&self.vfs
	}

	#[must_use]
	pub fn vfs_mut(&mut self) -> &mut VirtualFs {
		&mut self.vfs
	}

	#[must_use]
	pub fn config_get(&self) -> ConfigGet {
		ConfigGet(self)
	}

	#[must_use]
	pub fn config_set(&mut self) -> ConfigSet {
		ConfigSet(self)
	}

	// TODO: Re-enable this helper when Bevy supports it.
	// See: https://github.com/bevyengine/bevy/issues/1031
	#[cfg(any())]
	fn window_icon_from_file(
		&self,
		path: impl AsRef<VPath>,
	) -> Result<winit::window::Icon, Box<dyn std::error::Error>> {
		let path = path.as_ref();

		let file = self
			.get_file(path)
			.ok_or_else(|| Box::new(VfsError::NotFound(path.to_path_buf())))?;

		let bytes = file.try_read_bytes()?;
		let icon = image::load_from_memory(bytes)?.into_rgba8();
		let (width, height) = icon.dimensions();

		winit::window::Icon::from_rgba(icon.into_raw(), width, height).map_err(|err| {
			let b: Box<dyn std::error::Error> = Box::new(err);
			b
		})
	}

	pub fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
		self.ui_impl(ctx, ui);
	}
}

/// A type alias for convenience and to reduce line noise.
pub type CatalogAM = Arc<Mutex<Catalog>>;
/// A type alias for convenience and to reduce line noise.
pub type CatalogAL = Arc<RwLock<Catalog>>;

// Mount, MountInfo ////////////////////////////////////////////////////////////

// Loading /////////////////////////////////////////////////////////////////////

/// Also make sure to read [`Catalog::load`] and [`MountRequest`].
#[derive(Debug)]
pub struct LoadRequest {
	pub mount: MountRequest,
	/// Only pass a `Some` if you need to report to the end user on the progress of
	/// a load operation (e.g. a loading screen) or provide the ability to cancel.
	pub tracker: Option<Arc<SendTracker>>,
	/// Affects:
	/// - Lithica optimization. None are applied if this is `false`.
	/// - Lithica dev-mode-only assertions.
	pub dev_mode: bool,
}

#[derive(Debug)]
#[must_use = "loading may return errors which should be handled"]
pub enum LoadOutcome {
	/// A [load request](LoadRequest) was given with a zero-length load order.
	NoOp,
	/// A cancel was requested externally.
	/// The catalog's state was left unchanged.
	Cancelled,
	/// One or more fatal errors prevented a successful VFS mount.
	MountFail {
		/// Every *new* mount gets a sub-vec, but that sub-vec may be empty.
		errors: Vec<Vec<MountError>>,
	},
	/// Mounting succeeeded, but one or more fatal errors
	/// prevented successful data preparation.
	PrepFail {
		/// Every *new* mount gets a sub-vec, but that sub-vec may be empty.
		errors: Vec<Vec<PrepError>>,
	},
	/// Loading was successful, but non-fatal errors or warnings may have arisen.
	Ok {
		/// Every *new* mount gets a sub-vec, but that sub-vec may be empty.
		mount: Vec<Vec<MountError>>,
		/// Every *new* mount gets a sub-vec, but that sub-vec may be empty.
		prep: Vec<Vec<PrepError>>,
	},
}

impl LoadOutcome {
	#[must_use]
	pub fn total_err_len(&self) -> usize {
		match self {
			LoadOutcome::NoOp | LoadOutcome::Cancelled => 0,
			LoadOutcome::MountFail { errors } => {
				errors.iter().fold(0, |acc, subvec| acc + subvec.len())
			}
			LoadOutcome::PrepFail { errors } => {
				errors.iter().fold(0, |acc, subvec| acc + subvec.len())
			}
			LoadOutcome::Ok { mount, prep } => {
				mount.iter().fold(0, |acc, subvec| acc + subvec.len())
					+ prep.iter().fold(0, |acc, subvec| acc + subvec.len())
			}
		}
	}

	/// All errors get sorted by the attached [`PathBuf`]s.
	///
	/// [`PathBuf`]: std::path::PathBuf
	pub fn sort_errors(&mut self) {
		match self {
			LoadOutcome::MountFail { errors } => {
				errors.par_iter_mut().for_each(|subvec| {
					subvec.sort_by(|err1, err2| err1.path.cmp(&err2.path));
				});
			}
			LoadOutcome::PrepFail { errors } => {
				errors.par_iter_mut().for_each(|subvec| {
					subvec.sort_by(|err1, err2| err1.path.cmp(&err2.path));
				});
			}
			LoadOutcome::Ok { mount, prep } => {
				mount.par_iter_mut().for_each(|subvec| {
					subvec.sort_by(|err1, err2| err1.path.cmp(&err2.path));
				});

				prep.par_iter_mut().for_each(|subvec| {
					subvec.sort_by(|err1, err2| err1.path.cmp(&err2.path));
				});
			}
			_ => {}
		}
	}
}
/*
/// Opens the catalog's VFS up to a [`bevy::asset::AssetServer`].
impl AssetIo for Catalog {
	fn load_path<'a>(&'a self, path: &'a VPath) -> BoxedFuture<'a, Result<Vec<u8>, AssetIoError>> {
		Box::pin(async move {
			match self.vfs.get(path) {
				Some(fref) => {
					if !fref.is_readable() {
						return Err(AssetIoError::Io(std::io::ErrorKind::Other.into()));
					}

					Ok(fref.read_bytes().to_owned())
				}
				None => Err(AssetIoError::NotFound(path.to_path_buf())),
			}
		})
	}

	fn read_directory(
		&self,
		path: &VPath,
	) -> Result<Box<dyn Iterator<Item = VPathBuf>>, AssetIoError> {
		match self.vfs.get(path) {
			Some(fref) => {
				if !fref.is_dir() {
					return Err(AssetIoError::Io(std::io::Error::from(
						std::io::ErrorKind::Other,
					)));
				}

				Ok(Box::new(
					fref.child_paths()
						.unwrap()
						.map(|path| path.to_path_buf())
						.collect::<Vec<_>>()
						.into_iter(),
				))
			}
			None => Err(AssetIoError::NotFound(path.to_path_buf())),
		}
	}

	fn get_metadata(&self, path: &VPath) -> Result<bevy::asset::Metadata, AssetIoError> {
		match self.vfs.get(path) {
			Some(fref) => {
				if fref.is_dir() {
					Ok(bevy::asset::Metadata::new(bevy::asset::FileType::Directory))
				} else {
					Ok(bevy::asset::Metadata::new(bevy::asset::FileType::File))
				}
			}
			None => Err(AssetIoError::NotFound(path.to_path_buf())),
		}
	}

	fn watch_path_for_changes(
		&self,
		_to_watch: &VPath,
		_to_reload: Option<VPathBuf>,
	) -> Result<(), AssetIoError> {
		unimplemented!()
	}

	fn watch_for_changes(&self, _: &ChangeWatcher) -> Result<(), AssetIoError> {
		unimplemented!()
	}
}
 */
/// Field `1` is a hash of the datum's ID string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct DatumKey(TypeId, u64);

impl DatumKey {
	#[must_use]
	fn new<D: Datum>(id: &str) -> Self {
		let mut hasher = FxHasher::default();
		id.hash(&mut hasher);
		Self(TypeId::of::<D>(), hasher.finish())
	}
}

/// Expands `~` on Unix and performs environment variable substitution.
/// Deliberately designed to mimic `NicePath` in
/// <https://github.com/ZDoom/gzdoom/blob/master/src/common/utility/cmdlib.cpp>.
#[must_use]
pub fn nice_path(path: impl AsRef<Path>) -> PathBuf {
	let p = path.as_ref();

	if p.is_empty() {
		return PathBuf::from(".");
	}

	#[cfg(not(target_os = "windows"))]
	if p == Path::new("/") {
		return PathBuf::from("/");
	}

	let mut string = p.to_string_lossy().to_string();

	#[cfg(not(target_os = "windows"))]
	{
		let home = home::home_dir().unwrap_or_default();
		let home = home.to_string_lossy();
		string = string.replace('~', &home);
	}

	let matches = lazy_regex!(r"\$[[:word:]]+").find_iter(&string);
	let mut ret = string.clone();

	for m in matches {
		match env::var(m.as_str()) {
			Ok(v) => {
				ret.replace_range(m.range(), &v);
			}
			Err(_) => {
				ret.replace_range(m.range(), "");
			}
		}
	}

	PathBuf::from(string)
}

/// Extracts a version string from what will almost always be a file stem,
/// using a search pattern based off the most common versioning conventions used
/// in ZDoom modding. If the returned option is `None`, the given string is unmodified.
#[must_use]
pub fn version_from_string(string: &mut String) -> Option<String> {
	match lazy_regex!(
		r"(?x)
		(?:[VR]|[\s\-_][VvRr]|[\s\-_\.])\d{1,}
		(?:[\._\-]\d{1,})*
		(?:[\._\-]\d{1,})*
		[A-Za-z]*[\._\-]*
		[A-Za-z0-9]*
		$"
	)
	.find(string)
	{
		Some(m) => {
			const TO_TRIM: [char; 3] = [' ', '_', '-'];
			let ret = m.as_str().trim_matches(&TO_TRIM[..]).to_string();
			string.replace_range(m.range(), "");
			Some(ret)
		}
		None => None,
	}
}

// (RAT) If you're reading this, congratulations! You've found something special.
// This module sub-tree is, historically speaking, the most tortured code in VileTech.
// The Git history doesn't even reflect half of the reworks the VFS has undergone.
