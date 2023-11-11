//! # VileTechFS
//!
//! VileTech's virtual file system; an abstraction over the operating system's
//! "physical" FS. Real files, directories, and various archives are all merged
//! into one tree so that reading from them is more convenient at all other levels
//! of the engine, without exposing any details of the user's underlying machine.

mod detail;
mod mount;
mod path;

#[cfg(test)]
mod test;

use std::{
	fs::File,
	io::{Read, Seek, SeekFrom},
	ops::Range,
	path::{Path, PathBuf},
	string::FromUtf8Error,
	sync::Arc,
};

use parking_lot::{RwLock, RwLockWriteGuard};
use rayon::prelude::*;
use slotmap::{new_key_type, HopSlotMap};
use util::SmallString;
use zip_structs::zip_error::ZipReadError;

pub use self::path::{VPath, VPathBuf};

#[derive(Debug)]
pub struct VirtualFs {
	pub(crate) root: FolderSlot,
	pub(crate) mounts: Vec<MountInfo>,
	pub(crate) files: HopSlotMap<FileSlot, VFile>,
	pub(crate) folders: HopSlotMap<FolderSlot, VFolder>,
}

impl VirtualFs {
	#[must_use]
	pub fn root(&self) -> FolderRef {
		FolderRef {
			vfs: self,
			slot: self.root,
			vfolder: &self.folders[self.root],
		}
	}

	pub fn mount(&mut self, real_path: &Path, mount_point: &VPath) -> Result<(), Error> {
		if mount_point.byte_len() == 0 {
			return Err(Error::MountPointEmpty);
		}

		if mount_point.as_str().contains(['/', '\\', '*']) {
			return Err(Error::MountPointInvalidChars);
		}

		if self.mounts.iter().any(|mntinfo| {
			mntinfo
				.mount_point
				.as_str()
				.eq_ignore_ascii_case(mount_point.as_str())
		}) {
			return Err(Error::MountPointDuplicate);
		}

		let canon = real_path.canonicalize().map_err(Error::Canonicalize)?;

		if canon.is_symlink() {
			return Err(Error::MountSymlink);
		}

		match mount::mount(self, &canon, mount_point.as_str()) {
			Ok(mntinfo) => {
				self.mounts.push(mntinfo);
				Ok(())
			}
			Err(err) => {
				let to_clean = match self.get(mount_point) {
					Some(Ref::File(iref)) => Some(Slot::File(iref.slot)),
					Some(Ref::Folder(oref)) => Some(Slot::Folder(oref.slot)),
					None => None,
				};

				match to_clean {
					Some(Slot::File(islot)) => {
						self.remove_file_by_slot(islot);
					}
					Some(Slot::Folder(oslot)) => {
						self.remove_folder_by_slot(oslot);
					}
					None => {}
				}

				Err(err)
			}
		}
	}

	#[must_use]
	pub fn exists(&self, vpath: &VPath) -> bool {
		self.get(vpath).is_some()
	}

	#[must_use]
	pub fn file_exists(&self, slot: FileSlot) -> bool {
		self.files.contains_key(slot)
	}

	#[must_use]
	pub fn folder_exists(&self, slot: FolderSlot) -> bool {
		self.folders.contains_key(slot)
	}

	/// Returns `true` if a file was removed.
	pub fn remove_file_by_slot(&mut self, slot: FileSlot) -> bool {
		let ret = self.files.remove(slot).is_some();

		if let Some(p) = self.mounts.iter().position(|mntinfo| mntinfo.root == slot) {
			self.mounts.remove(p);
		}

		ret
	}

	pub fn remove_folder_by_slot(&mut self, slot: FolderSlot) {
		assert_ne!(slot, self.root, "root folder cannot be removed");
		self.remove_folder_recur(slot);

		if let Some(p) = self.mounts.iter().position(|mntinfo| mntinfo.root == slot) {
			self.mounts.remove(p);
		}
	}

