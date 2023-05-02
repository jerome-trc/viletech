//! Management of files, audio, graphics, levels, text, localization, and so on.

pub mod asset;
mod config;
mod detail;
mod error;
mod ext;
mod gui;
mod mount;
mod prep;
#[cfg(test)]
mod test;
pub mod vfs;

use std::{
	path::Path,
	sync::{
		atomic::{self, AtomicBool, AtomicUsize},
		Arc,
	},
};

use bevy_egui::egui;
use dashmap::DashMap;
use parking_lot::{Mutex, RwLock};
use slotmap::SlotMap;
use smallvec::SmallVec;

use crate::{vzs, EditorNum, SpawnNum, VPath};

use self::{
	detail::{AssetKey, AssetSlotKey, MountSlotKey},
	vfs::{FileRef, VirtualFs},
};

pub use self::{asset::*, config::*, error::*, ext::*};

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
/// Any given unit of data or [`Asset`] is stored behind an [`Arc`], allowing
/// other parts of the engine to take out high-speed [`Handle`]s to something and
/// safely access it without passing through locks or casts.
///
/// A footnote on semantics: it is impossible to mount a file that's nested within
/// an archive. If `mymod.zip` contains `myothermod.vpk7`, there's no way to
/// register `myothermod` as a mount in the official sense. It's just a part of
/// `mymod`'s file tree.
#[derive(Debug, Default)]
pub struct Catalog {
	pub(self) config: Config,
	pub(self) _vzscript: vzs::Project,
	pub(self) vfs: VirtualFs,
	/// The first element is always the engine's base data (ID `viletech`),
	/// but every following element is user-specified, including their order.
	pub(self) mounts: SlotMap<MountSlotKey, Mount>,
	/// In each value:
	/// - Field `0` is a key into `Self::mounts`.
	/// - Field `1` is a key into [`Mount::assets`].
	pub(self) assets: DashMap<AssetKey, (MountSlotKey, AssetSlotKey)>,
	/// Asset lookup table without namespacing. Thus, requesting `MAP01` returns
	/// the last element in the array behind that key, as doom.exe would if
	/// loading multiple WADs with similarly-named entries.
	pub(self) nicknames: DashMap<AssetKey, SmallVec<[(MountSlotKey, AssetSlotKey); 2]>>,
	/// See the key type's documentation for background details.
	/// Keyed assets are always of type [`Blueprint`].
	pub(self) editor_nums: DashMap<EditorNum, SmallVec<[(MountSlotKey, AssetSlotKey); 2]>>,
	/// See the key type's documentation for background details.
	/// Keyed assets are always of type [`Blueprint`].
	pub(self) spawn_nums: DashMap<SpawnNum, SmallVec<[(MountSlotKey, AssetSlotKey); 2]>>,
	// Q: FNV/aHash for maps using small key types?
}

impl Catalog {
	/// This is an end-to-end function that reads physical files, fills out the
	/// VFS, and then processes the files to decompose them into assets.
	/// Much of the important things to know are in the documentation for
	/// [`LoadRequest`]. The range of possible errors is documented by
	/// [`MountError`] and [`PrepError`].
	///
	/// Notes:
	/// - The order of pre-existing VFS entries and mounts is unchanged upon success.
	/// - This function is partially atomic. If mounting fails, the catalog's
	/// state is left entirely unchanged from before calling this.
	/// If asset preparation fails, the VFS state is not restored to before the
	/// call as a form of memoization, allowing future prep attempts to skip most
	/// mounting work (to allow faster mod development cycles).
	/// - Each load request is fulfilled in parallel using [`rayon`]'s global
	/// thread pool, but the caller thread itself gets blocked.
	pub fn load<RP, MP>(&mut self, request: LoadRequest<RP, MP>) -> LoadOutcome
	where
		RP: AsRef<Path>,
		MP: AsRef<VPath>,
	{
		if request.load_order.is_empty() {
			return LoadOutcome::NoOp;
		}

		let mnt_ctx = mount::Context::new(request.tracker, &request.load_order);

		// Note to reader: check `./mount.rs`.
		let mnt_output = match self.mount(request.load_order, mnt_ctx) {
			detail::Outcome::Ok(output) => output,
			detail::Outcome::Err(errors) => return LoadOutcome::MountFail { errors },
			detail::Outcome::Cancelled => return LoadOutcome::Cancelled,
			detail::Outcome::None => unreachable!(),
		};

		let p_ctx = prep::Context::new(mnt_output.tracker, mnt_output.new_mounts);

		// Note to reader: check `./prep.rs`.
		match self.prep(p_ctx) {
			detail::Outcome::Ok(output) => LoadOutcome::Ok {
				mount: mnt_output.errors,
				prep: output.errors,
			},
			detail::Outcome::Err(errors) => LoadOutcome::PrepFail { errors },
			detail::Outcome::Cancelled => LoadOutcome::Cancelled,
			detail::Outcome::None => unreachable!(),
		}
	}

	pub fn unload_all(&mut self) {
		unimplemented!("New VFS and load scheme are pending.")
	}

	/// Note that `A` here is a filter on the type that comes out of the lookup,
	/// rather than an assertion that the asset under `id` is that type, so this
	/// returns an `Option` rather than a [`Result`].
	#[must_use]
	pub fn get_asset<A: Asset>(&self, id: &str) -> Option<&Arc<A>> {
		let key = AssetKey::new::<A>(id);

		if let Some(kvp) = self.assets.get(&key) {
			self.mounts[kvp.0].assets[kvp.1].as_any().downcast_ref()
		} else {
			None
		}
	}

