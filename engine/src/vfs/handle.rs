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

use std::{borrow::Borrow, path::Path};

use fasthash::metro;
use regex::Regex;

use super::{
	entry::{Entry, EntryKind},
	error::Error,
	VirtualFs,
};

#[derive(Clone)]
pub struct Handle<'v, 'e> {
	pub(super) vfs: &'v VirtualFs,
	pub(super) entry: &'e Entry,
}

impl<'v, 'e> Handle<'v, 'e> {
	#[must_use]
	pub fn lookup(&self, path: impl AsRef<Path>) -> Option<Handle> {
		debug_assert!(self.entry.path.starts_with("/"));

		let mut hash = 0u64;

		for comp in self.entry.path.components() {
			hash ^= metro::hash64(comp.as_os_str().to_str().unwrap());
		}

		for comp in path.as_ref().components() {
			hash ^= metro::hash64(comp.as_os_str().to_str().unwrap());
		}

		self.vfs.lookup_hash(hash).map(|e| Handle {
			vfs: self.vfs,
			entry: e,
		})
	}

	#[must_use]
	pub fn lookup_nocase(&self, path: impl AsRef<Path>) -> Option<Handle> {
		let full_path = self.entry.path.join(path);

		for entry in &self.vfs.entries {
			if entry
				.path
				.to_string_lossy()
				.eq_ignore_ascii_case(full_path.to_string_lossy().borrow())
			{
				return Some(Handle {
					vfs: self.vfs,
					entry,
				});
			}
		}

		None
	}

	pub fn read(&self) -> Result<&[u8], Error> {
		match &self.entry.kind {
			EntryKind::Directory { .. } => Err(Error::Unreadable),
			EntryKind::Leaf { bytes } => Ok(&bytes[..]),
		}
	}

	/// Returns [`Error::InvalidUtf8`] if the entry's contents aren't valid UTF-8.
	/// Otherwise acts like [`Handle::read`].
	pub fn read_str(&self) -> Result<&str, Error> {
		match std::str::from_utf8(self.read()?) {
			Ok(ret) => Ok(ret),
			Err(_) => Err(Error::InvalidUtf8),
		}
	}

	/// Returns [`Error::Unreadable`] if attempting to read a directory.
	pub fn copy(&self) -> Result<Vec<u8>, Error> {
		match &self.entry.kind {
			EntryKind::Directory { .. } => Err(Error::Unreadable),
			EntryKind::Leaf { bytes } => Ok(bytes.clone()),
		}
	}

	/// Returns [`Error::InvalidUtf8`] if the entry's contents aren't valid UTF-8.
	/// Otherwise acts like [`Handle::copy`].
	pub fn copy_string(&self) -> Result<String, Error> {
		match String::from_utf8(self.copy()?) {
			Ok(ret) => Ok(ret),
			Err(_) => Err(Error::InvalidUtf8),
		}
	}

	pub fn children(&'e self) -> impl Iterator<Item = Handle> {
		self.child_entries().map(|e| Handle {
			vfs: self.vfs,
			entry: e,
		})
	}

	/// Note: non-recursive. Panics if used on a leaf node.
	/// Check to ensure it's a directory beforehand.
	#[must_use]
	pub fn contains(&self, name: &str) -> bool {
		self.child_entries().any(|e| e.file_name() == name)
	}

	/// Note: non-recursive. Panics if used on a leaf node.
	/// Check to ensure it's a directory beforehand.
	#[must_use]
	pub fn contains_regex(&self, regex: &Regex) -> bool {
		self.children().any(|h| regex.is_match(h.file_name()))
	}

	#[must_use]
	pub fn count(&self) -> usize {
		match &self.entry.kind {
			EntryKind::Leaf { .. } => 0,
			EntryKind::Directory { .. } => self.child_entries().count(),
		}
	}

	#[must_use]
	pub fn virtual_path(&self) -> &'e Path {
		&self.entry.path
	}

	#[must_use]
	pub fn path_str(&self) -> &'e str {
		self.entry.path_str()
	}

	#[must_use]
	pub fn file_name(&self) -> &str {
		self.entry.file_name()
	}

	#[must_use]
	pub fn is_dir(&self) -> bool {
		self.entry.is_dir()
	}

	#[must_use]
	pub fn is_leaf(&self) -> bool {
		self.entry.is_leaf()
	}
}

// Internal implementation details.
impl<'v, 'e> Handle<'v, 'e> {
	fn child_entries(&'e self) -> impl Iterator<Item = &'e Entry> {
		self.vfs
			.entries
			.iter()
			.filter(|e| e.parent_hash == self.entry.hash)
	}
}