	fn remove_folder_recur(&mut self, oslot: FolderSlot) {
		let subfolders = std::mem::take(&mut self.folders[oslot].subfolders);

		for slot in subfolders {
			self.remove_folder_recur(slot);
			let removed = self.folders.remove(slot);
			debug_assert!(removed.is_some());
		}

		for islot in self.folders[oslot].files.iter().copied() {
			let removed = self.files.remove(islot);
			debug_assert!(removed.is_some());
		}
	}

	pub fn retain<F>(&mut self, mut predicate: F) -> Result<(), Error>
	where
		F: FnMut(&MountInfo) -> bool,
	{
		let mut to_unmount = vec![];

		self.mounts.retain(|mntinfo| {
			if predicate(mntinfo) {
				true
			} else {
				to_unmount.push(mntinfo.root);
				false
			}
		});

		for root in to_unmount {
			match root {
				Slot::File(islot) => {
					let removed = self.files.remove(islot);
					debug_assert!(removed.is_some());
				}
				Slot::Folder(oslot) => {
					self.remove_folder_recur(oslot);
				}
			}
		}

		Ok(())
	}

	pub fn get<'vfs: 'p, 'p>(&'vfs self, vpath: &'p VPath) -> Option<Ref<'vfs>> {
		self.lookup_recur(self.root, &self.folders[self.root], vpath.components())
	}

	fn lookup_recur<'vfs: 'p, 'p>(
		&'vfs self,
		slot: FolderSlot,
		folder: &'vfs VFolder,
		mut components: impl Iterator<Item = &'p VPath>,
	) -> Option<Ref<'vfs>> {
		let Some(pcomp) = components.next() else {
			return Some(Ref::Folder(FolderRef {
				vfs: self,
				slot,
				vfolder: folder,
			}));
		};

		if let Some((sfslot, subfold)) = folder.subfolders.iter().copied().find_map(|s| {
			let fold = &self.folders[s];

			fold.name
				.eq_ignore_ascii_case(pcomp.as_str())
				.then_some((s, fold))
		}) {
			return self.lookup_recur(sfslot, subfold, components);
		}

		let option = match folder.files.len() {
			// TODO: tweak the parallel search threshold to determine an optima.
			0..=4096 => folder.files.iter().copied().find_map(|slot| {
				let file = &self.files[slot];

				file.name
					.eq_ignore_ascii_case(pcomp.as_str())
					.then_some((slot, file))
			}),
			_ => folder.files.par_iter().copied().find_map_any(|slot| {
				let file = &self.files[slot];

				file.name
					.eq_ignore_ascii_case(pcomp.as_str())
					.then_some((slot, file))
			}),
		};

		let Some((slot, file)) = option else {
			return None;
		};

		let guard = file.reader.write();

		Some(Ref::File(FileRef {
			vfs: self,
			slot,
			vfile: file,
			guard,
		}))
	}

	/// Each virtual file backed by a physical file reads its slice into a buffer
	/// belonging exclusively to that virtual file.
	pub fn ingest_all(&mut self) {
		#[must_use]
		fn ingest(
			reader: &mut Reader,
			orig_span: Range<usize>,
			compression: Compression,
		) -> Option<Vec<u8>> {
			let result = match reader {
				Reader::File(fh) => Reader::read_from_file(fh, orig_span),
				Reader::Memory(_) => return None,
				Reader::_Super(_) => unimplemented!(),
			};

			result.and_then(|b| detail::decompress(b, compression)).ok()
		}

		let mut vfiles = self.files.values_mut();

		let Some(vfile0) = vfiles.next() else {
			return;
		};

		let mut guard = vfile0.reader.write_arc();
		let mut prev_arc = Arc::as_ptr(&vfile0.reader);

		if let Some(bytes) = ingest(&mut guard, vfile0.span(), vfile0.compression) {
			vfile0.span = 0..(bytes.len() as u32);
			vfile0.reader = Arc::new(RwLock::new(Reader::Memory(bytes)));
			vfile0.compression = Compression::None;
		}

		for vfile in vfiles {
			// If the new lock is the same as the previous lock,
			// don't waste time on another re-open.
			let arc_ptr = Arc::as_ptr(&vfile.reader);

			if !std::ptr::eq(arc_ptr, prev_arc) {
				guard = vfile.reader.write_arc();
			};

			prev_arc = arc_ptr;

			if let Some(bytes) = ingest(&mut guard, vfile.span(), vfile.compression) {
				vfile.span = 0..(bytes.len() as u32);
				vfile.reader = Arc::new(RwLock::new(Reader::Memory(bytes)));
				vfile.compression = Compression::None;
			}
		}
	}

	#[must_use]
	pub fn mounts(&self) -> &[MountInfo] {
		&self.mounts
	}

	/// Computes in `O(1)` time.
	#[must_use]
	pub fn file_count(&self) -> usize {
		self.files.len()
	}

	/// Computes in `O(1)` time.
	#[must_use]
	pub fn folder_count(&self) -> usize {
		self.folders.len()
	}

	/// Shorthand for adding [`Self::file_count`] to [`Self::folder_count`].
	#[must_use]
	pub fn total_count(&self) -> usize {
		self.file_count() + self.folder_count()
	}

	pub fn files(&self) -> impl Iterator<Item = FileRef> {
		self.files.iter().map(|(k, v)| FileRef {
			vfs: self,
			slot: k,
			vfile: v,
			guard: v.reader.write(),
		})
	}

	pub fn folders(&self) -> impl Iterator<Item = FolderRef> {
		self.folders.iter().map(|(k, v)| FolderRef {
			vfs: self,
			slot: k,
			vfolder: v,
		})
	}

	#[must_use]
	pub fn file_is_mount(&self, slot: FileSlot) -> bool {
		self.mounts.iter().any(|mntinfo| mntinfo.root == slot)
	}

	#[must_use]
	pub fn folder_is_mount(&self, slot: FolderSlot) -> bool {
		self.mounts.iter().any(|mntinfo| mntinfo.root == slot)
	}

	pub fn clear(&mut self) {
		let root = self.folders.remove(self.root).unwrap();
		self.folders.clear();
		self.files.clear();
		self.root = self.folders.insert(root);
	}
}

