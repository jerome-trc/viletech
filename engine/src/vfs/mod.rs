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

use std::{
	collections::HashMap,
	fs,
	path::{Path, PathBuf},
};

use fasthash::metro;
use globset::Glob;
use log::info;
use log::warn;
use parking_lot::Mutex;
use rayon::prelude::*;
use regex::Regex;

mod entry;
mod error;
mod handle;
mod impure;
mod mount;
#[cfg(test)]
mod test;

use entry::{Entry, EntryKind};

pub use self::impure::{ImpureVfs, ImpureVfsHandle};
pub use error::Error;
pub use handle::Handle;

/// Abstraction over the OS file system for security and ease.
/// Inspired by PhysicsFS, but differs in that it owns every byte mounted.
/// Just the mounting process requires large amounts of time spent on file I/O,
/// so clustering a complete read along with it grants a time savings.
pub struct VirtualFs {
	entries: Vec<Entry>,
	/// Mounted game data object IDs are used as keys.
	real_paths: HashMap<String, PathBuf>,
}

// Public interface.
impl VirtualFs {
	/// For each tuple of the given slice, `::0` should be the path to the real
	/// file/directory, and `::1` should be the desired "mount point".
	/// Returns a `Vec` parallel to `mounts` which contains `true` for each
	/// successful mount and `false` otherwise.
	#[must_use]
	pub fn mount(
		&mut self,
		mounts: &[(impl AsRef<Path>, impl AsRef<Path>)],
	) -> Vec<Result<(), Error>> {
		let results = Vec::<(usize, Result<(), Error>)>::with_capacity(mounts.len());
		let results = Mutex::new(results);

		let mounts: Vec<(usize, (&Path, &Path))> = mounts
			.iter()
			.map(|pair| (pair.0.as_ref(), pair.1.as_ref()))
			.enumerate()
			.collect();

		let output = Mutex::new(Vec::<(Vec<Entry>, String, PathBuf)>::default());
		let root = self.lookup_hash(Self::hash_path("/"));
		let root_hash = root.expect("VFS root node is missing.").hash;

		mounts.par_iter().for_each(|tuple| {
			let pair = &tuple.1;

			let real_path = match pair.0.canonicalize() {
				Ok(c) => c,
				Err(err) => {
					warn!(
						"Failed to canonicalize real path: {}
						Error: {}",
						pair.0.display(),
						err
					);
					results
						.lock()
						.push((tuple.0, Err(Error::Canonicalization(err))));
					return;
				}
			};

			let mount_point = pair.1;

			// Don't let the caller mount symbolic links, etc.

			match Self::mount_supported(&real_path) {
				Ok(()) => {}
				Err(err) => {
					warn!(
						"Attempted to mount an unsupported file: {}
						Reason: {}",
						real_path.display(),
						err
					);
					results.lock().push((tuple.0, Err(err)));
					return;
				}
			};

			let mpoint_str = match mount_point.to_str() {
				Some(s) => s,
				None => {
					warn!(
						"Attempted to use a mount point that isn't valid Unicode ({})",
						mount_point.display()
					);
					results.lock().push((tuple.0, Err(Error::InvalidUtf8)));
					return;
				}
			};

			if RGX_INVALIDMOUNTPATH.is_match(mpoint_str) {
				warn!(
					"Attempted to use a mount point that isn't comprised \
					solely of alphanumerics, underscores, dashes, periods, \
					and forward slashes. ({})",
					mount_point.display()
				);
				results
					.lock()
					.push((tuple.0, Err(Error::InvalidMountPoint)));
				return;
			}

			// Ensure nothing already exists at end of mount point

			if self.exists(mount_point) {
				results.lock().push((tuple.0, Err(Error::Remount)));
				return;
			}

			// All checks passed. Start recurring down real path

			let mut mpoint = PathBuf::new();

			if !mount_point.starts_with("/") {
				mpoint.push("/");
			}

			mpoint.push(mount_point);

			let res = if real_path.is_dir() {
				Self::mount_dir(&real_path, mpoint.clone(), root_hash)
			} else {
				let bytes = match fs::read(&real_path) {
					Ok(b) => b,
					Err(err) => {
						warn!(
							"Failed to read object for mounting: {}
							Error: {}",
							real_path.display(),
							err
						);

						results.lock().push((tuple.0, Err(Error::IoError(err))));
						return;
					}
				};

				Self::mount_file(bytes, mpoint.clone(), root_hash)
			};

			let new_entries = match res {
				Ok(e) => e,
				Err(err) => {
					warn!(
						"Failed to mount object: {}
						Error: {}",
						real_path.display(),
						err
					);
					return;
				}
			};

			info!(
				"Mounted: \"{}\" -> \"{}\".",
				real_path.display(),
				mpoint.display()
			);

			output
				.lock()
				.push((new_entries, mpoint.to_str().unwrap().to_owned(), real_path));
			results.lock().push((tuple.0, Ok(())));
		});

		let mut output = output.into_inner();

		for mut troika in output.drain(..) {
			self.entries.append(&mut troika.0);
			troika.1.remove(0); // Take off preceding root backslash
			self.real_paths.insert(troika.1, troika.2);
		}

		let mut results = results.into_inner();
		let mut ret = Vec::<Result<(), Error>>::with_capacity(results.len());

		while !results.is_empty() {
			let mut i = 0;

			while i < results.len() {
				if results[i].0 == ret.len() {
					ret.push(results.swap_remove(i).1);
				} else {
					i += 1;
				}
			}
		}

		ret
	}

	pub fn mount_supported(path: impl AsRef<Path>) -> Result<(), Error> {
		let path = path.as_ref();

		if !path.exists() {
			return Err(Error::NonExistentFile(path.to_owned()));
		}

		if path.is_symlink() {
			return Err(Error::SymlinkMount);
		}

		Ok(())
	}

