//! Abstraction over the OS file system for security and ease.

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

// [Rat] If you're reading this, congratulations!
// This module sub-tree is, historically speaking, the most tortured code in Impure.

mod entry;
mod error;
mod fileref;
mod impure;
mod mount;
#[cfg(test)]
mod test;

use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};

use globset::Glob;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use regex::Regex;

use entry::{Entry, EntryKind, PathHash};

use crate::utils::path::PathExt;

pub use self::impure::{ImpureFileRef, ImpureVfs};
pub use error::Error;
pub use fileref::FileRef;

/// Abstraction over the OS file system for security and ease.
///
/// Inspired by PhysicsFS, but differs in that it owns every byte mounted.
/// Just the mounting process requires large amounts of time spent on file I/O,
/// so clustering a complete read along with it grants a time savings.
#[derive(Debug)]
pub struct VirtualFs {
	/// The first entry is always the root node. This is of the kind
	/// [`EntryKind::Directory`], and lies under the virtual path `/`.
	entries: IndexMap<PathHash, Entry>,
	/// Real filesystem paths are used as keys; retrieved values can be fed into
	/// `entries` to perform real-to-virtual path resolution.
	real_paths: HashMap<PathBuf, PathHash>,
	/// Monotonic counter for the number of changes made to this VFS. Starts at 0,
	/// and is incremented by 1 every time the VFS is modified. This includes
	/// single mounts, single unmounts, or multiple mounts at once (as long as
	/// at least one of those mounts succeeds).
	///
	/// When `debug_assertions` is on, attempting to index into the VFS with an
	/// external [`Handle`] will check to ensure that it has the same generation
	/// as the VFS does. A mismatch will cause a panic.
	generation: u16,
}

// Public interface.
impl VirtualFs {
	/// For each tuple of the given slice, `::0` should be the path to the real
	/// file/directory, and `::1` should be the desired "mount point".
	/// Returns a `Vec` parallel to `mounts` which contains an `Ok(())` for each
	/// successful mount and an error otherwise.
	///
	/// # Errors
	///
	/// - [`Error::NonExistentFile`] if attempting to mount a
	/// real path that points to nothing.
	/// - [`Error::SymlinkMount`] if attempting to mount a symbolic link, which
	/// is unconditionally forbidden.
	/// - [`Error::InvalidUtf8`] if a mount point is given that does not contain
	/// only valid UTF-8 characters.
	/// - [`Error::ParentlessMountPoint`] if the mount point's "parent" path
	/// somehow can not be resolved.
	/// - [`Error::NonExistentEntry`] if the VFS entry belonging to the mount point's
	/// "parent" path couldn't be resolved.
	/// - [`Error::Remount`] if there is already a VFS entry belonging to the same
	/// path as the mount point.
	/// - [`Error::IoError`] if reading physical filesystem data fails for any reason.
	#[must_use]
	pub fn mount(
		&mut self,
		mounts: &[(impl AsRef<Path>, impl AsRef<Path>)],
	) -> Vec<Result<(), Error>> {
		// Remember: never sort the whole entry array. Virtual representations of
		// WADs are expected to maintain their original order. Only real directories
		// and archive directories get sorted

		let ret = if mounts.len() <= 2 {
			self.mount_serial(mounts)
		} else {
			self.mount_parallel(mounts)
		};

		if ret.iter().any(|res| res.is_ok()) {
			self.update_dirs();
			self.generation += 1;
		}

		ret
	}

	/// # Errors
	///
	/// - [`Error::UnmountRoot`] if attempting to unmount the root node.
	/// - [`Error::NonExistentEntry`] if there's nothing to unmount at the given virtual path.
	pub fn unmount(&mut self, virtual_path: impl AsRef<Path>) -> Result<(), Error> {
		if virtual_path.is_root() {
			return Err(Error::UnmountRoot);
		}

		if !self.exists(&virtual_path) {
			return Err(Error::NonExistentEntry(virtual_path.as_ref().to_owned()));
		}

		let real_path = self
			.virtual_to_real(&virtual_path)
			.ok_or_else(|| Error::NonExistentEntry(virtual_path.as_ref().to_owned()))?;

		self.real_paths
			.remove(&real_path)
			.expect("`VirtualFs::unmount` failed to remove a real path.");

		let p = virtual_path.as_ref();
		let p_hash = PathHash::new(p);
		self.entries.retain(|_, v| {
			let ph = PathHash::new(&v.path);
			ph != p_hash && !v.path.is_child_of(p)
		});
		self.update_dirs();
		self.generation += 1;

		Ok(())
	}

	/// Removes all entries except the root.
	/// Calling this function advances the VFS "generation" by 1.
	pub fn clear(&mut self) {
		self.entries.truncate(1);
		self.update_dirs();
		self.real_paths.clear();
		self.generation += 1;
	}

