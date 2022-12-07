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

use std::path::PathBuf;

use fasthash::metro;

use crate::utils::path::PathExt;

pub(super) struct Entry {
	/// Absolute virtual. Guaranteed to contain only valid UTF-8
	/// and start with a root separator.
	pub(super) path: PathBuf,
	pub(super) kind: EntryKind,
	/// Derived from `path`; see [`VirtualFs::hash_path`].
	pub(super) hash: u64,
	/// Should only be 0 for the root node.
	/// Corresponds to parent's `hash`.
	pub(super) parent_hash: u64,
}

pub(super) enum EntryKind {
	String(String),
	Binary(Vec<u8>),
	Directory,
}

impl Entry {
	#[must_use]
	pub(super) fn new_leaf(virt_path: PathBuf, parent_hash: u64, bytes: Vec<u8>) -> Self {
		let mut hash = 0u64;
		let comps = virt_path.as_path().components();

		for comp in comps {
			hash ^= metro::hash64(
				comp.as_os_str()
					.to_str()
					.expect("A VFS virtual path wasn't sanitised (UTF-8)."),
			);
		}

		match String::from_utf8(bytes) {
			Ok(string) => Self {
				path: virt_path,
				kind: EntryKind::String(string),
				hash,
				parent_hash,
			},
			Err(err) => Self {
				path: virt_path,
				kind: EntryKind::Binary(err.into_bytes()),
				hash,
				parent_hash,
			},
		}
	}

	#[must_use]
	pub(super) fn new_dir(virt_path: PathBuf, parent_hash: u64) -> Self {
		let mut hash = 0u64;
		let comps = virt_path.as_path().components();

		for comp in comps {
			hash ^= metro::hash64(
				comp.as_os_str()
					.to_str()
					.expect("A VFS virtual path wasn't sanitised (UTF-8)."),
			);
		}

		Self {
			path: virt_path,
			kind: EntryKind::Directory,
			hash,
			parent_hash,
		}
	}

	#[must_use]
	pub(super) fn file_name(&self) -> &str {
		if self.path.is_root() {
			return "/";
		}

		self.path
			.file_name()
			.expect("A VFS virtual path wasn't sanitised (OS).")
			.to_str()
			.expect("A VFS virtual path wasn't sanitised (UTF-8).")
	}

	#[must_use]
	pub(super) fn path_str(&self) -> &str {
		self.path
			.to_str()
			.expect("A VFS virtual path wasn't UTF-8 sanitised.")
	}

	#[must_use]
	pub(super) fn is_leaf(&self) -> bool {
		!self.is_dir()
	}

	#[must_use]
	pub(super) fn is_dir(&self) -> bool {
		matches!(self.kind, EntryKind::Directory { .. })
	}

	#[must_use]
	pub(super) fn is_binary(&self) -> bool {
		matches!(self.kind, EntryKind::Binary { .. })
	}

	#[must_use]
	#[allow(unused)]
	pub(super) fn is_string(&self) -> bool {
		matches!(self.kind, EntryKind::String { .. })
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

	#[must_use]
	pub(super) fn read(&self) -> &[u8] {
		match &self.kind {
			EntryKind::Binary(bytes) => &bytes[..],
			EntryKind::String(string) => string.as_bytes(),
			EntryKind::Directory { .. } => unreachable!("Tried to `read` a VFS directory."),
		}
	}

	#[must_use]
	pub(super) fn read_str(&self) -> &str {
		match &self.kind {
			EntryKind::String(string) => &string,
			EntryKind::Binary { .. } | EntryKind::Directory { .. } => {
				unreachable!("Tried to `read_str` a VFS directory or binary leaf.")
			}
		}
	}
}
