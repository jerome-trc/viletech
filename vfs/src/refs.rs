//! [`Ref`], [`FileRef`], and [`FolderRef`].

use std::{borrow::Cow, sync::Arc};

use parking_lot::MutexGuard;

use crate::{
	detail::{self, Reader},
	Error, FileSlot, FolderSlot, VFile, VFolder, VPathBuf, VirtualFs,
};

/// A reference to a [`VFile`] or [`VFolder`].
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
}

/// A reference to a [`VFile`].
#[derive(Debug)]
pub struct FileRef<'vfs> {
	pub(crate) vfs: &'vfs VirtualFs,
	pub(crate) slot: FileSlot,
	pub(crate) vfile: &'vfs VFile,
}

impl<'vfs> FileRef<'vfs> {
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

	#[must_use]
	pub fn next_sibling(&self) -> Option<FileRef<'vfs>> {
		self.sibling(1)
	}

	#[must_use]
	pub fn prev_sibling(&self) -> Option<FileRef<'vfs>> {
		self.sibling(-1)
	}

	#[must_use]
	fn sibling(&self, offset: isize) -> Option<FileRef<'vfs>> {
		let parent = &self.vfs.folders[self.parent];
		let ix = parent.files.get_index_of(&self.slot).unwrap() as isize;

		parent
			.files
			.get_index((ix + offset) as usize)
			.copied()
			.map(|islot| FileRef {
				vfs: self.vfs,
				slot: islot,
				vfile: &self.vfs.files[islot],
			})
	}

	#[must_use]
	pub fn lock(&self) -> Guard {
		Guard {
			vfs: self.vfs,
			vfile: self.vfile,
			inner: self.vfile.reader.lock(),
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

/// A reference to a [`VFolder`].
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
		self.vfolder.files.iter().copied().map(|fslot| FileRef {
			vfs: self.vfs,
			slot: fslot,
			vfile: &self.vfs.files[fslot],
		})
	}

	/// Yields [`Ref::Folder`]s to all subfolders
	/// and then [`Ref::File`]s to all child files.
	pub fn children(&self) -> impl Iterator<Item = Ref<'vfs>> {
		self.vfolder
			.subfolders
			.iter()
			.copied()
			.map(|sfslot| {
				Ref::Folder(FolderRef {
					vfs: self.vfs,
					slot: sfslot,
					vfolder: &self.vfs.folders[sfslot],
				})
			})
			.chain(self.vfolder.files.iter().copied().map(|fslot| {
				Ref::File(FileRef {
					vfs: self.vfs,
					slot: fslot,
					vfile: &self.vfs.files[fslot],
				})
			}))
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

/// Acquired from [`VFile::read`] to gain access to the content it represents.
///
/// Beware that this wraps a mutex guard,
/// so the same caveats about possible deadlocks apply.
#[derive(Debug)]
pub struct Guard<'vfs> {
	vfs: &'vfs VirtualFs,
	vfile: &'vfs VFile,
	inner: MutexGuard<'vfs, Reader>,
}

impl Guard<'_> {
	pub fn read(&mut self) -> Result<Cow<[u8]>, Error> {
		self.inner.read(self.vfile.span(), self.vfile.compression)
	}

	/// Acquires the lock on a different file.
	///
	/// Prefer this to taking out a new [`FileRef`] and calling [`FileRef::lock`]
	/// if possible, since it will re-use the same guard if both virtual files are
	/// backed by the same reader.
	///
	/// If `slot` does not correspond to an existing virtual file, the `Err` variant
	/// will return `self` so it can be used for something else.
	pub fn transfer_by_slot(self, slot: FileSlot) -> Result<Self, Self> {
		let Some(f) = self.vfs.files.get(slot) else {
			return Err(self);
		};

		let guard = if Arc::ptr_eq(&f.reader, &self.vfile.reader) {
			self.inner
		} else {
			f.reader.lock()
		};

		Ok(Self {
			vfs: self.vfs,
			vfile: f,
			inner: guard,
		})
	}
}
