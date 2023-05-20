//! Management of files, audio, graphics, levels, text, localization, and so on.

mod config;
mod detail;
pub mod dobj;
mod error;
mod extras;
mod mount;
mod prep;
#[cfg(test)]
mod test;
pub mod vfs;

use std::{
	path::{Path, PathBuf},
	sync::{
		atomic::{self, AtomicBool, AtomicUsize},
		Arc,
	},
};

use bevy::{
	asset::{AssetIo, AssetIoError},
	utils::BoxedFuture,
};
use bevy_egui::egui;
use dashmap::DashMap;
use parking_lot::{Mutex, RwLock};
use rayon::prelude::*;
use slotmap::SlotMap;
use smallvec::SmallVec;

use crate::{vzs, EditorNum, SpawnNum, VPath, VPathBuf};

use self::{
	detail::{DatumKey, DatumSlotKey, DeveloperGui},
	dobj::{Blueprint, DataRef, Datum, DatumStore},
	vfs::{FileRef, VirtualFs},
};

pub use self::{config::*, error::*, extras::*};

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
/// [pointers]: obj::Rcd
#[derive(Debug)]
pub struct Catalog {
	pub(self) config: Config,
	/// When the catalog is initialized, this is empty.
	pub(self) vzscript: vzs::Project,
	/// See [`Self::new`]; mounts given as `basedata` through that function are
	/// always present here.
	pub(self) vfs: VirtualFs,
	/// When the catalog is initialized, this is empty.
	pub(self) mounts: Vec<Mount>,
	/// In each value:
	/// - Field `0` is an index into `Self::mounts`.
	/// - Field `1` is a key into [`Mount::objs`].
	pub(self) objs: DashMap<DatumKey, (usize, DatumSlotKey)>,
	/// Datum lookup table without namespacing. Thus, requesting `MAP01` returns
	/// the last element in the array behind that key, as doom.exe would if
	/// loading multiple WADs with similarly-named entries. Also contains names
	/// assigned via [`SNDINFO`](https://zdoom.org/wiki/SNDINFO).
	pub(self) nicknames: DashMap<DatumKey, SmallVec<[(usize, DatumSlotKey); 2]>>,
	/// See the key type's documentation for background details.
	/// Keyed objects are always of type [`Blueprint`].
	pub(self) editor_nums: DashMap<EditorNum, SmallVec<[(usize, DatumSlotKey); 2]>>,
	/// See the key type's documentation for background details.
	/// Keyed objects are always of type [`Blueprint`].
	pub(self) spawn_nums: DashMap<SpawnNum, SmallVec<[(usize, DatumSlotKey); 2]>>,
	pub(self) gui: DeveloperGui,
	pub(self) populated: bool,
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
			vzscript: vzs::Project::default(),
			vfs: VirtualFs::default(),
			mounts: vec![],
			objs: DashMap::default(),
			nicknames: DashMap::default(),
			editor_nums: DashMap::default(),
			spawn_nums: DashMap::default(),
			gui: DeveloperGui::default(),
			populated: false,
		};

		let mut load_order = vec![];

		for pair in basedata {
			load_order.push(pair);
		}

		let mnt_ctx = mount::Context::new(None, load_order.len(), true);

		match ret.mount(&load_order, mnt_ctx) {
			detail::Outcome::Ok(_) => {}
			detail::Outcome::Err(errs) => panic!("Basedata mount failed: {}", {
				let mut msg = String::default();

				for subvec in errs {
					msg.push_str("\r\n\r\n");

					for err in subvec {
						msg.push_str(&err.to_string());
					}
				}

				msg
			}),
			detail::Outcome::Cancelled | detail::Outcome::None => unreachable!(),
		};

		for (rp, _) in load_order {
			ret.config.basedata.insert(rp);
		}

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
	pub fn load<RP, MP>(&mut self, request: LoadRequest<RP, MP>) -> LoadOutcome
	where
		RP: AsRef<Path>,
		MP: AsRef<VPath>,
	{
		assert!(
			!self.populated,
			"Attempted to load a game to an already-populated `Catalog`."
		);

		if request.load_order.is_empty() {
			return LoadOutcome::NoOp;
		}

		let mnt_ctx = mount::Context::new(request.tracker, request.load_order.len(), false);

		// Note to reader: check `./mount.rs`.
		let mnt_output = match self.mount(&request.load_order, mnt_ctx) {
			detail::Outcome::Ok(output) => output,
			detail::Outcome::Err(errors) => return LoadOutcome::MountFail { errors },
			detail::Outcome::Cancelled => return LoadOutcome::Cancelled,
			detail::Outcome::None => unreachable!(),
		};

		let p_ctx = prep::Context::new(mnt_output.tracker, self.mounts.len());

		// Note to reader: check `./prep.rs`.
		match self.prep(p_ctx) {
			detail::Outcome::Ok(output) => {
				self.populated = true;

				LoadOutcome::Ok {
					mount: mnt_output.errors,
					prep: output.errors,
				}
			}
			detail::Outcome::Err(errors) => LoadOutcome::PrepFail { errors },
			detail::Outcome::Cancelled => LoadOutcome::Cancelled,
			detail::Outcome::None => unreachable!(),
		}
	}

	pub fn clear(&mut self) {
		for mount in &self.mounts {
			if self.config.basedata.contains(mount.info.real_path()) {
				continue;
			}

			self.vfs.remove_recursive(mount.info.virtual_path());
		}

		self.mounts.clear();
		self.vzscript.clear();
		self.objs.clear();
		self.nicknames.clear();
		self.editor_nums.clear();
		self.spawn_nums.clear();

		self.populated = false;
	}

	/// Note that `D` here is a filter on the type that comes out of the lookup,
	/// rather than an assertion that the datum under `id` is that type, so this
	/// returns an `Option` rather than a [`Result`].
	#[must_use]
	pub fn get<D: Datum>(&self, id: &str) -> Option<DataRef<D>> {
		let key = DatumKey::new::<D>(id);

		if let Some(kvp) = self.objs.get(&key) {
			Some(DataRef::new(self, &self.mounts[kvp.0].objs[kvp.1]))
		} else {
			None
		}
	}

	/// Find an [actor] [`Blueprint`] by a 16-bit editor number.
	/// The last blueprint assigned the given number is what gets returned.
	///
	/// [actor]: crate::sim::actor
	#[must_use]
	pub fn bp_by_ednum(&self, num: EditorNum) -> Option<DataRef<Blueprint>> {
		let Some(kvp) = self.editor_nums.get(&num) else { return None; };
		let stack = kvp.value();

		let (mnt_i, dsk) = stack
			.last()
			.expect("Catalog cleanup missed an empty ed-num stack.");

		Some(DataRef::new(self, &self.mounts[*mnt_i].objs[*dsk]))
	}

	/// Find an [actor] [`Blueprint`] by a 16-bit spawn number.
	/// The last blueprint assigned the given number is what gets returned.
	///
	/// [actor]: crate::sim::actor
	#[must_use]
	pub fn bp_by_spawnnum(&self, num: SpawnNum) -> Option<DataRef<Blueprint>> {
		let Some(kvp) = self.spawn_nums.get(&num) else { return None; };
		let stack = kvp.value();

		let (mnt_i, dsk) = stack
			.last()
			.expect("Catalog cleanup missed an empty spawn-num stack.");

		Some(DataRef::new(self, &self.mounts[*mnt_i].objs[*dsk]))
	}

	#[must_use]
	pub fn last_by_nick<D: Datum>(&self, nick: &str) -> Option<DataRef<D>> {
		let key = DatumKey::new::<D>(nick);
		let Some(kvp) = self.nicknames.get(&key) else { return None; };
		let stack = kvp.value();

		let (mnt_i, dsk) = stack
			.last()
			.expect("Catalog cleanup missed an empty nickname stack.");

		Some(DataRef::new(self, &self.mounts[*mnt_i].objs[*dsk]))
	}

	#[must_use]
	pub fn first_by_nick<D: Datum>(&self, nick: &str) -> Option<DataRef<D>> {
		let key = DatumKey::new::<D>(nick);
		let Some(kvp) = self.nicknames.get(&key) else { return None; };
		let stack = kvp.value();

		let (mnt_i, dsk) = stack
			.first()
			.expect("Catalog cleanup missed an empty nickname stack.");

		Some(DataRef::new(self, &self.mounts[*mnt_i].objs[*dsk]))
	}

	#[must_use]
	pub fn last_paletteset(&self) -> Option<&PaletteSet> {
		self.mounts
			.iter()
			.rev()
			.find_map(|mnt| mnt.extras.palset.as_deref())
	}

	#[must_use]
	pub fn vfs(&self) -> &VirtualFs {
		&self.vfs
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

#[derive(Debug)]
pub struct Mount {
	pub(self) objs: SlotMap<DatumSlotKey, Arc<dyn DatumStore>>,
	pub(self) info: MountInfo,
	pub(self) extras: WadExtras,
}

#[derive(Debug)]
pub struct MountInfo {
	/// Specified by `meta.toml` if one exists.
	/// Otherwise, this comes from the file stem of the mount point.
	pub(super) id: String,
	pub(super) format: MountFormat,
	pub(super) kind: MountKind,
	/// Always canonicalized, but may not necessarily be valid UTF-8.
	pub(super) real_path: Box<Path>,
	pub(super) virtual_path: Box<VPath>,
	/// Comes from `meta.toml`, so most mounts will lack this.
	pub(super) meta: Option<Box<MountMeta>>,
	pub(super) vzscript: Option<VzsManifest>,
	// Q:
	// - Dependency specification?
	// - Incompatibility specification?
	// - Ordering specification?
	// - Forced specifications, or just strongly-worded warnings? Multiple levels?
}

#[derive(Debug)]
pub struct VzsManifest {
	/// The base of the package's VZScript include tree.
	///
	/// This is irrelevant to WADs, which can only use VZS through lumps named
	/// `VZSCRIPT`.
	///
	/// Normally, the scripts can define manifest items used to direct loading,
	/// but if there is no script root or manifests, ZDoom loading rules are used.
	///
	/// A package can only specify a directory owned by it as a script root, so this
	/// is always relative. `viletech.vpk3`'s script root, for example, is `scripts`.
	pub(super) root_dir: VPathBuf,
	pub(super) namespace: Option<String>,
	pub(super) version: vzs::Version,
}

/// A WAD file may come with its own color palettes, color-remapping table,
/// and/or ENDOOM lump. These are practically never included multiple times,
/// or included by non-WADs, and most PWADs also have none.
#[derive(Debug, Default)]
pub struct WadExtras {
	pub(self) colormap: Option<Box<ColorMap>>,
	pub(self) palset: Option<Box<PaletteSet>>,
	pub(self) endoom: Option<Box<EnDoom>>,
}

impl VzsManifest {
	#[must_use]
	pub fn namespace(&self) -> Option<&str> {
		self.namespace.as_deref()
	}

	#[must_use]
	pub fn version(&self) -> vzs::Version {
		self.version
	}
}

impl MountInfo {
	#[must_use]
	pub fn id(&self) -> &str {
		&self.id
	}

	#[must_use]
	pub fn format(&self) -> MountFormat {
		self.format
	}

	/// The real file/directory this mount represents.
	/// Always canonicalized, but may not necessarily be valid UTF-8.
	#[must_use]
	pub fn real_path(&self) -> &Path {
		&self.real_path
	}

	/// Also known as the "mount point". Corresponds to a VFS node.
	#[must_use]
	pub fn virtual_path(&self) -> &VPath {
		&self.virtual_path
	}

	#[must_use]
	pub fn metadata(&self) -> Option<&MountMeta> {
		self.meta.as_deref()
	}

	#[must_use]
	pub fn is_basedata(&self) -> bool {
		self.id == crate::BASEDATA_ID
	}

	#[must_use]
	pub fn vzscript(&self) -> &Option<VzsManifest> {
		&self.vzscript
	}
}

/// Informs the rules used for preparing data from a mount.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountKind {
	/// If the mount's own root has an immediate child text file named `meta.toml`
	/// (ASCII-case-ignored), that indicates that the mount is a VileTech package.
	VileTech,
	/// If mounting an archive with:
	/// - no immediate text file child named `meta.toml`, and
	/// - the extension `.pk3`, `.ipk3`, `.pk7`, or `.ipk7`,
	/// then this is what gets resolved. If it's a directory instead of an archive,
	/// the heuristic used is if there's an immediate child text file with a file
	/// stem belonging to a ZDoom-exclusive lump.
	ZDoom,
	/// If mounting an archive with:
	/// - no immediate text file child named `meta.toml`, and
	/// - the extension `.pke`,
	/// then this is what gets resolved. If it's a directory instead of an archive,
	/// the heuristic used is if there's an immediate child text file with the
	/// file stem `edfroot` or `emapinfo` (ASCII-case-ignored).
	Eternity,
	/// Deduced from [`MountFormat`], which is itself deduced from the file header.
	Wad,
	/// Fallback if the mount resolved to none of the other kinds.
	/// Usually used if mounting a single non-archive file.
	Misc,
}

