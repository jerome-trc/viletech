//! Things that can go wrong during VFS and asset management operations.

use std::path::PathBuf;

use doomfront::ParseError;
use zip::result::ZipError;

use crate::{wad, VPathBuf};

/// Things that can go wrong during (non-mounting) virtual file system operations,
/// like unmounting, lookup, and reading. Also see [`MountError`].
#[derive(Debug)]
pub enum VfsError {
	/// The caller gave a path that didn't resolve to any [`VirtualFile`].
	///
	/// [`VirtualFile`]: super::vfs::File
	NotFound(VPathBuf),
	/// The caller attempted to unmount the root node (an empty path or `/`).
	UnmountRoot,
	/// The caller tried to read/clone the raw bytes of a file with no such content.
	ByteReadFail,
	/// The caller tried to read/clone the text of a file with non-UTF-8 content,
	/// or no byte content whatsoever.
	StringReadFail,
}

impl std::error::Error for VfsError {}

impl std::fmt::Display for VfsError {
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

/// Things that can go wrong during (non-preparation) asset management operations,
/// like lookup and mutation. Also see [`PrepError`].
#[derive(Debug)]
pub enum AssetError {
	/// An asset ID didn't resolve to anything.
	NotFound(String),
}

impl std::error::Error for AssetError {}

impl std::fmt::Display for AssetError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::NotFound(id) => {
				write!(f, "No asset exists by the ID: {id}")
			}
		}
	}
}

/// Game loading is a two-step process; VFS mounting is the first step.
/// This covers the errors that can possibly happen during these operations.
#[derive(Debug)]
pub struct MountError {
	pub path: PathBuf,
	pub kind: MountErrorKind,
}

#[derive(Debug)]
pub enum MountErrorKind {
	/// A (non-virtual) path argument failed to canonicalize somehow.
	Canonicalization(std::io::Error),
	/// Failure to read the entries of a directory the caller wanted to mount.
	DirectoryRead(std::io::Error),
	/// The caller attempted to perform an operating on a real file,
	/// but the given path didn't resolve to anything.
	FileNotFound,
	/// Failed to read a file's bytes during a mount.
	FileRead(std::io::Error),
	/// Failed to acquire a file's type while attempting to mount a directory.
	FileType(std::io::Error),
	/// The given mount point wasn't valid UTF-8, had invalid characters, had a
	/// component comprised only of `.` characters, or had a component with a
	/// reserved name in it.
	InvalidMountPoint(MountPointError),
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
	ZipArchiveRead(ZipError),
	/// Indexed retrieval of a zip archive entry failed.
	ZipFileGet(usize, ZipError),
	/// A zip archive entry contains an unsafe or malformed name.
	ZipFileName(String),
	/// Failed to correctly read all the bytes in a zip archive entry.
	ZipFileRead {
		name: PathBuf,
		err: Option<std::io::Error>,
	},
}

#[derive(Debug)]
pub enum MountPointError {
	InvalidUtf8,
	/// A component in the path is `.` or `..`.
	Relative,
	/// One of the components in the mount path is engine-reserved.
	Reserved,
}

impl std::error::Error for MountError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match &self.kind {
			MountErrorKind::Canonicalization(err) => Some(err),
			MountErrorKind::DirectoryRead(err) => Some(err),
			MountErrorKind::FileRead(err) => Some(err),
			MountErrorKind::Metadata(err) => Some(err),
			MountErrorKind::Wad(err) => Some(err),
			MountErrorKind::ZipArchiveRead(err) => Some(err),
			_ => None,
		}
	}
}

impl std::fmt::Display for MountError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.kind {
			MountErrorKind::Canonicalization(err) => {
				write!(f, "Failed to canonicalize a given path. Details: {err}")
			}
			MountErrorKind::DirectoryRead(err) => {
				write!(f, "Failed to read a directory's contents: {err}")
			}
			MountErrorKind::FileNotFound => {
				write!(f, "No file exists at path: {}", self.path.display())
			}
			MountErrorKind::FileRead(err) => {
				write!(f, "File read failed: {err}")
			}
			MountErrorKind::FileType(err) => {
				write!(f, "Failed to retrieve type of file: {err}")
			}
			MountErrorKind::InvalidMountPoint(err) => {
				write!(
					f,
					"Mount point is invalid: {p}\r\n\t\
					Reason: {e}",
					p = self.path.display(),
					e = match err {
						MountPointError::InvalidUtf8 => "Path is not valid UTF-8.",
						MountPointError::Relative => "Path contains a `.` or `..` component.",
						MountPointError::Reserved =>
							"Path contains a component that is engine-reserved.",
					}
				)
			}
			MountErrorKind::MountFallthrough => {
				write!(f, "Another mount operation failed, so this one failed.")
			}
			MountErrorKind::MountHidden => {
				write!(f, "Tried to mount a hidden file (name starting with `.`).")
			}
			MountErrorKind::MountParentNotFound(path) => {
				write!(
					f,
					"A mount point's parent path mapped to no virtual file: {}",
					path.display()
				)
			}
			MountErrorKind::MountSymlink => {
				write!(f, "Tried to mount a symbolic link.")
			}
			MountErrorKind::ParentlessMountPoint => {
				write!(f, "The given path has no parent path.")
			}
			MountErrorKind::Metadata(err) => {
				write!(f, "Failed to get file metadata: {err}")
			}
			MountErrorKind::Remount => {
				write!(
					f,
					"Attempted to overwrite an existing entry with a new mount."
				)
			}
			MountErrorKind::Wad(err) => {
				write!(f, "Failed to parse a WAD archive: {err}")
			}
			MountErrorKind::ZipArchiveRead(err) => {
				write!(f, "Failed to open a zip archive: {err}")
			}
			MountErrorKind::ZipFileGet(index, err) => {
				write!(
					f,
					"Failed to get zip archive entry by index: {index} ({err})"
				)
			}
			MountErrorKind::ZipFileName(name) => {
				write!(f, "Zip archive entry name is malformed or unsafe: {name}")
			}
			MountErrorKind::ZipFileRead { name, err } => {
				if let Some(err) = err {
					write!(
						f,
						"Failed to read zip archive entry: {n} ({err})",
						n = name.display()
					)
				} else {
					write!(
						f,
						"Failed to read all content of zip archive entry: {n}",
						n = name.display()
					)
				}
			}
		}
	}
}