	#[must_use]
	pub fn root(&self) -> FileRef {
		FileRef {
			vfs: self,
			entry: self.root_entry(),
			index: 0,
		}
	}

	/// Returns `None` if nothing exists at the given path.
	#[must_use]
	pub fn lookup(&self, path: impl AsRef<Path>) -> Option<FileRef> {
		self.entries
			.get_full(&PathHash::new(path))
			.map(|(index, _, e)| FileRef {
				vfs: self,
				entry: e,
				index,
			})
	}

	/// Returns `None` if and only if nothing exists at the given path.
	/// Note that `path` must be exact, including the root path separator.
	#[must_use]
	pub fn lookup_nocase(&self, path: impl AsRef<Path>) -> Option<FileRef> {
		let string = path.as_ref().to_string_lossy();

		self.entries
			.values()
			.enumerate()
			.find(|(_, e)| e.path_str().eq_ignore_ascii_case(string.as_ref()))
			.map(|(index, entry)| FileRef {
				vfs: self,
				entry,
				index,
			})
	}

	#[must_use]
	pub fn make_handle(&self, path: impl AsRef<Path>) -> Option<Handle> {
		self.entries
			.get_full(&PathHash::new(path))
			.map(|(i, _, _)| Handle {
				index: i,
				generation: self.generation,
			})
	}

	/// This function can't fail, since a `Handle` is assumed to always be correct.
	/// In a debug build, this will panic if the VFS was modified in any way after
	/// the handle was retrieved via [`make_handle`](Self::make_handle).
	/// In a release build, this function may panic or quietly emit incorrect
	/// results if the VFS gets modified after the given handle was generated.
	#[must_use]
	pub fn get(&self, handle: Handle) -> FileRef {
		debug_assert!(handle.generation == self.generation);

		FileRef {
			vfs: self,
			entry: &self.entries[handle.index],
			index: handle.index,
		}
	}

	/// Check if anything exists at the given path.
	#[must_use]
	pub fn exists(&self, path: impl AsRef<Path>) -> bool {
		self.lookup(path).is_some()
	}

	/// Returns `false` if nothing exists at the given path.
	#[must_use]
	pub fn is_dir(&self, path: impl AsRef<Path>) -> bool {
		match self.lookup(path) {
			Some(entry) => entry.is_dir(),
			None => false,
		}
	}

	/// Returns [`Error::NonExistentEntry`] if there's nothing at the supplied path,
	/// or [`Error::Unreadable`] if attempting to read a directory or empty entry.
	pub fn read(&self, path: impl AsRef<Path>) -> Result<&[u8], Error> {
		let entry = match self.entries.get(&PathHash::new(&path)) {
			Some(e) => e,
			None => {
				return Err(Error::NonExistentEntry(path.as_ref().to_owned()));
			}
		};

		match &entry.kind {
			EntryKind::Binary(..) | EntryKind::String(..) => Ok(entry.read_unchecked()),
			_ => Err(Error::Unreadable),
		}
	}

	/// # Errors
	///
	/// - [`Error::NonExistentEntry`] if nothing is found at the given path.
	/// - [`Error::InvalidUtf8`] if attempting to read from a binary entry.
	/// - [`Error::Unreadable`] if attempting to read a string from a directory entry.
	pub fn read_str(&self, path: impl AsRef<Path>) -> Result<&str, Error> {
		let bytes = self.read(path)?;

		match std::str::from_utf8(bytes) {
			Ok(ret) => Ok(ret),
			Err(_) => Err(Error::InvalidUtf8),
		}
	}

	/// The total number of mounted entries, excluding the root.
	#[must_use]
	pub fn total_count(&self) -> usize {
		self.entries.len() - 1
	}

	/// The number of real files/directories mounted anywhere in the tree.
	#[must_use]
	pub fn mount_count(&self) -> usize {
		self.real_paths.len()
	}

	/// Linear-searches for all entries which match a glob pattern.
	/// If the VFS doesn't have anything mounted (or nothing matches),
	/// the returned iterator will be empty.
	pub fn glob(&self, pattern: Glob) -> impl Iterator<Item = FileRef> {
		let glob = pattern.compile_matcher();

		self.entries
			.values()
			.enumerate()
			.filter(move |(_, e)| glob.is_match(&e.path))
			.map(|(index, entry)| FileRef {
				vfs: self,
				entry,
				index,
			})
	}

	#[must_use]
	pub fn virtual_to_real(&self, path: impl AsRef<Path>) -> Option<PathBuf> {
		let phash = PathHash::new(path);

		self.entries
			.get(&phash)
			.map(|_| self.real_paths.iter().find(|(_, ph)| **ph == phash))
			.map(|opt| {
				opt.expect("`VirtualFs::virtual_to_real` failed to resolve a real path.")
					.0
					.clone()
			})
	}

