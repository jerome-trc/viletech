//! # VileTechFS
//!
//! VileTech's virtual file system; an abstraction over the operating system's
//! "physical" FS. Real files, directories, and various archives are all merged
//! into one tree so that reading from them is more convenient at all other levels
//! of the engine, without exposing any details of the user's underlying machine.

mod path;

#[cfg(test)]
mod test;

use std::{
	io::Read,
	path::{Path, PathBuf},
};

use arc_swap::{ArcSwapAny, Guard};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use triomphe::{Arc, ThinArc};
use util::rstring::RString;

pub use path::*;

#[derive(Debug)]
pub struct VirtualFs {
	root: Folder,
	/// Runs parallel to `root.subfolders`.
	mounts: Vec<MountInfo>,
	/// Every currently-mounted file will be somewhere in this map.
	filespaced: FxHashMap<SpacedPath, Vec<Arc<VFile>>>,
	/// Every currently-mounted file will be somewhere in this map.
	short_names: FxHashMap<ShortPath, Vec<Arc<VFile>>>,
}

impl VirtualFs {
	#[must_use]
	pub fn root(&self) -> Ref {
		Ref::Folder {
			vfs: self,
			folder: &self.root,
		}
	}

	#[must_use]
	pub fn lookup(&self, path: &str) -> Option<Ref> {
		let vpath = VPath::new(path);
		let mut components = vpath.components();

		if vpath.is_absolute() {
			let _ = components.next().unwrap();
		}

		let mut stack = SmallVec::<[&Folder; 8]>::default();
		stack.push(&self.root);

		for comp in components {
			let fold = *stack.last().unwrap();

			let (subfold, subfile) = if fold.subfolders.is_empty() {
				// Likely a WAD.
				(None, fold.files.par_iter().find_any(|vf| vf.path.name().is_some_and(|n| n == comp)))
			} else if fold.files.is_empty() {
				(
					fold.subfolders.par_iter().find_any(|vf| vf.path.name().is_some_and(|n| n == comp)),
					None,
				)
			} else {
				rayon::join(
					|| {
						fold.subfolders
							.iter()
							.find(|s| s.path.name().is_some_and(|n| n == comp))
					},
					|| {
						fold.files
							.iter()
							.find(|s| s.path.name().is_some_and(|n| n == comp))
					},
				)
			};

			if let Some(vf) = subfile {
				return Some(Ref::File {
					vfs: self,
					file: vf,
					content: vf.content.as_ref().map(|arcswap| arcswap.load()),
				});
			} else if let Some(fold) = subfold {
				stack.push(fold);
			} else {
				return None;
			}
		}

		Some(Ref::Folder {
			vfs: self,
			folder: stack.pop().unwrap(),
		})
	}

	#[must_use]
	pub fn mounts(&self) -> &[MountInfo] {
		&self.mounts
	}

	/// Leaves only the root.
	pub fn clear(&mut self) {
		self.root.subfolders.clear();
		self.filespaced.clear();
		self.short_names.clear();
	}
}

impl Default for VirtualFs {
	fn default() -> Self {
		Self {
			root: Folder {
				path: FolderPath(RString::new("/")),
				subfolders: vec![],
				files: vec![],
				child_count: 0,
			},
			mounts: vec![],
			filespaced: FxHashMap::default(),
			short_names: FxHashMap::default(),
		}
	}
}

#[derive(Debug)]
pub struct MountInfo {
	pub(crate) id: String,
	pub(crate) format: MountFormat,
	pub(crate) real_path: PathBuf,
}

impl MountInfo {
	/// Specified by `meta.toml` if one exists.
	/// Otherwise, this comes from the file stem of the mount point.
	#[must_use]
	pub fn id(&self) -> &str {
		&self.id
	}

	#[must_use]
	pub fn format(&self) -> MountFormat {
		self.format
	}