#[derive(Debug)]
pub struct PrepError {
	pub path: VPathBuf,
	pub kind: PrepErrorKind,
	/// If one of these arises during a prep pass,
	/// the load process must stop before moving on to the next pass.
	pub fatal: bool,
}

/// Game loading is a two-step process; asset preparation is the second step.
/// This covers the errors that can possibly happen during these operations.
#[derive(Debug)]
pub enum PrepErrorKind {
	/// A [COLORMAP] WAD lump is the wrong size.
	///
	/// [COLORMAP]: crate::data::extras::ColorMap
	ColorMap(usize),
	/// An [ENDOOM] WAD lump is the wrong size.
	///
	/// [ENDOOM]: crate::data::extras::EnDoom
	EnDoom(usize),
	Level(LevelError),
	Io(std::io::Error),
	/// A mount declared a script root file that was not found in the VFS.
	MissingVzsDir,
	/// A [PNAMES] WAD lump is too short or an incorrect size.
	///
	/// [PNAMES]: https://doomwiki.org/wiki/PNAMES
	PNames,
	/// A [TEXTURE1 or TEXTURE2] WAD lump is too short or an incorrect size.
	///
	/// [TEXTURE1 or TEXTURE2]: https://doomwiki.org/wiki/TEXTURE1_and_TEXTURE2
	TextureX,
	VzsParse(ParseError),
	/// Failure to decode a FLAC, MP3, Ogg, or WAV file.
	WaveformAudio(kira::sound::FromFileError),
}

impl std::error::Error for PrepError {}

impl std::fmt::Display for PrepError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.kind {
			PrepErrorKind::ColorMap(size) => {
				write!(
					f,
					"`COLORMAP` lump is the wrong size: {p}\r\n\t\
					Expected 8704, but found: {size}",
					p = self.path.display()
				)
			}
			PrepErrorKind::EnDoom(size) => {
				write!(
					f,
					"`ENDOOM` lump is the wrong size: {p}\r\n\t\
					Expected 4000, but found: {size}",
					p = self.path.display()
				)
			}
			PrepErrorKind::Io(err) => err.fmt(f),
			PrepErrorKind::Level(err) => {
				write!(
					f,
					"Map `{}` is invalid. Reason: {err}",
					self.path.display(),
					err = match err {
						LevelError::MalformedFile(file) => {
							format!("`{}` has malformed contents.", file.display())
						}
						LevelError::UnreadableFile(file) => {
							format!("`{}` is empty or a directory.", file.display())
						}
						LevelError::UnknownLineSpecial(short) => {
							format!("Unknown line special: {short}")
						}
						LevelError::UnknownSectorSpecial(short) => {
							format!("Unknown sector special: {short}")
						}
					}
				)
			}
			PrepErrorKind::MissingVzsDir => {
				write!(f, "No directory found at path: {}", self.path.display())
			}
			PrepErrorKind::PNames => {
				write!(f, "Malformed PNAMES lump: {}", self.path.display())
			}
			PrepErrorKind::TextureX => {
				write!(
					f,
					"Malformed TEXTURE1 or TEXTURE2 lump: {}",
					self.path.display()
				)
			}
			PrepErrorKind::VzsParse(err) => err.fmt(f),
			PrepErrorKind::WaveformAudio(err) => write!(
				f,
				"Failed to load audio file: {p}\r\n\t\
				Details: {err}",
				p = self.path.display()
			),
		}
	}
}

/// Things that can go wrong when trying to process files into a [Level] asset.
///
/// [Level]: super::asset::Level
#[derive(Debug)]
pub enum LevelError {
	/// For example, a file's byte length is not divisible
	/// by the size of its individual structures.
	MalformedFile(VPathBuf),
	/// A VFS entry was deduced to be a level component,
	/// but is empty or a directory.
	UnreadableFile(VPathBuf),
	/// Non-fatal.
	UnknownLineSpecial(i16),
	/// Non-fatal.
	UnknownSectorSpecial(i16),
}
