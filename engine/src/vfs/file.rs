use std::{hash, sync::Arc};

use globset::Glob;
use indexmap::IndexSet;
use rayon::prelude::*;
use regex::Regex;

use crate::{utils::path::PathExt, VPath};

use super::{VfsError, VirtualFs};

/// A virtual directory or virtual proxy for a physical file or archive entry.
#[derive(Debug)]
pub enum File {
	/// Fallback storage type for physical files or archive entries that can't be
	/// identified as anything else and don't pass UTF-8 validation.
	Binary(Box<[u8]>),
	/// All files that pass UTF-8 validation end up stored as one of these.
	Text(Box<str>),
	/// If a file's length is exactly 0, it's stored as this kind.
	Empty,
	/// The content of mounts that are not single files, as well as the root VFS
	/// node. Elements are ordered using `Path`'s `Ord`, except in the case of
	/// the contents of WADS, which retain their defined order.
	Directory(IndexSet<FileKey>),
}

impl File {
	/// Note that being a leaf node does not necessarily mean that this file is
	/// [readable](Self::is_readable).
	#[must_use]
	pub fn is_leaf(&self) -> bool {
		!self.is_dir()
	}

	#[must_use]
	pub fn is_dir(&self) -> bool {
		matches!(self, Self::Directory(..))
	}

	#[must_use]
	pub fn is_binary(&self) -> bool {
		matches!(self, Self::Binary(..))
	}

	#[must_use]
	pub fn is_text(&self) -> bool {
		matches!(self, Self::Text(..))
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		matches!(self, Self::Empty)
	}

	/// Returns `true` if this is a binary or text file.
	#[must_use]
	pub fn is_readable(&self) -> bool {
		self.is_binary() || self.is_text()
	}

	/// Returns [`VfsError::ByteReadFail`] if this entry is a directory,
	/// or otherwise has no byte content.
	pub fn try_read_bytes(&self) -> Result<&[u8], VfsError> {
		match self {
			Self::Binary(bytes) => Ok(bytes),
			Self::Text(string) => Ok(string.as_bytes()),
			_ => Err(VfsError::ByteReadFail),
		}
	}

	/// Like [`Self::try_read_bytes`] but panics if this is a directory,
	/// or otherwise has no byte content.
	#[must_use]
	pub fn read_bytes(&self) -> &[u8] {
		match self {
			Self::Binary(bytes) => bytes,
			Self::Text(string) => string.as_bytes(),
			_ => panic!("Tried to read the bytes of a VFS entry with no byte content."),
		}
	}

	/// Returns [`VfsError::StringReadFail`]
	/// if this is a directory, binary, or empty entry.
	pub fn try_read_str(&self) -> Result<&str, VfsError> {
		match self {
			Self::Text(string) => Ok(string.as_ref()),
			_ => Err(VfsError::StringReadFail),
		}
	}

	/// Like [`Self::try_read_str`], but panics
	/// if this is a directory, binary, or empty entry.
	#[must_use]
	pub fn read_str(&self) -> &str {
		match self {
			Self::Text(string) => string.as_ref(),
			_ => panic!("Tried to read text from a VFS entry without UTF-8 content."),
		}
	}

	/// Returns 0 for directories and empty files.
	#[must_use]
	pub fn byte_len(&self) -> usize {
		match self {
			Self::Binary(bytes) => bytes.len(),
			Self::Text(string) => string.len(),
			_ => 0,
		}
	}

	#[must_use]
	pub fn child_paths(&self) -> Option<impl Iterator<Item = &VPath>> {
		match self {
			Self::Directory(children) => Some(children.iter().map(|arc| arc.as_ref())),
			_ => None,
		}
	}

	/// Returns 0 if this is a leaf node or an empty directory.
	#[must_use]
	pub fn child_count(&self) -> usize {
		match self {
			Self::Directory(children) => children.len(),
			_ => 0,
		}
	}

	/// Returns a new [empty], [text], or [binary] file, depending on `bytes`.
	///
	/// [empty]: File::Empty
	/// [text]: File::Text
	/// [binary]: File::Binary
	#[must_use]
	pub(super) fn new_leaf(bytes: Vec<u8>) -> Self {
		if bytes.is_empty() {
			return Self::Empty;
		}

		match String::from_utf8(bytes) {
			Ok(string) => File::Text(string.into_boxed_str()),
			Err(err) => File::Binary(err.into_bytes().into_boxed_slice()),
		}
	}
}

impl hash::Hash for File {
	/// Be aware that this method panics if this is a [`File::Empty`] or
	/// [`File::Directory`] to prevent misuse.
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		match self {
			File::Binary(bytes) => bytes.hash(state),
			File::Text(string) => string.hash(state),
			File::Empty | File::Directory(_) => {
				panic!("Attempted to hash an unreadable virtual file.")
			}
		}
	}
}

// FileRef /////////////////////////////////////////////////////////////////////

/// The primary interface for quick introspection into the virtual file system.
///
/// Provides read access to one entry and the VFS itself. Prefer these over working
/// directly with references to [`File`]s, since this can trace inter-file relationships.
#[derive(Debug, Clone, Copy)]
pub struct FileRef<'vfs> {
	pub(super) vfs: &'vfs VirtualFs,
	pub(super) path: &'vfs FileKey,
	pub(super) file: &'vfs File,
}

impl<'vfs> FileRef<'vfs> {
	#[must_use]
	pub fn vfs(&self) -> &VirtualFs {
		self.vfs
	}

	// Path ////////////////////////////////////////////////////////////////////

	#[must_use]
	pub fn path(&self) -> &VPath {
		self.path.as_ref()
	}

