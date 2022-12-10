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

use std::path::{Path, PathBuf};

use fasthash::metro;

use crate::utils::path::PathExt;

use super::Error;

crate::newtype!(
	/// To make path-hashing flexible over paths that don't include a root path
	/// separator (the VFS never deals in relative paths), the path is hashed
	/// by its components (with a preceding path separator hashed beforehand if
	/// necessary) one at a time, rather than as a whole string.
	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
	pub(super) struct PathHash(u64)
);

impl PathHash {
	#[must_use]
	pub(super) fn new(path: impl AsRef<Path>) -> Self {
		let path = path.as_ref();
		let mut hash = 0u64;

		if !path.starts_with("/") {
			hash ^= metro::hash64("/");
		}

		let comps = path.components();

		for comp in comps {
			hash ^= metro::hash64(
				comp.as_os_str()
					.to_str()
					.expect("`PathHash::new` received a path with invalid UTF-8."),
			);
		}

		Self(hash)
	}
}

/// A virtual proxy for a physical file, physical directory, or archive entry.
///
/// It's preferable to interact with these through [`FileRef`](super::FileRef),
/// since these aren't capable of independently accessing their children.
#[derive(Debug)]
pub struct Entry {
	/// Absolute virtual. Guaranteed to contain only valid UTF-8
	/// and start with a root separator.
	pub path: PathBuf,
	pub kind: EntryKind,
}

#[derive(Debug)]
pub enum EntryKind {
	/// If a file's contents pass UTF-8 validation, they are stored in one of these.
	String(String),
	/// If a file's contents do not pass UTF-8 validation, they are stored in one of these.
	Binary(Box<[u8]>),
	/// If a file's length is exactly 0, the entry will have this kind.
	Empty,
	Directory(Vec<usize>),
}

impl Entry {
	#[must_use]
	pub(super) fn new_leaf(mut virt_path: PathBuf, bytes: Vec<u8>) -> Self {
		virt_path.shrink_to_fit();

		if bytes.is_empty() {
			return Self {
				path: virt_path,
				kind: EntryKind::Empty,
			};
		}

		match String::from_utf8(bytes) {
			Ok(mut string) => {
				string.shrink_to_fit(); // Will never grow any further

				Self {
					path: virt_path,
					kind: EntryKind::String(string),
				}
			}
			Err(err) => {
				let mut bytes = err.into_bytes();
				bytes.shrink_to_fit(); // Will never grow any further

				Self {
					path: virt_path,
					kind: EntryKind::Binary(bytes.into_boxed_slice()),
				}
			}
		}
	}

	#[must_use]
	pub(super) fn new_dir(mut virt_path: PathBuf) -> Self {
		virt_path.shrink_to_fit();

		Self {
			path: virt_path,
			kind: EntryKind::Directory(Vec::default()),
		}
	}

	#[must_use]
	pub fn file_name(&self) -> &str {
		if self.path.is_root() {
			return "/";
		}

		self.path
			.file_name()
			.expect("A VFS virtual path wasn't sanitised (OS).")
			.to_str()
			.expect("A VFS virtual path wasn't sanitised (UTF-8).")
	}

	/// For dealing in files in terms of "lumps" and their 8-ASCII-character name limit.
	/// Truncates output of [`Self::file_name`] to 8 characters maximum.
	#[must_use]
	pub fn file_name_short(&self) -> &str {
		let strlen = self.file_name().chars().count().min(8);
		&self.file_name()[..strlen]
	}

	#[must_use]
	pub fn path_str(&self) -> &str {
		self.path
			.to_str()
			.expect("A VFS virtual path wasn't UTF-8 sanitised.")
	}

	#[must_use]
	pub fn is_leaf(&self) -> bool {
		!self.is_dir()
	}

	#[must_use]
	pub fn is_dir(&self) -> bool {
		matches!(self.kind, EntryKind::Directory(..))
	}

	#[must_use]
	pub fn is_binary(&self) -> bool {
		matches!(self.kind, EntryKind::Binary(..))
	}

	#[must_use]
	pub fn is_string(&self) -> bool {
		matches!(self.kind, EntryKind::String(..))
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		matches!(self.kind, EntryKind::Empty)
	}

	#[must_use]
	pub(super) fn cmp_name(a: &Entry, b: &Entry) -> std::cmp::Ordering {
		if a.is_leaf() && b.is_dir() {
			std::cmp::Ordering::Greater
		} else if a.is_dir() && b.is_leaf() {
			std::cmp::Ordering::Less
		} else {
			a.file_name().partial_cmp(b.file_name()).unwrap()
		}
	}

	/// Returns [`Error::Unreadable`] if this entry is a directory, or empty.
	pub fn read(&self) -> Result<&[u8], Error> {
		match &self.kind {
			EntryKind::Binary(bytes) => Ok(&bytes[..]),
			EntryKind::String(string) => Ok(string.as_bytes()),
			_ => Err(Error::Unreadable),
		}
	}

	/// Returns [`Error::InvalidUtf8`] if attempting to read a binary entry.
	/// Otherwise acts like [`read`].
	pub fn read_str(&self) -> Result<&str, Error> {
		match &self.kind {
			EntryKind::String(string) => Ok(string),
			EntryKind::Binary(_) => Err(Error::InvalidUtf8),
			_ => Err(Error::Unreadable),
		}
	}

	/// Like [`read`] but panics if trying to read a directory or empty entry.
	#[must_use]
	pub fn read_unchecked(&self) -> &[u8] {
		match &self.kind {
			EntryKind::Binary(bytes) => &bytes[..],
			EntryKind::String(string) => string.as_bytes(),
			_ => unreachable!("Tried to `read` a VFS directory."),
		}
	}

	/// Like [`read_str`] but panics if this isn't a string entry.
	#[must_use]
	pub fn read_str_unchecked(&self) -> &str {
		match &self.kind {
			EntryKind::String(string) => string,
			_ => unreachable!("Tried to `read_str` a VFS non-string leaf."),
		}
	}

	/// Returns [`Error::Unreadable`] if this entry is a directory, or empty.
	pub fn clone(&self) -> Result<Vec<u8>, Error> {
		match &self.kind {
			EntryKind::Binary(bytes) => Ok(bytes.clone().to_vec()),
			EntryKind::String(string) => Ok(string.as_bytes().to_owned()),
			_ => Err(Error::Unreadable),
		}
	}

	/// Returns [`Error::InvalidUtf8`] if this isn't a string entry.
	/// Otherwise acts like [`clone`].
	pub fn clone_string(&self) -> Result<String, Error> {
		match &self.kind {
			EntryKind::Binary(_) => Err(Error::InvalidUtf8),
			EntryKind::String(string) => Ok(string.clone()),
			_ => Err(Error::Unreadable),
		}
	}
}