/// Primarily serves to specify the type of compression used, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountFormat {
	PlainFile,
	Directory,
	Wad,
	Zip,
	// TODO: Support LZMA, XZ, GRP, PAK, RFF, SSI
}

#[derive(Debug)]
pub struct MountMeta {
	pub(super) version: Option<String>,
	/// Specified by `meta.toml` if one exists.
	/// Human-readable, presented to users in the frontend.
	pub(super) name: Option<String>,
	/// Specified by `meta.toml` if one exists.
	/// Human-readable, presented to users in the frontend.
	pub(super) description: Option<String>,
	/// Specified by `meta.toml` if one exists.
	/// Human-readable, presented to users in the frontend.
	pub(super) authors: Vec<String>,
	/// Specified by `meta.toml` if one exists.
	/// Human-readable, presented to users in the frontend.
	pub(super) copyright: Option<String>,
	/// Specified by `meta.toml` if one exists.
	/// Allow a package to link to its forum post/homepage/Discord server/etc.
	pub(super) links: Vec<String>,
}

impl MountMeta {
	#[must_use]
	pub fn name(&self) -> Option<&str> {
		match &self.name {
			Some(s) => Some(s),
			None => None,
		}
	}

	#[must_use]
	pub fn version(&self) -> Option<&str> {
		match &self.version {
			Some(s) => Some(s),
			None => None,
		}
	}

