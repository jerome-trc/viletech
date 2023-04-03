//! The symbols making up the content of the virtual file system.

use crate::{utils::path::PathExt, VPath};

use super::{detail::VfsKey, VfsError};

/// A virtual proxy for a physical file, physical directory, or archive entry.
#[derive(Debug)]
pub struct File {
	/// Virtual and absolute.
	/// Guaranteed to contain only valid UTF-8 and start with a root separator.
	pub path: Box<VPath>,
	pub(super) kind: FileKind,
}

#[derive(Debug)]
pub(super) enum FileKind {
	/// Fallback storage type for physical files or archive entries that can't be
	/// identified as anything else and don't pass UTF-8 validation.
	Binary(Box<[u8]>),
	/// All files that pass UTF-8 validation end up stored as one of these.
	Text(Box<str>),
	/// If a file's length is exactly 0, it's stored as this kind.
	Empty,
	/// Includes mounts that aren't single files, as well as the root VFS node.
	Directory(Vec<VfsKey>),
}

impl File {
	/// See [`std::path::Path::file_name`]. Returns a string slice instead of an
	/// OS string slice since mounted paths are pre-sanitized.
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

	/// See [`std::path::Path::file_stem`]. Returns a string slice instead of an
	/// OS string slice since mounted paths are pre-sanitized.
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

	/// Quickly gets the full path as a string slice.
	/// This is infallible, since mounted paths are pre-sanitized.
	#[must_use]
	pub fn path_str(&self) -> &str {
		self.path
			.to_str()
			.expect("A VFS path wasn't UTF-8 sanitised.")
	}

	/// See [`std::path::Path::parent`].
	#[must_use]
	pub fn parent_path(&self) -> Option<&VPath> {
		self.path.parent()
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

	/// Note that being a leaf node does not necessarily mean that this file is
	/// [readable](Self::is_readable).
	#[must_use]
	pub fn is_leaf(&self) -> bool {
		!self.is_dir()
	}

	#[must_use]
	pub fn is_dir(&self) -> bool {
		matches!(self.kind, FileKind::Directory(..))
	}

	#[must_use]
	pub fn is_binary(&self) -> bool {
		matches!(self.kind, FileKind::Binary(..))
	}

	#[must_use]
	pub fn is_text(&self) -> bool {
		matches!(self.kind, FileKind::Text(..))
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		matches!(self.kind, FileKind::Empty)
	}

	/// Returns `true` if this is a binary or text file.
	#[must_use]
	pub fn is_readable(&self) -> bool {
		self.is_binary() || self.is_text()
	}

	/// Returns [`VfsError::ByteReadFail`] if this entry is a directory,
	/// or otherwise has no byte content.
	pub fn try_read_bytes(&self) -> Result<&[u8], VfsError> {
		match &self.kind {
			FileKind::Binary(bytes) => Ok(bytes),
			FileKind::Text(string) => Ok(string.as_bytes()),
			_ => Err(VfsError::ByteReadFail),
		}
	}

	/// Like [`Self::try_read_bytes`] but panics if this is a directory,
	/// or otherwise has no byte content.
	#[must_use]
	pub fn read_bytes(&self) -> &[u8] {
		match &self.kind {
			FileKind::Binary(bytes) => bytes,
			FileKind::Text(string) => string.as_bytes(),
			_ => panic!("Tried to read the bytes of a VFS entry with no byte content."),
		}
	}

	/// Returns [`VfsError::StringReadFail`]
	/// if this is a directory, binary, or empty entry.
	pub fn try_read_str(&self) -> Result<&str, VfsError> {
		match &self.kind {
			FileKind::Text(string) => Ok(string.as_ref()),
			_ => Err(VfsError::StringReadFail),
		}
	}

	/// Like [`Self::try_read_str`], but panics
	/// if this is a directory, binary, or empty entry.
	#[must_use]
	pub fn read_str(&self) -> &str {
		match &self.kind {
			FileKind::Text(string) => string.as_ref(),
			_ => panic!("Tried to read text from a VFS entry without UTF-8 content."),
		}
	}

	#[must_use]
	pub(super) fn cmp_name(a: &Self, b: &Self) -> std::cmp::Ordering {
		if a.is_leaf() && b.is_dir() {
			std::cmp::Ordering::Greater
		} else if a.is_dir() && b.is_leaf() {
			std::cmp::Ordering::Less
		} else {
			a.file_name().partial_cmp(b.file_name()).unwrap()
		}
	}

	/// Returns 0 for directories and empty files.
	#[must_use]
	pub(super) fn byte_len(&self) -> usize {
		match &self.kind {
			FileKind::Binary(bytes) => bytes.len(),
			FileKind::Text(string) => string.len(),
			_ => 0,
		}
	}
}
