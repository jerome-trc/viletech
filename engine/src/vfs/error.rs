/*

Copyright (C) 2022 ***REMOVED***

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.

*/

use std::{fmt, io, path::PathBuf};

use zip::result::ZipError;

use crate::wad;

#[derive(Debug)]
pub enum Error {
	/// A path argument failed to canonicalize somehow.
	Canonicalization(io::Error),
	/// Failure to read the entries of a directory the caller wanted to mount.
	DirectoryRead(io::Error),
	/// The caller provided a mount point that isn't comprised solely of
	/// alphanumeric characters, underscores, dashes, periods, and forward slashes.
	InvalidMountPoint,
	/// A path argument did not pass a UTF-8 validity check.
	InvalidUtf8,
	IoError(io::Error),
	/// Trying to mount something onto `DOOM2/PLAYPAL`, for example, is illegal.
	MountToLeaf,
	/// The caller attempted to lookup/read/write/unmount a non-existent file.
	NonExistentEntry(PathBuf),
	/// The caller attempted to lookup/read/write/mount a non-existent file.
	NonExistentFile(PathBuf),
	/// The caller attempted to read a non-leaf node.
	Unreadable,
	/// The caller attempted to mount something to a point which
	/// already had something mounted onto it.
	Remount,
	/// The caller attempted to illegally mount a symbolic link.
	SymlinkMount,
	WadError(wad::Error),
	ZipError(ZipError),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Canonicalization(err) => {
				write!(f, "Failed to canonicalize given path: {}", err)
			}
			Self::DirectoryRead(err) => {
				write!(f, "Failed to read a directory: {}", err)
			}
			Self::InvalidMountPoint => {
				write!(
					f,
					"Mount point can only contain letters, numbers, underscores, \
					periods, dashes, and forward slashes."
				)
			}
			Self::InvalidUtf8 => {
				write!(f, "Given path failed to pass a UTF-8 validity check.")
			}
			Self::IoError(err) => {
				write!(f, "{}", err)
			}
			Self::MountToLeaf => {
				write!(
					f,
					"Attempted to mount something using an existing leaf node \
					as part of the mount point."
				)
			}
			Self::NonExistentEntry(path) => {
				write!(
					f,
					"Attempted to operate on non-existent entry: {}",
					path.display()
				)
			}
			Self::NonExistentFile(path) => {
				write!(
					f,
					"Attempted to operate on non-existent file: {}",
					path.display()
				)
			}
			Self::Unreadable => {
				write!(
					f,
					"Directories, archives, and WADs are ineligible for reading."
				)
			}
			Self::Remount => {
				write!(f, "Something is already mounted to the given path.")
			}
			Self::SymlinkMount => {
				write!(f, "Symbolic links can not be mounted.")
			}
			Self::WadError(err) => {
				write!(f, "{}", err)
			}
			Self::ZipError(err) => {
				write!(f, "{}", err)
			}
		}
	}
}