	#[must_use]
	pub fn description(&self) -> Option<&str> {
		match &self.description {
			Some(s) => Some(s),
			None => None,
		}
	}

	#[must_use]
	pub fn authors(&self) -> &[String] {
		self.authors.as_ref()
	}

	#[must_use]
	pub fn copyright_info(&self) -> Option<&str> {
		match &self.copyright {
			Some(s) => Some(s),
			None => None,
		}
	}

	/// User-specified URLS to a forum post/homepage/Discord server/et cetera.
	#[must_use]
	pub fn public_links(&self) -> &[String] {
		&self.links
	}
}

// Loading /////////////////////////////////////////////////////////////////////

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

/// Also make sure to read [`Catalog::load`].
#[derive(Debug)]
pub struct LoadRequest<RP, MP>
where
	RP: AsRef<Path>,
	MP: AsRef<VPath>,
{
	/// This can be empty; it makes the load operation into a no-op.
	///
	/// With regards to mount points (`MP`):
	/// - `mymount` and `/mymount` both put the mount on the root.
	/// - An empty path and `/` are both invalid mount points.
	pub load_order: Vec<(RP, MP)>,
	/// Only pass a `Some` if you need to report to the end user on the progress of
	/// a load operation (e.g. a loading screen) or provide the ability to cancel.
	pub tracker: Option<Arc<LoadTracker>>,
	/// Affects:
	/// - VZScript optimization. None are applied if this is `false`.
	pub dev_mode: bool,
}