	/// Always canonicalized, but may not necessarily be valid UTF-8.
	#[must_use]
	pub fn real_path(&self) -> &Path {
		&self.real_path
	}
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
pub struct Folder {
	pub path: FolderPath,
	pub subfolders: Vec<Folder>,
	/// The original ordering of the ingested physical files is preserved.
	pub files: Vec<Arc<VFile>>,
	/// This is recursive, and does not include sub-folders.
	pub child_count: usize,
}

#[derive(Debug)]
pub struct VFile {
	/// Always in lowercase.
	pub path: VPathBuf,
	/// This is `None` if the file is initialized to have 0 byte content
	/// (e.g. marker lumps in WAD archives), but not if the file was consumed.
	pub content: Option<ArcSwapAny<Content>>,
}

impl VFile {
	#[must_use]
	pub fn new<R: Read>(path: impl AsRef<str>, mut reader: R, len: usize) -> std::io::Result<Self> {
		let header = ContentHeader {
			id: ContentId::Binary, // Not known yet; this is just a default.
			consumed: false,
		};

		struct ExactSizeIter<I: Iterator<Item = u8>>(I, usize);

		impl<I: Iterator<Item = u8>> Iterator for ExactSizeIter<I> {
			type Item = u8;

			fn next(&mut self) -> Option<Self::Item> {
				self.0.next()
			}

			fn size_hint(&self) -> (usize, Option<usize>) {
				(self.1, Some(self.1))
			}
		}

		impl<I: Iterator<Item = u8>> ExactSizeIterator for ExactSizeIter<I> {}

		let iter = ExactSizeIter(std::iter::repeat(0).take(len), len);
		let content = Content::from_header_and_iter(header, iter);

		content.with_arc(|arc| {
			let ptr = arc.as_ptr().cast_mut();
			// SAFETY: This ARC is currently held exclusively.
			let slice = unsafe { &mut (*ptr).slice };
			reader.read_exact(slice)
		})?;

		Ok(Self {
			path: VPathBuf(RString::new(path)),
			content: Some(ArcSwapAny::new(content)),
		})
	}

	#[must_use]
	pub fn new_empty(path: impl AsRef<str>) -> Self {
		Self {
			path: VPathBuf(RString::new(path)),
			content: None,
		}
	}

	#[must_use]
	pub fn path(&self) -> &VPath {
		std::borrow::Borrow::borrow(&self.path)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentId {
	/// Used for virtual files whose content has been consumed.
	None,
	/// Non-UTF-8; the meaning of the content of the file is unknown.
	Binary,
	/// UTF-8; the meaning of the content of the file is unknown.
	Text,
}

impl ContentId {
	#[must_use]
	pub fn is_binary(self) -> bool {
		match self {
			ContentId::Binary => true,
			ContentId::None | ContentId::Text => false,
		}
	}

	#[must_use]
	pub fn is_text(self) -> bool {
		match self {
			ContentId::None | ContentId::Binary => false,
			ContentId::Text => true,
		}
	}
}

#[derive(Debug)]
pub struct ContentHeader {
	id: ContentId,
	consumed: bool,
}

impl ContentHeader {
	#[must_use]
	pub(crate) fn is_text(&self) -> bool {
		self.id.is_text() && !self.consumed
	}
}

/// See <https://zdoom-docs.github.io/staging/Api/Base/Wads/WadNamespace.html>.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileSpace {
	AcsLib,
	Colormaps,
	Flats,
	Global,
	Graphics,
	HiRes,
	Music,
	Textures,
	Patches,
	Sounds,
	Sprites,
	Voices,
	Voxels,
}

/// The primary interface for quick introspection into the virtual file system.
#[derive(Debug)]
pub enum Ref<'v> {
	File {
		vfs: &'v VirtualFs,
		file: &'v Arc<VFile>,
		content: Option<Guard<Content>>,
	},
	Folder {
		vfs: &'v VirtualFs,
		folder: &'v Folder,
	},
}

impl Ref<'_> {
	#[must_use]
	pub fn vfs(&self) -> &VirtualFs {
		match self {
			Self::File { vfs, .. } => vfs,
			Self::Folder { vfs, .. } => vfs,
		}
	}