	/// The real path given must exactly match the canonicalized form of whatever
	/// was mounted.
	#[must_use]
	pub fn real_to_virtual(&self, path: impl AsRef<Path>) -> Option<PathBuf> {
		self.real_paths.get(&path.as_ref().to_owned()).map(|rp| {
			self.entries
				.get(rp)
				.expect("`VirtualFs::real_to_virtual` failed to resolve an entry.")
				.path
				.clone()
		})
	}

	/// Provides quantitative information about the VFS' current internal state.
	#[must_use]
	pub fn diag(&self) -> DiagInfo {
		let mut mem_usage = std::mem::size_of::<Self>();
		mem_usage += self.entries.capacity() * std::mem::size_of::<(PathHash, Entry)>();
		mem_usage += self.real_paths.capacity() * std::mem::size_of::<(PathBuf, PathHash)>();

		for entry in self.entries.values() {
			mem_usage += entry.path.capacity();

			match &entry.kind {
				EntryKind::String(string) => mem_usage += string.capacity(),
				EntryKind::Binary(bytes) => mem_usage += bytes.len(),
				EntryKind::Directory(children) => {
					mem_usage += children.capacity() * std::mem::size_of::<usize>()
				}
				EntryKind::Empty => {}
			}
		}

		for real_path in self.real_paths.keys() {
			mem_usage += real_path.capacity();
		}

		DiagInfo {
			mount_count: self.mount_count(),
			num_entries: self.total_count(),
			mem_usage,
		}
	}
}

#[derive(Debug)]
pub struct DiagInfo {
	pub mount_count: usize,
	pub num_entries: usize,
	pub mem_usage: usize,
}

/// An index into the VFS, allowing `O(1)` access to a single entry.
/// These are as fast to produce as a [`FileRef`] and trivial to copy.
///
/// To acquire one, use [`VirtualFs::make_handle`], and to consume one, use
/// [`VirtualFs::get`]. See those two functions' documentation for usage caveats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Handle {
	index: usize,
	/// See [`VirtualFs::generation`].
	generation: u16,
}

#[derive(Debug)]
pub struct Iter<'vfs> {
	vfs: &'vfs VirtualFs,
	elements: &'vfs [usize],
	/// This points into `elements`, not `VirtualFs::entries`.
	current: usize,
}

impl<'vfs> Iterator for Iter<'vfs> {
	type Item = &'vfs Entry;

	fn next(&mut self) -> Option<Self::Item> {
		if self.current >= self.elements.len() {
			return None;
		}

		let ret = &self.vfs.entries[self.elements[self.current]];
		self.current += 1;
		Some(ret)
	}
}

// Internal ////////////////////////////////////////////////////////////////////

impl Default for VirtualFs {
	#[must_use]
	fn default() -> Self {
		VirtualFs {
			entries: indexmap::indexmap! {
				PathHash::new("/") => Entry::new_dir(PathBuf::from("/".to_string())),
			},
			real_paths: HashMap::default(),
			generation: 0,
		}
	}
}

impl VirtualFs {
	#[must_use]
	fn root_entry(&self) -> &Entry {
		self.entries
			.get_index(0)
			.expect("The root VFS entry is not at index 0.")
			.1
	}

	/// Panics if a non-directory entry is passed to it.
	fn children_of<'vfs>(&'vfs self, entry: &'vfs Entry) -> impl Iterator<Item = &'vfs Entry> {
		match &entry.kind {
			EntryKind::Directory(elements) => Iter::<'vfs> {
				vfs: self,
				elements,
				current: 0,
			},
			_ => unreachable!(),
		}
	}

	fn update_dirs(&mut self) {
		for entry in self.entries.values_mut() {
			if let EntryKind::Directory(dir) = &mut entry.kind {
				dir.clear();
			}
		}

		for idx in 0..self.entries.len() {
			let parent_hash = if let Some(p) = self.entries[idx].path.parent() {
				PathHash::new(p)
			} else {
				// No parent; `entries[idx]` is the root
				continue;
			};

			let parent = self.entries.get_mut(&parent_hash).unwrap();

			if let EntryKind::Directory(dir) = &mut parent.kind {
				dir.push(idx);
			} else {
				unreachable!()
			}
		}
	}
}

static RGX_INVALIDMOUNTPATH: Lazy<Regex> = Lazy::new(|| {
	Regex::new(r"[^A-Za-z0-9-_/\.]").expect(stringify!(
			"Failed to evaluate regex set: "
			module_path!(),
			":",
			line!(),
			":"
			column!()
	))
});
