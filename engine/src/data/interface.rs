//! Assorted parts of the public API, in a separate file for cleanliness.

use std::{
	path::Path,
	sync::{
		atomic::{self, AtomicBool, AtomicUsize},
		Arc,
	},
};

use slotmap::SlotMap;

use crate::VPath;

use super::{
	detail::{AssetSlotKey, VfsKey},
	Asset, Catalog, File, FileKind,
};

// FileRef /////////////////////////////////////////////////////////////////////

/// The primary interface for quick introspection into the virtual file system;
/// provides read access to one entry and the catalog itself. Prefer these over
/// working directly with references to [`File`]s, since this can trace
/// inter-file relationships.
#[derive(Debug, Clone, Copy)]
pub struct FileRef<'cat> {
	pub(super) catalog: &'cat Catalog,
	pub(super) file: &'cat File,
}

impl<'cat> FileRef<'cat> {
	/// The catalog this reference came from.
	#[must_use]
	pub fn catalog(&self) -> &Catalog {
		self.catalog
	}

	/// This only returns `None` if this file is the root directory.
	#[must_use]
	pub fn parent(&self) -> Option<&File> {
		if let Some(parent) = self.file.parent_path() {
			Some(
				self.catalog
					.files
					.get(&VfsKey::new(parent))
					.expect("A VFS entry has a dangling parent path."),
			)
		} else {
			None
		}
	}

	/// This only returns `None` if this file is the root directory.
	#[must_use]
	pub fn parent_ref(&'cat self) -> Option<Self> {
		self.parent().map(|file| Self {
			catalog: self.catalog,
			file,
		})
	}

	/// Non-recursive; only gets immediate children. If this file is not a directory,
	/// or is an empty directory, the returned iterator will yield no items.
	pub fn children(&self) -> impl Iterator<Item = &File> {
		let closure = |key: &VfsKey| {
			self.catalog
				.files
				.get(key)
				.expect("A VFS directory has a dangling child key.")
		};

		match &self.file.kind {
			FileKind::Directory(children) => children.iter().map(closure),
			_ => [].iter().map(closure),
		}
	}

	/// Non-recursive; only gets immediate children. If this file is not a directory,
	/// or is an empty directory, the returned iterator will yield no items.
	pub fn child_refs(&'cat self) -> impl Iterator<Item = FileRef<'cat>> + '_ {
		self.children().map(|file| Self {
			catalog: self.catalog,
			file,
		})
	}

	/// Returns 0 if this is a leaf node or an empty directory.
	#[must_use]
	pub fn child_count(&self) -> usize {
		match &self.kind {
			FileKind::Directory(children) => children.len(),
			_ => 0,
		}
	}
}

impl std::ops::Deref for FileRef<'_> {
	type Target = File;

	fn deref(&self) -> &Self::Target {
		self.file
	}
}

impl PartialEq for FileRef<'_> {
	fn eq(&self, other: &Self) -> bool {
		std::ptr::eq(self.catalog, other.catalog) && std::ptr::eq(self.file, other.file)
	}
}

impl Eq for FileRef<'_> {}

// Mount, MountInfo ////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Mount {
	pub(super) assets: SlotMap<AssetSlotKey, Arc<dyn Asset>>,
	pub(super) info: MountInfo,
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
	/// Normally, the scripts can define manifest items used to direct post-processing,
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
}

/// Informs the rules used for post-processing assets from a mount.
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

// Configuration ///////////////////////////////////////////////////////////////

/// Configuration methods are kept in a wrapper around a [`Catalog`] reference
/// to prevent bloat in the interface of the catalog itself.
#[derive(Debug)]
#[repr(transparent)]
pub struct ConfigGet<'cat>(pub(super) &'cat Catalog);

impl ConfigGet<'_> {
	/// The limit on the size of a virtual binary file. Irrelevant to asset management.
	/// A mount can be any size the physical filesystem can handle, but any children
	/// over this size are rejected with a logged warning.
	///
	/// The returned value is in bytes, and defaults to [`limits::DEFAULT_BIN_FILE_SIZE`].
	#[must_use]
	pub fn bin_size_limit(&self) -> usize {
		self.0.config.bin_size_limit
	}

	/// The limit on the size of a virtual text file. Irrelevant to asset management.
	/// A mount can be any size the physical filesystem can handle, but any children
	/// over this size are rejected with a logged warning.
	///
	/// The returned value is in bytes, and defaults to [`limits::DEFAULT_TEXT_FILE_SIZE`].
	#[must_use]
	pub fn text_size_limit(&self) -> usize {
		self.0.config.text_size_limit
	}
}

/// Configuration methods are kept in a wrapper around a [`Catalog`] reference
/// to prevent bloat in the interface of the catalog itself.
#[derive(Debug)]
#[repr(transparent)]
pub struct ConfigSet<'cat>(pub(super) &'cat mut Catalog);