	/// Find an [`Actor`] [`Blueprint`] by a 16-bit editor number.
	/// The last blueprint assigned the given number is what gets returned.
	///
	/// [`Actor`]: crate::sim::actor::Actor
	#[must_use]
	pub fn bp_by_ednum(&self, num: EditorNum) -> Option<&Blueprint> {
		let Some(kvp) = self.editor_nums.get(&num) else { return None; };
		let stack = kvp.value();

		let (msk, ask) = stack
			.last()
			.expect("Catalog cleanup missed an empty ed-num stack.");

		let asset = &self.mounts[*msk].assets[*ask];
		Some(asset.as_any().downcast_ref::<Blueprint>().unwrap())
	}

	/// Find an [`Actor`] [`Blueprint`] by a 16-bit spawn number.
	/// The last blueprint assigned the given number is what gets returned.
	///
	/// [`Actor`]: crate::sim::actor::Actor
	#[must_use]
	pub fn bp_by_spawnnum(&self, num: SpawnNum) -> Option<&Blueprint> {
		let Some(kvp) = self.spawn_nums.get(&num) else { return None; };
		let stack = kvp.value();

		let (msk, ask) = stack
			.last()
			.expect("Catalog cleanup missed an empty spawn-num stack.");

		let asset = &self.mounts[*msk].assets[*ask];
		Some(asset.as_any().downcast_ref::<Blueprint>().unwrap())
	}

	#[must_use]
	pub fn last_asset_by_nick<A: Asset>(&self, nick: &str) -> Option<&A> {
		let key = AssetKey::new::<A>(nick);
		let Some(kvp) = self.nicknames.get(&key) else { return None; };
		let stack = kvp.value();

		let (msk, ask) = stack
			.last()
			.expect("Catalog cleanup missed an empty nickname stack.");

		let asset = &self.mounts[*msk].assets[*ask];
		Some(asset.as_any().downcast_ref::<A>().unwrap())
	}

	#[must_use]
	pub fn first_asset_by_nick<A: Asset>(&self, nick: &str) -> Option<&A> {
		let key = AssetKey::new::<A>(nick);
		let Some(kvp) = self.nicknames.get(&key) else { return None; };
		let stack = kvp.value();

		let (msk, ask) = stack
			.first()
			.expect("Catalog cleanup missed an empty nickname stack.");

		let asset = &self.mounts[*msk].assets[*ask];
		Some(asset.as_any().downcast_ref::<A>().unwrap())
	}

	#[must_use]
	pub fn vfs(&self) -> &VirtualFs {
		&self.vfs
	}

	pub fn mounts(&self) -> impl Iterator<Item = &Mount> {
		self.mounts.values()
	}

	#[must_use]
	pub fn mounts_len(&self) -> usize {
		self.mounts.len()
	}

	#[must_use]
	pub fn config_get(&self) -> ConfigGet {
		ConfigGet(self)
	}

	#[must_use]
	pub fn config_set(&mut self) -> ConfigSet {
		ConfigSet(self)
	}

	pub fn ui_assets(&self, ctx: &egui::Context, ui: &mut egui::Ui) {
		self.ui_assets_impl(ctx, ui);
	}
}

/// A type alias for convenience and to reduce line noise.
pub type CatalogAM = Arc<Mutex<Catalog>>;
/// A type alias for convenience and to reduce line noise.
pub type CatalogAL = Arc<RwLock<Catalog>>;

// Mount, MountInfo ////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Mount {
	pub(self) assets: SlotMap<AssetSlotKey, Arc<dyn Asset>>,
	pub(self) info: MountInfo,
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
	/// The base of the package's VZScript include tree.
	///
	/// - For VileTech packages, this is specified by a `meta.toml` file.
	/// - For ZDoom and Eternity packages, the script root is the first
	/// "lump" with the file stem `VZSCRIPT` in the package's global namespace.
	/// - For WADs, the script root is the first `VZSCRIPT` "lump" found.
	///
	/// Normally, the scripts can define manifest items used to direct loading,
	/// but if there is no script root or manifests, ZDoom loading rules are used.
	///
	/// A package can only specify a file owned by it as a script root, so this
	/// is always relative. `viletech.vpk3`'s script root, for example, is `main.vzs`.
	pub(super) script_root: Option<Box<VPath>>,
	// Q:
	// - Dependency specification?
	// - Incompatibility specification?
	// - Ordering specification?
	// - Forced specifications, or just strongly-worded warnings? Multiple levels?
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
}

/// Informs the rules used for preparing assets from a mount.
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
	/// prevented successful asset preparation.
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
	pub fn num_errs(&self) -> usize {
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
	/// been processed into prepared assets thus far.
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

	/// Temporary.
	pub(super) fn finish_prep(&self) {
		self.prep_progress.store(
			self.prep_target.load(atomic::Ordering::SeqCst),
			atomic::Ordering::SeqCst,
		)
	}

	#[must_use]
	pub(super) fn is_cancelled(&self) -> bool {
		self.cancelled.load(atomic::Ordering::SeqCst)
	}
}

// (RAT) If you're reading this, congratulations! You've found something special.
// This module sub-tree is, historically speaking, the most tortured code in VileTech.
// The Git history doesn't even reflect half of the reworks the VFS has undergone.