	/// See [`std::path::Path::file_name`].
	///
	/// Returns a string slice instead of an OS string slice since mounted paths
	/// are pre-sanitized.
	///
	/// Panics if this is the root.
	#[must_use]
	pub fn file_name(&self) -> &str {
		if self.path.is_root() {
			return "/";
		}

		self.path
			.file_name()
			.expect("A VFS path wasn't sanitised (OS).")
			.to_str()
			.expect("A VFS path wasn't sanitised (UTF-8).")
	}

	/// See [`std::path::Path::file_stem`].
	///
	/// Returns a string slice instead of an OS string slice since mounted paths
	/// are pre-sanitized.
	///
	/// Panics if this is the root.
	#[must_use]
	pub fn file_stem(&self) -> &str {
		if self.path.is_root() {
			return "/";
		}

		self.path
			.file_stem()
			.expect("A VFS path wasn't sanitised (OS).")
			.to_str()
			.expect("A VFS path wasn't sanitised (UTF-8).")
	}

	/// See [`std::path::Path::file_prefix`].
	///
	/// Returns a string slice instead of an OS string slice since mounted paths
	/// are pre-sanitized.
	///
	/// Panics if this is the root.
	pub fn file_prefix(&self) -> &str {
		if self.path.is_root() {
			return "/";
		}

		self.path
			.file_stem()
			.expect("A VFS path wasn't sanitised (OS).")
			.to_str()
			.expect("A VFS path wasn't sanitised (UTF-8).")
			.split('.')
			.next()
			.unwrap()
	}

	/// Quickly gets the full path as a string slice.
	/// This is infallible, since mounted paths are pre-sanitized.
	#[must_use]
	pub fn path_str(&self) -> &str {
		self.path
			.to_str()
			.expect("A VFS path wasn't UTF-8 sanitised.")
	}

	/// See [`std::path::Path::extension`].
	#[must_use]
	pub fn path_extension(&self) -> Option<&str> {
		self.path.extension().map(|os_str| {
			os_str
				.to_str()
				.expect("A VFS path wasn't sanitised (UTF-8).")
		})
	}

	/// See [`std::path::Path::parent`]. Only returns `None` if this is the root directory.
	#[must_use]
	pub fn parent_path(&self) -> Option<&VPath> {
		self.path.parent()
	}

	// Relationships ///////////////////////////////////////////////////////////

	/// This only returns `None` if this file is the root directory.
	#[must_use]
	pub fn parent(&self) -> Option<Self> {
		self.parent_path().map(|parent| {
			self.vfs
				.get(parent)
				.expect("A VFS file's parent path is invalid.")
		})
	}

	/// Non-recursive; only gets immediate children. Returns `None` if this
	/// file is not a directory; returns an empty iterator if this file is an
	/// empty directory.
	///
	/// Files are yielded in the order specified by [`std::path::Path::cmp`],
	/// unless this is a directory representing a WAD file.
	pub fn children(&self) -> Option<impl Iterator<Item = FileRef>> {
		match self.file {
			File::Directory(children) => Some(children.iter().map(|key| {
				self.vfs
					.get(key.as_ref())
					.expect("A VFS directory has a dangling child key.")
			})),
			_ => None,
		}
	}

	/// Files are yielded in the order specified by [`std::path::Path::cmp`],
	/// unless this is a directory representing a WAD file.
	pub fn children_glob(&self, pattern: Glob) -> Option<impl Iterator<Item = FileRef>> {
		let glob = pattern.compile_matcher();

		self.children()
			.map(|iter| iter.filter(move |file| glob.is_match(file.path_str())))
	}

	/// Shorthand for `children_glob().par_bridge()`.
	#[must_use = "iterators are lazy and do nothing unless consumed"]
	pub fn children_glob_par(
		&self,
		pattern: Glob,
	) -> Option<impl ParallelIterator<Item = FileRef>> {
		self.children_glob(pattern).map(|iter| iter.par_bridge())
	}

	/// Files are yielded in the order specified by [`std::path::Path::cmp`],
	/// unless this is a directory representing a WAD file.
	pub fn children_regex(&self, pattern: Regex) -> Option<impl Iterator<Item = FileRef>> {
		self.children()
			.map(|iter| iter.filter(move |file| pattern.is_match(file.path_str())))
	}

	/// Shorthand for `children_regex().par_bridge()`.
	#[must_use = "iterators are lazy and do nothing unless consumed"]
	pub fn children_regex_par(
		&self,
		pattern: Regex,
	) -> Option<impl ParallelIterator<Item = FileRef>> {
		self.children_regex(pattern).map(|iter| iter.par_bridge())
	}

	/// Returns 0 if this is a leaf node or an empty directory.
	#[must_use]
	pub fn child_count(&self) -> usize {
		match self.file {
			File::Directory(children) => children.len(),
			_ => 0,
		}
	}

	/// Panics if this is not a directory node, or if `path`'s parent is not equal
	/// to this file's path.
	#[must_use]
	pub fn child_index(&self, path: impl AsRef<VPath>) -> Option<usize> {
		let path = path.as_ref();

		if path.parent().filter(|&p| p == self.path()).is_none() {
			panic!("`child_index` expects `path` to be a child of `self.path`.");
		}

		if let File::Directory(children) = self.file {
			children.get_index_of(path)
		} else {
			panic!("`child_index` expects `self` to be a directory.");
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
	/// Check that these two `FileRef`s point to the same file in the same VFS.
	fn eq(&self, other: &Self) -> bool {
		std::ptr::eq(self.vfs, other.vfs) && std::ptr::eq(self.file, other.file)
	}
}

impl Eq for FileRef<'_> {}

/// [`Arc`] is used instead of [`PathBuf`] to slightly reduce duplication between
/// the file map and directory sets.
pub type FileKey = Arc<VPath>;
