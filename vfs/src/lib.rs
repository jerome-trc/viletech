//! # VileTechFS
//!
//! VileTech's virtual file system; an abstraction over the operating system's
//! "physical" FS. Real files, directories, and various archives are all merged
//! into one tree so that reading from them is more convenient at all other levels
//! of the engine, without exposing any details of the user's underlying machine.

// TODO: Disallow these when done.
#![allow(unused)]
#![allow(dead_code)]

mod error;
mod file;
#[cfg(feature = "egui")]
mod gui;
mod mount;

#[cfg(test)]
pub mod test;

use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	sync::Arc,
};

use globset::Glob;
use rayon::prelude::*;
use regex::Regex;
use util::{path::PathExt, Outcome, SendTracker};

use crate::file::Content;

pub use self::{
	error::MountError,
	error::VfsError,
	file::{File, FileRef},
};

/// Disambiguates between real FS paths and virtual FS paths.
pub type VPath = std::path::Path;
/// Disambiguates between real FS paths and virtual FS paths.
pub type VPathBuf = std::path::PathBuf;

/// [`Arc`] over [`VPath`] is used instead of a plain [`VPathBuf`] to slightly
/// reduce duplication between the file map and directory sets.
pub(crate) type FileKey = Arc<VPath>;

#[derive(Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Resource))]
pub struct VirtualFs {
	/// Always contains the root node, under virtual path `/`.
	files: HashMap<FileKey, File>,
	mounts: Vec<MountInfo>,
	#[cfg(feature = "egui")]
	gui: gui::DevGui,
	config: Config,
}

impl VirtualFs {
	// Accessors ///////////////////////////////////////////////////////////////

	#[must_use]
	pub fn root(&self) -> FileRef {
		self.get("/").unwrap()
	}

	#[must_use]
	pub fn get(&self, path: impl AsRef<VPath>) -> Option<FileRef> {
		self.files
			.get_key_value(path.as_ref())
			.map(|(path, file)| FileRef {
				vfs: self,
				path,
				file,
			})
	}

	#[must_use]
	pub fn contains(&self, path: impl AsRef<VPath>) -> bool {
		self.files.contains_key(path.as_ref())
	}

	#[must_use]
	pub fn is_dir(&self, path: impl AsRef<VPath>) -> bool {
		self.files
			.get(path.as_ref())
			.filter(|f| f.is_dir())
			.is_some()
	}

	/// The total number of entries in this virtual file system, root included.
	#[must_use]
	pub fn file_count(&self) -> usize {
		self.files.len()
	}

	/// Yields every file, root included, in an unspecified order.
	pub fn iter(&self) -> impl Iterator<Item = FileRef> {
		self.files.iter().map(|(path, file)| FileRef {
			vfs: self,
			path,
			file,
		})
	}

	/// Shorthand for `all_files().par_bridge()`.
	#[must_use = "iterators are lazy and do nothing unless consumed"]
	pub fn par_iter(&self) -> impl ParallelIterator<Item = FileRef> {
		self.iter().par_bridge()
	}

	/// Yields every file whose path matches `pattern`, potentially including the root,
	/// in an unspecified order.
	pub fn glob(&self, pattern: Glob) -> impl Iterator<Item = FileRef> {
		let glob = pattern.compile_matcher();

		self.iter()
			.filter(move |file| glob.is_match(file.path_str()))
	}

	/// Shorthand for `glob().par_bridge()`.
	#[must_use = "iterators are lazy and do nothing unless consumed"]
	pub fn glob_par(&self, pattern: Glob) -> impl ParallelIterator<Item = FileRef> {
		self.glob(pattern).par_bridge()
	}

	/// Yields every file whose path matches `pattern`, potentially including the root,
	/// in an unspecified order.
	pub fn regex(&self, pattern: Regex) -> impl Iterator<Item = FileRef> {
		self.iter()
			.filter(move |file| pattern.is_match(file.path_str()))
	}

	/// Shorthand for `regex().par_bridge()`.
	#[must_use = "iterators are lazy and do nothing unless consumed"]
	pub fn regex_par(&self, pattern: Regex) -> impl ParallelIterator<Item = FileRef> {
		self.regex(pattern).par_bridge()
	}

	#[must_use]
	pub fn mounts(&self) -> &[MountInfo] {
		&self.mounts
	}

	// Mutators ////////////////////////////////////////////////////////////////

	pub fn mount(&mut self, request: MountRequest) -> MountOutcome {
		if request.load_order.is_empty() {
			return MountOutcome::NoOp;
		}

		let mnt_ctx =
			mount::Context::new(request.tracker, request.load_order.len(), request.basedata);

		// Note to reader: check `./mount.rs`.
		match self.mount_impl(&request.load_order, mnt_ctx) {
			Outcome::Ok(output) => MountOutcome::Ok(output),
			Outcome::Err(errors) => MountOutcome::Errs(errors),
			Outcome::Cancelled => MountOutcome::Cancelled,
			Outcome::None => unreachable!(),
		}
	}

	/// Note that this is with regards to the mount array, not the file tree,
	/// although all removed mounts will have their subtrees removed.
	pub fn truncate(&mut self, len: usize) {
		for i in (len + 1)..self.mounts.len() {
			let mp = self.mounts[i].mount_point().to_path_buf();
			self.remove_recursive(&mp);
		}

		self.mounts.truncate(len);
	}

