//! [`Ref`], [`FileRef`], and [`FolderRef`].

use std::sync::Arc;

use parking_lot::RwLockWriteGuard;

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

/// A reference to a [`VFile`].
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
	pub fn as_memory(&self) -> Option<&[u8]> {
		match std::ops::Deref::deref(&self.guard) {
			Reader::Memory(bytes) => Some(bytes.as_slice()),
			_ => None,
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
				let vfile = &self.vfs.files[fslot];

				Ref::File(FileRef {
					vfs: self.vfs,
					slot: fslot,
					vfile,
					guard: vfile.reader.write(),
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