impl Default for VirtualFs {
	fn default() -> Self {
		let mut folders = HopSlotMap::default();

		let root = folders.insert(VFolder {
			name: SmallString::from("/"),
			parent: None,
			files: vec![],
			subfolders: vec![],
		});

		Self {
			root,
			mounts: vec![],
			files: HopSlotMap::default(),
			folders,
		}
	}
}

/// Metadata for a file subtree registered using [`VirtualFs::mount`].
#[derive(Debug)]
pub struct MountInfo {
	pub real_path: PathBuf,
	pub mount_point: VPathBuf,
	pub root: Slot,
	pub format: MountFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountFormat {
	Uncompressed,
	Directory,
	Wad,
	Zip,
}

#[derive(Debug)]
pub enum Ref<'vfs> {
	File(FileRef<'vfs>),
	Folder(FolderRef<'vfs>),
}

impl<'vfs> Ref<'vfs> {
	#[must_use]
	pub fn vfs(&self) -> &'vfs VirtualFs {
		match self {
			Self::File(iref) => iref.vfs,
			Self::Folder(oref) => oref.vfs,
		}
	}

	#[must_use]
	pub fn name(&self) -> &str {
		match self {
			Self::File(iref) => iref.name(),
			Self::Folder(oref) => oref.name(),
		}
	}

	pub fn into_file(self) -> Option<FileRef<'vfs>> {
		match self {
			Self::File(iref) => Some(iref),
			Self::Folder(_) => None,
		}
	}