impl ConfigSet<'_> {
	/// The limit on the size of a virtual binary file. Irrelevant to asset management.
	/// A mount can be any size the physical filesystem can handle, but any children
	/// over this size are rejected with a logged warning.
	///
	/// The parameter is in bytes, and gets clamped between 0 and
	/// [`limits::MAX_BIN_FILE_SIZE`]. The default is [`limits::DEFAULT_BIN_FILE_SIZE`].
	pub fn bin_size_limit(self, limit: usize) -> Self {
		self.0.config.bin_size_limit = limit.clamp(0, limits::MAX_BIN_FILE_SIZE);
		self
	}

	/// The limit on the size of a virtual text file. Irrelevant to asset management.
	/// A mount can be any size the physical filesystem can handle, but any children
	/// over this size are rejected with a logged warning.
	///
	/// The parameter is in bytes, and gets clamped between 0 and
	/// [`limits::MAX_TEXT_FILE_SIZE`]. The default is [`limits::DEFAULT_TEXT_FILE_SIZE`].
	pub fn text_size_limit(self, limit: usize) -> Self {
		self.0.config.text_size_limit = limit.clamp(0, limits::MAX_TEXT_FILE_SIZE);
		self
	}
}

pub mod limits {
	/// 1024 B * 1024 kB * 512 MB = 536870912 bytes.
	pub const DEFAULT_BIN_FILE_SIZE: usize = 1024 * 1024 * 512;
	/// 1024 B * 1024 kB * 64 MB = 67108864 bytes.
	pub const DEFAULT_TEXT_FILE_SIZE: usize = 1024 * 1024 * 64;
	/// 1024 B * 1024 kB * 1024 MB * 2 GB = 2147483648 bytes.
	pub const MAX_BIN_FILE_SIZE: usize = 1024 * 1024 * 1024 * 2;
	/// 1024 B * 1024 kB * 128 MB = 134217728 bytes.
	pub const MAX_TEXT_FILE_SIZE: usize = 1024 * 1024 * 128;

	// (RAT) If you guessed that the default text file size limit could
	// be much lower if not for the UDMF TEXTMAP format, then you're correct.
	// Ar Luminae's MAP01 TEXTMAP is 43.69 MB.
}

// Loading /////////////////////////////////////////////////////////////////////

/// Also make sure to read [`Catalog::load`].
#[derive(Debug)]
pub struct LoadRequest<RP, MP>
where
	RP: AsRef<Path>,
	MP: AsRef<VPath>,
{
	/// In any given tuple, element `::0` should be a real path and `::1` should
	/// be the mount point. `mymount` and `/mymount` both put the mount on the root.
	/// An empty path and `/` are both invalid mount points, but one can mount
	/// `/mymount` and then `mymount/myothermount`.
	///
	/// If this is an empty slice, any mount operation becomes a no-op, and
	/// an empty array of results is returned.
	pub paths: Vec<(RP, MP)>,
	/// Only pass a `Some` if you need to, for instance, display a loading screen,
	/// or otherwise report to the end user on the progress of a mount operation.
	pub tracker: Option<Arc<LoadTracker>>,
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
	/// been post-processed thus far.
	pproc_progress: AtomicUsize,
	/// The number of files added to the VFS during the mount phase.
	pproc_target: AtomicUsize,
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
	pub fn pproc_progress(&self) -> usize {
		self.pproc_progress.load(atomic::Ordering::SeqCst)
	}

	#[must_use]
	pub fn pproc_target(&self) -> usize {
		self.pproc_target.load(atomic::Ordering::SeqCst)
	}

	/// 0.0 means just started; 1.0 means done.
	#[must_use]
	pub fn pproc_progress_percent(&self) -> f64 {
		let prog = self.pproc_progress.load(atomic::Ordering::SeqCst);
		let tgt = self.pproc_target.load(atomic::Ordering::SeqCst);

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
	pub fn pproc_done(&self) -> bool {
		self.pproc_progress.load(atomic::Ordering::SeqCst)
			>= self.pproc_target.load(atomic::Ordering::SeqCst)
	}

	pub fn cancel(&self) {
		self.cancelled.store(true, atomic::Ordering::SeqCst);
	}

	pub(super) fn set_mount_target(&self, target: usize) {
		debug_assert_eq!(self.mount_target.load(atomic::Ordering::Relaxed), 0);

		self.mount_target.store(target, atomic::Ordering::SeqCst);
	}

	pub(super) fn set_pproc_target(&self, target: usize) {
		debug_assert_eq!(self.pproc_target.load(atomic::Ordering::Relaxed), 0);

		self.pproc_target.store(target, atomic::Ordering::SeqCst);
	}

	pub(super) fn add_mount_progress(&self, amount: usize) {
		self.mount_progress
			.fetch_add(amount, atomic::Ordering::SeqCst);
	}

	pub(super) fn add_pproc_progress(&self, amount: usize) {
		self.pproc_progress
			.fetch_add(amount, atomic::Ordering::SeqCst);
	}

	/// Temporary.
	pub(super) fn finish_pproc(&self) {
		self.pproc_progress.store(
			self.pproc_target.load(atomic::Ordering::SeqCst),
			atomic::Ordering::SeqCst,
		)
	}

	#[must_use]
	pub(super) fn is_cancelled(&self) -> bool {
		self.cancelled.load(atomic::Ordering::SeqCst)
	}
}