	/// Returns `None` if and only if nothing exists at the given path.
	#[must_use]
	pub fn lookup(&self, path: impl AsRef<Path>) -> Option<Handle> {
		let entry = match self.lookup_hash(Self::hash_path(path)) {
			Some(e) => e,
			None => {
				return None;
			}
		};

		Some(Handle { vfs: self, entry })
	}

	/// Returns `None` if and only if nothing exists at the given path.
	/// Note that that `path` must be exact, including the root path separator.
	#[must_use]
	pub fn lookup_nocase(&self, path: impl AsRef<Path>) -> Option<Handle> {
		self.entries
			.iter()
			.find(|e| {
				e.path_str().eq_ignore_ascii_case(
					path.as_ref()
						.to_str()
						.expect("`lookup_nocase` received a path with invalid UTF-8."),
				)
			})
			.map(|e| Handle {
				vfs: self,
				entry: e,
			})
	}

	pub fn exists(&self, path: impl AsRef<Path>) -> bool {
		self.lookup(path).is_some()
	}

	/// Returns `false` if nothing is at the given path.
	#[must_use]
	pub fn is_dir(&self, path: impl AsRef<Path>) -> bool {
		match self.lookup(path) {
			Some(entry) => entry.is_dir(),
			None => false,
		}
	}

	/// Returns [`Error::NonExistentEntry`] if there's nothing at the supplied path,
	/// or [`Error::Unreadable`] if attempting to read a directory.
	pub fn read(&self, path: impl AsRef<Path>) -> Result<&[u8], Error> {
		let path = path.as_ref();

		let entry = match self.lookup_hash(Self::hash_path(path)) {
			Some(e) => e,
			None => {
				return Err(Error::NonExistentEntry(path.to_owned()));
			}
		};

		match &entry.kind {
			EntryKind::Directory { .. } => Err(Error::Unreadable),
			EntryKind::Leaf { bytes } => Ok(&bytes[..]),
		}
	}

	/// Returns [`Error::InvalidUtf8`] if the contents at the path are not valid UTF-8.
	/// Otherwise acts like [`VirtualFs::read`].
	pub fn read_str(&self, path: impl AsRef<Path>) -> Result<&str, Error> {
		let bytes = self.read(path)?;

		match std::str::from_utf8(bytes) {
			Ok(ret) => Ok(ret),
			Err(_) => Err(Error::InvalidUtf8),
		}
	}

	/// Returns `Some(0)` if the given path is a leaf node.
	/// Returns `None` if and only if nothing exists at the given path.
	#[must_use]
	pub fn count(&self, path: impl AsRef<Path>) -> Option<usize> {
		let entry = self.lookup_hash(Self::hash_path(path))?;

		if entry.is_leaf() {
			Some(0)
		} else {
			Some(self.children_of(entry).count())
		}
	}

	/// The total number of mounted entries, excluding the root.
	#[must_use]
	pub fn total_count(&self) -> usize {
		self.entries.len() - 1
	}

	#[must_use]
	pub fn mount_count(&self) -> usize {
		self.real_paths.len()
	}

	#[must_use]
	pub fn glob(&self, pattern: Glob) -> Option<impl Iterator<Item = Handle>> {
		let glob = pattern.compile_matcher();

		Some(
			self.entries
				.iter()
				.filter(move |e| glob.is_match(e.path_str()))
				.map(move |e| Handle {
					vfs: self,
					entry: e,
				}),
		)
	}

	#[must_use]
	pub fn diag(&self) -> DiagInfo {
		DiagInfo {
			mount_count: self.real_paths.len(),
			num_entries: self.entries.len(),
			mem_usage: self.mem_usage(&self.entries[0]),
		}
	}
}

impl Default for VirtualFs {
	#[must_use]
	fn default() -> Self {
		VirtualFs {
			entries: vec![Entry::new_dir(PathBuf::from("/"), 0)],
			real_paths: Default::default(),
		}
	}
}

pub struct DiagInfo {
	pub mount_count: usize,
	pub num_entries: usize,
	pub mem_usage: usize,
}

// Miscellaneous internal implementation details.
impl VirtualFs {
	/// To make path-hashing flexible over paths that don't include a root path
	/// separator (the VFS never deals in relative paths), the path is hashed
	/// by its components (with a preceding path separator hashed beforehand if
	/// necessary) one at a time, rather than as a whole string.
	#[must_use]
	fn hash_path(path: impl AsRef<Path>) -> u64 {
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
					.expect("`hash_path` received a path with invalid UTF-8."),
			);
		}

		hash
	}

	fn children_of<'v>(&'v self, dir: &'v Entry) -> impl Iterator<Item = &'v Entry> {
		self.entries.iter().filter(|e| e.parent_hash == dir.hash)
	}

	#[must_use]
	fn lookup_hash(&self, hash: u64) -> Option<&Entry> {
		self.entries.iter().find(|e| e.hash == hash)
	}

	/// Recursively gets the total memory usage of a directory.
	#[must_use]
	fn mem_usage(&self, dir: &Entry) -> usize {
		let mut ret = 0;

		for child in self.children_of(dir) {
			ret += std::mem::size_of_val(child);

			match &child.kind {
				EntryKind::Leaf { bytes } => {
					ret += bytes.len();
				}
				EntryKind::Directory => {
					ret += self.mem_usage(child);
				}
			}
		}

		ret
	}
}

lazy_static::lazy_static! {
	static ref RGX_INVALIDMOUNTPATH: Regex = Regex::new(r"[^A-Za-z0-9-_/\.]")
		.expect("Failed to evaluate `vfs::RGX_INVALIDMOUNTPATH`.");
}