	#[must_use]
	pub fn path(&self) -> &VPath {
		match self {
			Self::File { file, .. } => std::borrow::Borrow::borrow(&file.path),
			Self::Folder { folder, .. } => std::borrow::Borrow::borrow(&folder.path),
		}
	}

	// SAFETY: When forcing a string read, the absence of a binary content ID
	// guarantees that this virtual file's content was verified to be valid UTF-8
	// upon its creation, and it has been immutable since then.

	#[must_use]
	pub fn try_read_str(&self) -> Option<&str> {
		let Self::File { content, .. } = self else {
			return None;
		};

		let Some(content) = content else {
			return None;
		};

		if !content.header.header.is_text() {
			return None;
		}

		Some(unsafe { std::str::from_utf8_unchecked(&content.slice) })
	}

	#[must_use]
	pub fn read_str(&self) -> &str {
		let Self::File { content, .. } = self else {
			panic!("tried to `read_str` from a virtual folder")
		};

		let Some(content) = content else {
			panic!("tried to `read_str` from an empty virtual file")
		};

		assert!(
			!content.header.header.id.is_binary() && !content.header.header.consumed,
			"virtual file has non-UTF-8 content or has been consumed"
		);

		unsafe { std::str::from_utf8_unchecked(&content.slice) }
	}

	#[must_use]
	pub fn try_consume_text(&self) -> Option<FileText> {
		let Self::File { file, .. } = self else {
			return None;
		};

		let Some(content) = &file.content else {
			return None;
		};

		{
			let g = content.load();

			if g.header.header.consumed || !g.header.header.is_text() {
				return None;
			}
		}

		let ret = content.swap(Content::from_header_and_iter(
			ContentHeader {
				id: ContentId::None,
				consumed: true,
			},
			[].iter().copied(),
		));

		Some(FileText(ret))
	}

	/// Be aware that this will return the byte content of a UTF-8 virtual file.
	/// Consider attempting [`Self::try_consume_text`] first.
	#[must_use]
	pub fn try_consume_binary(&self) -> Option<FileBytes> {
		let Self::File { file, .. } = self else {
			return None;
		};

		let Some(content) = &file.content else {
			return None;
		};

		let ret = content.swap(ThinArc::from_header_and_iter(
			ContentHeader {
				id: ContentId::None,
				consumed: true,
			},
			[].iter().copied(),
		));

		if ret.header.header.consumed {
			return None;
		}

		Some(FileBytes(ret))
	}
}

impl PartialEq for Ref<'_> {
	/// Verify that these two refs are to the same virtual file in the same VFS.
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(
				Self::File {
					vfs: l_vfs,
					file: l_file,
					..
				},
				Self::File {
					vfs: r_vfs,
					file: r_file,
					..
				},
			) => std::ptr::eq(*l_vfs, *r_vfs) && std::ptr::eq(*l_file, *r_file),
			(
				Self::Folder {
					vfs: l_vfs,
					folder: l_folder,
				},
				Self::Folder {
					vfs: r_vfs,
					folder: r_folder,
				},
			) => std::ptr::eq(*l_vfs, *r_vfs) && std::ptr::eq(*l_folder, *r_folder),
			_ => false,
		}
	}
}

/// See [`Ref::try_consume_text`].
/// This wraps an [`Arc`] so it is relatively cheap to copy.
#[derive(Debug, Clone)]
pub struct FileText(Content);

impl std::ops::Deref for FileText {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		debug_assert!(self.0.header.header.is_text());
		// SAFETY: Same guarantees as reading a string from a `Ref`.
		unsafe { std::str::from_utf8_unchecked(&self.0.slice) }
	}
}

/// See [`Ref::try_consume_binary`].
/// This wraps an [`Arc`] so it is relatively cheap to copy.
#[derive(Debug, Clone)]
pub struct FileBytes(Content);

impl std::ops::Deref for FileBytes {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		&self.0.slice
	}
}

pub(crate) type Content = ThinArc<ContentHeader, u8>;
