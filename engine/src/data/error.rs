//! Things that can go wrong during VFS and asset management operations.

use std::{any::TypeId, path::PathBuf};

use zip::result::ZipError;

use crate::{wad, VPathBuf};

/// Things that can go wrong during (non-mounting) virtual file system operations,
/// like unmounting, lookup, and reading. Also see [`Mount`].
#[derive(Debug)]
pub enum Vfs {
	/// The caller gave a path that didn't resolve to any [`VirtualFile`].
	///
	/// [`VirtualFile`]: super::VirtualFile
	NotFound(VPathBuf),
	/// The caller attempted to unmount the root node (an empty path or `/`).
	UnmountRoot,
	/// The caller tried to read/clone the raw bytes of a file with no such content.
	ByteReadFail,
	/// The caller tried to read/clone the text of a file with non-UTF-8 content,
	/// or no byte content whatsoever.
	StringReadFail,
}

impl std::error::Error for Vfs {}

impl std::fmt::Display for Vfs {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::NotFound(path) => {
				write!(f, "No virtual file exists under path: {}", path.display())
			}
			Self::UnmountRoot => {
				write!(f, "Attempted to unmount the root VFS entry.")
			}
			Self::ByteReadFail => {
				write!(
					f,
					"Tried to read/clone the raw bytes of a file with no such content."
				)
			}
			Self::StringReadFail => {
				write!(
					f,
					"Tried to read/clone the text of a file with non-UTF-8 content or no byte content."
				)
			}
		}
	}
}

/// Things that can go wrong during (non-postprocessing) asset management operations,
/// like lookup and mutation. Also see [`PostProc`].
#[derive(Debug)]
pub enum Asset {
	/// Tried to get a mutable reference to a [`Record`] that had
	/// [`Handle`]s to it. See [`Catalog::try_mutate`].
	///
	/// [`Record`]: super::Record
	/// [`Handle`]: super::Handle
	/// [`Catalog::try_mutate`]: super::Catalog::try_mutate
	Immutable(usize),
	/// An asset ID didn't resolve to a [`Record`].
	///
	/// [`Record`]: super::Record
	NotFound(String),
	/// A caller tried to get a [`Handle`] to an asset by path and found it,
	/// but requested a type different to the [`Record`]'s storage type.
	///
	/// [`Handle`]: super::Handle
	/// [`Record`]: super::Record
	TypeMismatch { expected: TypeId, given: TypeId },
}

impl std::error::Error for Asset {}

impl std::fmt::Display for Asset {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Immutable(handles) => {
				write!(
					f,
					"Tried to mutate an asset record with {handles} outstanding handles."
				)
			}
			Self::NotFound(id) => {
				write!(f, "No asset exists by the ID: {id}")
			}
			Self::TypeMismatch { expected, given } => {
				write!(
					f,
					"Type mismatch during asset lookup. Expected {e:#?}, got {g:#?}.",
					e = expected,
					g = given
				)
			}
		}
	}
}

/// Things that can possibly go wrong during a [`load`] operation.
///
/// [`load`]: super::Catalog::load
#[derive(Debug)]
pub enum Load {
	Mount(Mount),
	PostProc(PostProc),
}

impl std::error::Error for Load {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Self::Mount(err) => Some(err),
			Self::PostProc(err) => Some(err),
		}
	}
}

impl std::fmt::Display for Load {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Mount(err) => err.fmt(f),
			Self::PostProc(err) => err.fmt(f),
		}
	}
}