	#[must_use]
	pub fn into_folder(self) -> Option<FolderRef<'vfs>> {
		match self {
			Self::Folder(oref) => Some(oref),
			Self::File(_) => None,
		}
	}

	/// Only returns `None` if this is a reference to the root folder.
	#[must_use]
	pub fn parent(&self) -> Option<FolderRef> {
		match self {
			Self::File(iref) => Some(FolderRef {
				vfs: iref.vfs,
				slot: iref.vfile.parent,
				vfolder: &iref.vfs.folders[iref.vfile.parent],
			}),
			Self::Folder(oref) => oref.vfolder.parent.map(|slot| FolderRef {
				vfs: oref.vfs,
				slot,
				vfolder: &oref.vfs.folders[slot],
			}),
		}
	}

	#[must_use]
	pub fn path(&self) -> VPathBuf {
		match self {
			Self::File(iref) => iref.path(),
			Self::Folder(oref) => oref.path(),
		}
	}

	#[must_use]
	pub fn is_readable(&self) -> bool {
		match self {
			Self::File(iref) => !iref.is_empty(),
			Self::Folder(_) => false,
		}
	}

	pub fn read(&mut self) -> Result<Vec<u8>, Error> {
		let Self::File(FileRef {
			vfile,
			ref mut guard,
			..
		}) = self
		else {
			return Err(Error::VFolderRead);
		};

		guard.read(vfile.span(), vfile.compression)
	}

	pub fn read_text(&mut self) -> Result<String, Error> {
		let bytes = self.read()?;
		String::from_utf8(bytes).map_err(Error::Utf8)
	}
}

#[derive(Debug)]
pub struct FileRef<'vfs> {
	pub(crate) vfs: &'vfs VirtualFs,
	pub(crate) slot: FileSlot,
	pub(crate) vfile: &'vfs VFile,
	pub(crate) guard: RwLockWriteGuard<'vfs, Reader>,
}

impl FileRef<'_> {
	#[must_use]
	pub fn name(&self) -> &str {
		self.name.as_str()
	}

	#[must_use]
	pub fn slot(&self) -> FileSlot {
		self.slot
	}

	#[must_use]
	pub fn path(&self) -> VPathBuf {
		let mut buf = String::from('/');
		buf.push_str(self.name());
		detail::path_append(self.vfs, &mut buf, self.parent);
		VPathBuf::new(buf)
	}

	/// Prefer this to taking a new reference out of the [`VirtualFs`] since it
	/// will re-use the same [`RwLock`] guard if the same reader underlies this
	/// ref's file and the file behind `slot`.
	pub fn change_ref_by_slot(self, slot: FileSlot) -> Result<Self, Self> {
		match self.vfs.files.get(slot) {
			Some(vfile) => {
				let guard = if Arc::ptr_eq(&self.vfile.reader, &vfile.reader) {
					self.guard
				} else {
					vfile.reader.write()
				};

				Ok(Self {
					vfs: self.vfs,
					slot,
					vfile,
					guard,
				})
			}
			None => Err(self),
		}
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.vfile.span.is_empty()
	}
}

impl std::ops::Deref for FileRef<'_> {
	type Target = VFile;

	fn deref(&self) -> &Self::Target {
		self.vfile
	}
}

impl PartialEq for FileRef<'_> {
	fn eq(&self, other: &Self) -> bool {
		std::ptr::eq(self.vfs, other.vfs) && std::ptr::eq(self.vfile, other.vfile)
	}
}

impl Eq for FileRef<'_> {}

impl<'vfs> From<FileRef<'vfs>> for Ref<'vfs> {
	fn from(value: FileRef<'vfs>) -> Self {
		Self::File(value)
	}
}

#[derive(Debug)]
pub struct FolderRef<'vfs> {
	pub(crate) vfs: &'vfs VirtualFs,
	pub(crate) slot: FolderSlot,
	pub(crate) vfolder: &'vfs VFolder,
}

impl<'vfs> FolderRef<'vfs> {
	#[must_use]
	pub fn name(&self) -> &str {
		self.name.as_str()
	}

	#[must_use]
	pub fn slot(&self) -> FolderSlot {
		self.slot
	}

