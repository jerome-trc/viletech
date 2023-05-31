use std::{ops::Range, path::Path, sync::Arc};

use globset::Glob;
use indexmap::IndexSet;
use rayon::prelude::*;
use regex::Regex;
use util::path::PathExt;

use crate::{FileKey, VPath, VirtualFs};

#[derive(Debug)]
pub(crate) enum Reader {
	Memory(Box<[u8]>),
	File(Box<Path>),
	Wad {
		/// WADs are sometimes nested in PK3/7 (i.e. zip) files, but this is
		/// less common than not.
		reader: Arc<Reader>,
		/// Indicates what part of `reader` has to be read to get this WAD's
		/// data, header excluded.
		slice: Range<usize>,
	},
	Zip {
		/// Likely `Self::File`, but this may be nested in another archive.
		reader: Arc<Reader>,
		/// Indicates what part of `reader` has to be read to get this zip's
		/// data, header excluded.
		slice: Range<usize>,
	},
}

#[derive(Debug)]
pub struct File {
	pub(crate) content: Content,
}

/// A virtual directory or virtual proxy for a physical file or archive entry.
#[derive(Debug)]
pub(crate) enum Content {
	/// The set of children is boxed so as not to penalize the size of other files.
	Directory(Box<IndexSet<FileKey>>),
	Empty,
	File {
		reader: Arc<Reader>,
		slice: Range<usize>,
	},
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
		matches!(self.content, Content::Directory(..))
	}

	/// Returns `true` if this is a binary or text file.
	#[must_use]
	pub fn is_readable(&self) -> bool {
		!matches!(self.content, Content::Directory(..) | Content::Empty)
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		matches!(self.content, Content::Empty)
	}

	/// Panics if this file is unreadable (e.g. empty or a directory).
	#[must_use]
	pub fn read_string(&self) -> String {
		todo!()
	}

	/// Panics if this file is unreadable (e.g. empty or a directory).
	#[must_use]
	pub fn read_bytes(&self) -> Vec<u8> {
		todo!()
	}

	#[must_use]
	pub fn child_paths(&self) -> Option<impl Iterator<Item = &VPath>> {
		match &self.content {
			Content::Directory(children) => Some(children.iter().map(|arc| arc.as_ref())),
			_ => None,
		}
	}

	/// Returns 0 if this is a leaf node or an empty directory.
	#[must_use]
	pub fn child_count(&self) -> usize {
		match &self.content {
			Content::Directory(children) => children.len(),
			_ => 0,
		}
	}
}

/// Internals.
impl File {
	#[must_use]
	pub(crate) fn new_empty() -> Self {
		Self {
			content: Content::Empty,
		}
	}

	#[must_use]
	pub(crate) fn new_dir() -> Self {
		Self {
			content: Content::Directory(Box::new(indexmap::indexset! {})),
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
	pub(crate) vfs: &'vfs VirtualFs,
	pub(crate) path: &'vfs FileKey,
	pub(crate) file: &'vfs File,
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
		match &self.file.content {
			Content::Directory(children) => Some(children.iter().map(|key| {
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
		match &self.file.content {
			Content::Directory(children) => children.len(),
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

		if let Content::Directory(children) = &self.file.content {
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
