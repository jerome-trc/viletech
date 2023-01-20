//! Internal implementation details that don't belong anywhere else.

use std::{
	any::TypeId,
	hash::{Hash, Hasher},
};

use fasthash::SeaHasher;
use rayon::prelude::*;
use serde::Deserialize;

use crate::{VPath, VPathBuf};

use super::{Asset, Catalog, VirtFileKind};

#[derive(Debug)]
pub(super) struct Config {
	pub(super) bin_size_limit: usize,
	pub(super) text_size_limit: usize,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			bin_size_limit: super::limits::DEFAULT_BIN_FILE_SIZE,
			text_size_limit: super::limits::DEFAULT_TEXT_FILE_SIZE,
		}
	}
}

impl Catalog {
	/// Clears the paths out of every virtual directory in preparation for
	/// repopulation via `populate_dirs`.
	pub(super) fn clear_dirs(&mut self) {
		self.files.par_iter_mut().for_each(|(_, file)| {
			if let VirtFileKind::Directory(children) = &mut file.kind {
				children.clear();
			}
		});
	}

	/// Sets every virtual directory to hold paths to child entries.
	/// Remember to call `clear_dirs` first if necessary.
	pub(super) fn populate_dirs(&mut self) {
		for index in 0..self.files.len() {
			let parent = if let Some(p) = self.files[index].parent_path() {
				VfsKey::new(p)
			} else {
				continue; // No parent; `self.files[index]` is the root node
			};

			let (&key, _) = self.files.get_index(index).unwrap();
			let parent = self.files.get_mut(&parent).unwrap();

			if let VirtFileKind::Directory(children) = &mut parent.kind {
				children.push(key);
			} else {
				unreachable!()
			}
		}
	}

	/// Truncate `self.files` and `self.mounts` back to the given points.
	pub(super) fn load_fail_cleanup(&mut self, orig_files_len: usize, orig_mounts_len: usize) {
		self.files.truncate(orig_files_len);
		self.mounts.truncate(orig_mounts_len);
	}
}

// Q: SeaHasher is used for building these two key types because it requires no
// allocation, unlike metro and xx. Are any others faster for this?

/// The catalog never deals in relative paths; the "current working directory" can
/// be considered to always be the root. To make path-hashing flexible over
/// paths that don't include a root path separator, the path is hashed by its
/// components (with a preceding path separator hashed beforehand if necessary)
/// one at a time, rather than as a whole string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct VfsKey(u64);

impl VfsKey {
	#[must_use]
	pub(super) fn new(path: impl AsRef<VPath>) -> Self {
		let mut hasher = SeaHasher::default();
		path.as_ref().hash(&mut hasher);
		Self(hasher.finish())
	}
}

/// Assets of different types are allowed to share IDs (to the scripts, the Rust
/// type system is an irrelevant detail). The actual map keys are composed by
/// hashing the ID string slice and then the type ID, in that order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct AssetKey(u64);

impl AssetKey {
	pub(super) fn new<A: Asset>(id: &str) -> Self {
		let mut hasher = SeaHasher::default();
		id.hash(&mut hasher);
		TypeId::of::<A>().hash(&mut hasher);
		Self(hasher.finish())
	}
}

/// Intermediate format for parsing parts of [`MountInfo`] from meta.toml.
///
/// [`MountInfo`]: super::MountInfo
#[derive(Debug, Default, Deserialize)]
pub(super) struct MountMetaIngest {
	pub id: String,
	#[serde(default)]
	pub version: Option<String>,
	#[serde(default)]
	pub name: Option<String>,
	#[serde(default)]
	pub description: Option<String>,
	#[serde(default)]
	pub authors: Vec<String>,
	#[serde(default)]
	pub copyright: Option<String>,
	#[serde(default)]
	pub links: Vec<String>,
	#[serde(default)]
	pub manifest: Option<VPathBuf>,
}