	#[must_use]
	pub fn path(&self) -> VPathBuf {
		let mut buf = String::new();

		buf.push_str(self.name());

		if let Some(p) = self.parent {
			detail::path_append(self.vfs, &mut buf, p);
		}

		VPathBuf::new(buf)
	}

	pub fn subfolders(&self) -> impl Iterator<Item = FolderRef<'vfs>> {
		self.vfolder
			.subfolders
			.iter()
			.copied()
			.map(|sfslot| FolderRef {
				vfs: self.vfs,
				slot: sfslot,
				vfolder: &self.vfs.folders[sfslot],
			})
	}

	pub fn files(&self) -> impl Iterator<Item = FileRef<'vfs>> {
		self.vfolder.files.iter().copied().map(|fslot| {
			let vfile = &self.vfs.files[fslot];

			FileRef {
				vfs: self.vfs,
				slot: fslot,
				vfile,
				guard: vfile.reader.write(),
			}
		})
	}
}

impl std::ops::Deref for FolderRef<'_> {
	type Target = VFolder;

	fn deref(&self) -> &Self::Target {
		self.vfolder
	}
}

impl PartialEq for FolderRef<'_> {
	fn eq(&self, other: &Self) -> bool {
		std::ptr::eq(self.vfs, other.vfs) && std::ptr::eq(self.vfolder, other.vfolder)
	}
}

impl Eq for FolderRef<'_> {}

impl<'vfs> From<FolderRef<'vfs>> for Ref<'vfs> {
	fn from(value: FolderRef<'vfs>) -> Self {
		Self::Folder(value)
	}
}

/// Short for "virtual file".
/// May represent a real file or an entry in an archive.
#[derive(Debug)]
pub struct VFile {
	pub(crate) name: SmallString,
	pub(crate) parent: FolderSlot,
	pub(crate) reader: Arc<RwLock<Reader>>,
	pub(crate) span: Range<u32>,
	pub(crate) compression: Compression,
}

impl VFile {
	#[must_use]
	pub fn name(&self) -> &VPath {
		VPath::new(self.name.as_str())
	}

	/// How many bytes are represented by this virtual file?
	/// Beware that this is pre-compression, if any.
	#[must_use]
	pub fn size(&self) -> usize {
		self.span().len()
	}

	#[must_use]
	fn span(&self) -> Range<usize> {
		(self.span.start as usize)..(self.span.end as usize)
	}
}

#[derive(Debug)]
pub struct VFolder {
	pub(crate) name: SmallString,
	/// Only `None` for the root.
	pub(crate) parent: Option<FolderSlot>,
	pub(crate) files: Vec<FileSlot>,
	pub(crate) subfolders: Vec<FolderSlot>,
}