/// Game loading is a two-step process; VFS mounting is the first step.
/// This covers the errors that can possibly happen during these operations.
#[derive(Debug)]
pub enum Mount {
	/// A (non-virtual) path argument failed to canonicalize somehow.
	Canonicalization(std::io::Error),
	/// Failure to read the entries of a directory the caller wanted to mount.
	DirectoryRead(std::io::Error),
	/// Failed to read a file's bytes during a mount.
	FileRead(std::io::Error),
	/// The caller attempted to perform an operating on a real file,
	/// but the given path didn't resolve to anything.
	FileNotFound(PathBuf),
	/// The given mount point wasn't valid UTF-8, had invalid characters, had a
	/// component comprised only of `.` characters, or had a component with a
	/// reserved name in it.
	InvalidMountPoint(VPathBuf, &'static str),
	/// Mount batch operations are atomic; if one fails, they all fail.
	/// During a non-parallel mount, the first failure will have a specific error,
	/// and results for all following requested mounts will be this, to avoid
	/// doing unnecessary work when a batch is already a failure.
	/// During a parallelized mount, this error will never appear, so it's clear
	/// which mounts succeeded and which are problematic.
	MountFallthrough,
	/// The caller attempted to mount a file whose name starts with a `.`.
	MountHidden,
	/// The virtual file which the mount path points to as a parent wasn't found.
	MountParentNotFound(VPathBuf),
	/// The caller attempted to illegally mount a symbolic link.
	MountSymlink,
	/// If, for example, a mount point `/hello/world/foo/bar` is given, its parent
	/// path is `/hello/world/foo`. If the parent of a given mount point cannot
	/// be retrieved for whatever reason, this error will be emitted.
	ParentlessMountPoint,
	/// Failed to read the metadata for a mount's top-level real file;
	/// the user likely lacks the operating system permission.
	Metadata(std::io::Error),
	/// The caller attempted to mount something to a point which
	/// already had something mounted onto it.
	Remount,
	/// Something went wrong when trying to parse a WAD archive during loading.
	Wad(wad::Error),
	/// Something went wrong when trying to open a zip archive during loading.
	Zip(ZipError),
}

impl std::error::Error for Mount {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Self::Canonicalization(err) => Some(err),
			Self::DirectoryRead(err) => Some(err),
			Self::FileRead(err) => Some(err),
			Self::Metadata(err) => Some(err),
			Self::Wad(err) => Some(err),
			Self::Zip(err) => Some(err),
			_ => None,
		}
	}
}

impl std::fmt::Display for Mount {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Canonicalization(err) => {
				write!(f, "Failed to canonicalize a given path. Details: {err}")
			}
			Self::DirectoryRead(err) => {
				write!(f, "Failed to read a directory's contents: {err}")
			}
			Self::FileRead(err) => {
				write!(f, "File read failed: {err}")
			}
			Self::FileNotFound(path) => {
				write!(f, "No file exists at path: {}", path.display())
			}
			Self::InvalidMountPoint(path, reason) => {
				write!(
					f,
					"Mount point is invalid: {p}\r\n\t\
					Reason: {reason}",
					p = path.display()
				)
			}
			Self::MountFallthrough => {
				write!(f, "Another mount operation failed, so this one failed.")
			}
			Self::MountHidden => {
				write!(f, "Tried to mount a hidden file (name starting with `.`).")
			}
			Self::MountParentNotFound(path) => {
				write!(
					f,
					"A mount point's parent path mapped to no virtual file: {}",
					path.display()
				)
			}
			Self::MountSymlink => {
				write!(f, "Tried to mount a symbolic link.")
			}
			Self::ParentlessMountPoint => {
				write!(f, "The given path has no parent path.")
			}
			Self::Metadata(err) => {
				write!(f, "Failed to get file metadata: {err}")
			}
			Self::Remount => {
				write!(
					f,
					"Attempted to overwrite an existing entry with a new mount."
				)
			}
			Self::Wad(err) => {
				write!(f, "Failed to parse a WAD archive: {err}")
			}
			Self::Zip(err) => {
				write!(f, "Failed to open a zip archive: {err}")
			}
		}
	}
}

/// Game loading is a two-step process; post-processing is the second step.
/// This covers the errors that can possibly happen during these operations.
#[derive(Debug)]
pub enum PostProc {}

impl std::error::Error for PostProc {}

impl std::fmt::Display for PostProc {
	fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		unimplemented!("Soon!")
	}
}