/// Wrap in an [`Arc`] and use to check how far along a load operation is.
#[derive(Debug, Default)]
pub struct LoadTracker {
	/// Set to `true` to make the load thread return to be joined as soon as possible.
	/// The catalog's state will be the same as before calling [`Catalog::load`].
	cancelled: AtomicBool,
	/// The number of VFS mounts performed (successfully or not) thus far.
	mount_progress: AtomicUsize,
	/// The number of VFS mounts requested by the user.
	mount_target: AtomicUsize,
	/// The number of files added to the VFS during the mount phase which have
	/// been processed into prepared data thus far.
	prep_progress: AtomicUsize,
	/// The number of files added to the VFS during the mount phase.
	prep_target: AtomicUsize,
}

impl LoadTracker {
	#[must_use]
	pub fn mount_progress(&self) -> usize {
		self.mount_progress.load(atomic::Ordering::SeqCst)
	}

	#[must_use]
	pub fn mount_target(&self) -> usize {
		self.mount_target.load(atomic::Ordering::SeqCst)
	}

	/// 0.0 means just started; 1.0 means done.
	#[must_use]
	pub fn mount_progress_percent(&self) -> f64 {
		let prog = self.mount_progress.load(atomic::Ordering::SeqCst);
		let tgt = self.mount_target.load(atomic::Ordering::SeqCst);

		if tgt == 0 {
			return 0.0;
		}

		prog as f64 / tgt as f64
	}