	#[must_use]
	pub fn config_set(&mut self) -> ConfigSet {
		ConfigSet(self)
	}

	// Miscellaneous ///////////////////////////////////////////////////////////

	#[cfg(feature = "egui")]
	pub fn ui(&self, _ctx: &egui::Context, ui: &mut egui::Ui) {
		self.ui_impl(ui);
	}
}

/// Internals.
impl VirtualFs {
	/// Panics if attempting to remove the root node (path `/` or an empty path),
	/// or attempting to remove a directory which still has children.
	fn _remove(&mut self, path: impl AsRef<VPath>) -> Option<File> {
		assert!(!path.is_root(), "Tried to remove the root node from a VFS.");

		let removed = self.files.remove(path.as_ref());

		if let Some(r) = &removed {
			assert_eq!(
				r.child_count(),
				0,
				"Tried to remove VFS directory with children: {}",
				path.as_ref().display()
			);
			let parent_path = path.as_ref().parent().unwrap();
			let parent = self.files.get_mut(parent_path).unwrap();
			Self::unparent(parent, path)
		}

		removed
	}

	/// Panics if attempting to remove the root node (path `/` or an empty path).
	/// Trying to remove a non-existent file is valid.
	fn remove_recursive(&mut self, path: impl AsRef<VPath>) {
		assert!(!path.is_root(), "Tried to remove the root node from a VFS.");

		let Some(removed) = self.files.remove(path.as_ref()) else { return; };

		let parent_path = path.as_ref().parent().unwrap();
		let parent = self.files.get_mut(parent_path).unwrap();
		Self::unparent(parent, path);

		let Content::Directory(children) = removed.content else { return; };

		for child in children.iter() {
			recur(self, child.as_ref());
		}

		fn recur(this: &mut VirtualFs, path: impl AsRef<VPath>) {
			let Some(removed) = this.files.remove(path.as_ref()) else { unreachable!() };
			let Content::Directory(children) = removed.content else { return; };

			for child in children.iter() {
				recur(this, child.as_ref());
			}
		}
	}

	fn unparent(parent: &mut File, child_path: impl AsRef<VPath>) {
		if let Content::Directory(children) = &mut parent.content {
			children.remove(child_path.as_ref());
		} else {
			unreachable!()
		}
	}
}

impl Default for VirtualFs {
	fn default() -> Self {
		let path = VPathBuf::from("/").into();
		let root = File::new_dir();

		Self {
			files: HashMap::from([(path, root)]),
			mounts: vec![],
			#[cfg(feature = "egui")]
			gui: gui::DevGui::default(),
			config: Config::default(),
		}
	}
}

/// Also make sure to read [`VirtualFs::mount`].
#[derive(Debug)]
pub struct MountRequest {
	/// This can be empty; it makes the mount operation into a no-op.
	///
	/// With regards to mount points (`MP`):
	/// - `mymount` and `/mymount` both put the mount on the root.
	/// - An empty path and `/` are both invalid mount points.
	pub load_order: Vec<(PathBuf, VPathBuf)>,
	/// Only pass a `Some` if you need to report to the end user on the progress of
	/// a mount operation (e.g. a loading screen) or provide the ability to cancel.
	pub tracker: Option<Arc<SendTracker>>,
	/// If true, checks for reserved mount points are bypassed.
	pub basedata: bool,
}

#[derive(Debug)]
#[must_use = "mounting may return errors which should be handled"]
pub enum MountOutcome {
	NoOp,
	Cancelled,
	/// Every *new* mount gets a sub-vec, but that sub-vec may be empty.
	Errs(Vec<Vec<MountError>>),
	/// Mounting was successful, but non-fatal errors or warnings may have arisen.
	///
	/// Every *new* mount gets a sub-vec, but that sub-vec may be empty.
	Ok(Vec<Vec<MountError>>),
}

#[derive(Debug)]
pub struct MountInfo {
	/// Specified by `meta.toml` if one exists.
	/// Otherwise, this comes from the file stem of the mount point.
	pub(self) id: String,
	pub(self) format: MountFormat,
	/// Always canonicalized, but may not necessarily be valid UTF-8.
	pub(self) real_path: PathBuf,
	/// Guaranteed to be valid UTF-8 at mount time.
	pub(self) mount_point: VPathBuf,
}

impl MountInfo {
	#[must_use]
	pub fn id(&self) -> &str {
		&self.id
	}

	#[must_use]
	pub fn format(&self) -> MountFormat {
		self.format
	}

	#[must_use]
	pub fn real_path(&self) -> &Path {
		&self.real_path
	}

	#[must_use]
	pub fn mount_point(&self) -> &VPath {
		&self.mount_point
	}
}

/// Primarily serves to specify the type of compression used, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize, serde::Deserialize))]
pub enum MountFormat {
	PlainFile,
	Directory,
	Wad,
	Zip,
	// TODO: Support LZMA, XZ, GRP, PAK, RFF, SSI
}

// Config //////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
struct Config {
	pub(self) reserved_mount_points: Vec<String>,
}

/// Configuration methods are kept in a wrapper around a [`VirtualFs`] reference
/// to prevent bloat in the interface of the VFS itself.
#[derive(Debug)]
pub struct ConfigSet<'vfs>(&'vfs mut VirtualFs);

impl ConfigSet<'_> {
	pub fn reserve_mount_point(self, mp: String) -> Self {
		self.0.config.reserved_mount_points.push(mp);
		self
	}
}