new_key_type! {
	/// A unique identifier for a virtual file. This is always valid for the
	/// VFS which emitted it, regardless of what mounts/unmounts/insertions/removals
	/// take place.
	///
	/// Using this in a VFS other than the one that emitted it will yield
	/// unexpected results but is safe.
	///
	/// Also see [`Slot`].
	pub struct FileSlot;
	/// A unique identifier for a virtual folder. This is always valid for the
	/// VFS which emitted it, regardless of what mounts/unmounts/insertions/removals
	/// take place.
	///
	/// Using this in a VFS other than the one that emitted it will yield
	/// unexpected results but is safe.
	///
	/// Also see [`Slot`].
	pub struct FolderSlot;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slot {
	File(FileSlot),
	Folder(FolderSlot),
}

impl PartialEq<FileSlot> for Slot {
	fn eq(&self, other: &FileSlot) -> bool {
		match self {
			Self::File(islot) => *islot == *other,
			Self::Folder(_) => false,
		}
	}
}

impl PartialEq<FolderSlot> for Slot {
	fn eq(&self, other: &FolderSlot) -> bool {
		match self {
			Self::Folder(oslot) => *oslot == *other,
			Self::File(_) => false,
		}
	}
}

impl From<FileSlot> for Slot {
	fn from(value: FileSlot) -> Self {
		Self::File(value)
	}
}

impl From<FolderSlot> for Slot {
	fn from(value: FolderSlot) -> Self {
		Self::Folder(value)
	}
}

#[derive(Debug)]
pub enum Error {
	Canonicalize(std::io::Error),
	Decompress(std::io::Error),
	DirRead(std::io::Error),
	EmptyRead,
	FileHandleClone(std::io::Error),
	FileOpen(std::io::Error),
	FileRead(std::io::Error),
	Metadata(std::io::Error),
	MountPointDuplicate,
	MountPointEmpty,
	MountPointInvalidChars,
	MountSymlink,
	NotFound,
	Seek(std::io::Error),
	Utf8(FromUtf8Error),
	VFolderRead,
	Wad(wadload::Error),
	Zip(ZipReadError),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Canonicalize(err) => write!(f, "failed to canonicalize a mount path: {err}"),
			Self::Decompress(err) => write!(f, "failed to decompress an archive entry: {err}"),
			Self::DirRead(err) => write!(
				f,
				"failed to get the contents of a physical directory: {err}"
			),
			Self::EmptyRead => write!(f, "attempted to read the byte content of an empty entry"),
			Self::FileHandleClone(err) => {
				write!(f, "failed to clone a physical file handle: {err}")
			}
			Self::FileOpen(err) => write!(f, "failed to open a physical file handle: {err}"),
			Self::FileRead(err) => write!(f, "failed to read a physical file: {err}"),
			Self::Metadata(err) => write!(f, "failed to retrieve physical file metadata: {err}"),
			Self::MountPointDuplicate => {
				write!(f, "attempt a mount using an already-present mount point")
			}
			Self::MountPointEmpty => write!(f, "given mount point is empty"),
			Self::MountPointInvalidChars => write!(f, "given mount point has invalid characters"),
			Self::NotFound => write!(f, "no entry found by the given path"),
			Self::Seek(err) => write!(f, "failed to seek a physical file handle: {err}"),
			Self::MountSymlink => write!(f, "attempted to mount a symbolic link"),
			Self::Utf8(err) => write!(f, "failed to read UTF-8 text from a virtual file: {err}"),
			Self::VFolderRead => write!(f, "attempted to read byte content of a virtual folder"),
			Self::Wad(err) => write!(f, "WAD read error: {err}"),
			Self::Zip(err) => write!(f, "zip archive read error: {err}"),
		}
	}
}

#[derive(Debug)]
pub(crate) enum Reader {
	/// e.g. lump in a WAD, or entry in a zip archive.
	File(File),
	Memory(Vec<u8>),
	/// e.g. entry in a zip archive nested within another zip archive.
	_Super(ReaderLayer),
}

impl Reader {
	fn read(&mut self, span: Range<usize>, compression: Compression) -> Result<Vec<u8>, Error> {
		let bytes = match self {
			Self::File(ref mut fh) => Self::read_from_file(fh, span)?,
			Self::Memory(bytes) => bytes[span].to_vec(),
			Self::_Super(layer) => {
				let mut guard = layer.parent.write();
				guard.read(layer.span.clone(), layer.compression)?
			}
		};

		detail::decompress(bytes, compression)
	}

	fn read_from_file(fh: &mut File, span: Range<usize>) -> Result<Vec<u8>, Error> {
		fh.seek(SeekFrom::Start(span.start as u64))
			.map_err(Error::Seek)?;
		let mut bytes = vec![0; span.len()];
		fh.read_exact(&mut bytes).map_err(Error::FileRead)?;
		Ok(bytes)
	}
}

#[derive(Debug)]
pub(crate) struct ReaderLayer {
	pub(crate) parent: Arc<RwLock<Reader>>,
	pub(crate) span: Range<usize>,
	pub(crate) compression: Compression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Compression {
	/// Always the case for WAD lumps.
	None,
	Bzip2,
	Deflate,
	Lzma,
	Xz,
	Zstd,
}