	#[must_use]
	pub fn prep_progress(&self) -> usize {
		self.prep_progress.load(atomic::Ordering::SeqCst)
	}

	#[must_use]
	pub fn prep_target(&self) -> usize {
		self.prep_target.load(atomic::Ordering::SeqCst)
	}

	/// 0.0 means just started; 1.0 means done.
	#[must_use]
	pub fn prep_progress_percent(&self) -> f64 {
		let prog = self.prep_progress.load(atomic::Ordering::SeqCst);
		let tgt = self.prep_target.load(atomic::Ordering::SeqCst);

		if tgt == 0 {
			return 0.0;
		}

		prog as f64 / tgt as f64
	}

	#[must_use]
	pub fn mount_done(&self) -> bool {
		self.mount_progress.load(atomic::Ordering::SeqCst)
			>= self.mount_target.load(atomic::Ordering::SeqCst)
	}

	#[must_use]
	pub fn prep_done(&self) -> bool {
		self.prep_progress.load(atomic::Ordering::SeqCst)
			>= self.prep_target.load(atomic::Ordering::SeqCst)
	}

	pub fn cancel(&self) {
		self.cancelled.store(true, atomic::Ordering::SeqCst);
	}

	pub(super) fn set_mount_target(&self, target: usize) {
		debug_assert_eq!(self.mount_target.load(atomic::Ordering::Relaxed), 0);

		self.mount_target.store(target, atomic::Ordering::SeqCst);
	}

	pub(super) fn add_to_prep_target(&self, files: usize) {
		self.prep_target.fetch_add(files, atomic::Ordering::SeqCst);
	}

	pub(super) fn add_mount_progress(&self, amount: usize) {
		self.mount_progress
			.fetch_add(amount, atomic::Ordering::SeqCst);
	}

	pub(super) fn add_prep_progress(&self, amount: usize) {
		self.prep_progress
			.fetch_add(amount, atomic::Ordering::SeqCst);
	}

	pub(self) fn finish_mount(&self) {
		self.mount_progress.store(
			self.prep_target.load(atomic::Ordering::SeqCst),
			atomic::Ordering::SeqCst,
		)
	}

	pub(self) fn finish_prep(&self) {
		self.prep_progress.store(
			self.prep_target.load(atomic::Ordering::SeqCst),
			atomic::Ordering::SeqCst,
		)
	}

	#[must_use]
	pub(self) fn is_cancelled(&self) -> bool {
		self.cancelled.load(atomic::Ordering::SeqCst)
	}
}

/// Newtype for trait impl coherence; pass to [`bevy::asset::AssetServer::new`].
pub struct CatalogAssetIo(pub CatalogAL);

/// Opens the catalog's VFS up to a [`bevy::asset::AssetServer`].
impl AssetIo for CatalogAssetIo {
	fn load_path<'a>(&'a self, path: &'a VPath) -> BoxedFuture<'a, Result<Vec<u8>, AssetIoError>> {
		Box::pin(async move {
			let catalog = self.0.read();

			match catalog.vfs.get(path) {
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
		let catalog = self.0.read();

		match catalog.vfs.get(path) {
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
		let catalog = self.0.read();

		match catalog.vfs.get(path) {
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

	fn watch_for_changes(&self) -> Result<(), AssetIoError> {
		unimplemented!()
	}
}

// (RAT) If you're reading this, congratulations! You've found something special.
// This module sub-tree is, historically speaking, the most tortured code in VileTech.
// The Git history doesn't even reflect half of the reworks the VFS has undergone.
