//! The symbols making up the content of the virtual file system.

use std::sync::{
	atomic::{self, AtomicUsize},
	Arc,
};

use bevy_egui::egui;
use dashmap::DashMap;
use globset::Glob;
use indexmap::{IndexMap, IndexSet};
use rayon::prelude::*;
use regex::Regex;

use crate::{utils::path::PathExt, VPath, VPathBuf};

use super::VfsError;

#[derive(Debug)]
pub struct VirtualFs {
	/// Element 0 is always the root node, under virtual path `/`.
	files: IndexMap<FileKey, File>,
	gui_sel: AtomicUsize,
}

impl VirtualFs {
	#[must_use]
	pub fn root(&self) -> FileRef {
		let (path, file) = self.files.get_index(0).unwrap();

		FileRef {
			vfs: self,
			path,
			file,
		}
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

	pub fn insert(&mut self, path: impl Into<Arc<VPath>>, file: File) -> Option<File> {
		self.files.insert(path.into(), file)
	}

	/// Panics if attempting to remove the root node (path `/` or an empty path),
	/// or attempting to remove a directory which still has children.
	pub fn remove(&mut self, path: impl AsRef<VPath>) -> Option<File> {
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
	pub fn remove_recursive(&mut self, path: impl AsRef<VPath>) {
		assert!(!path.is_root(), "Tried to remove the root node from a VFS.");

		let Some(removed) = self.files.remove(path.as_ref()) else { return; };

		let parent_path = path.as_ref().parent().unwrap();
		let parent = self.files.get_mut(parent_path).unwrap();
		Self::unparent(parent, path);

		let File::Directory(children) = removed else { return; };

		for child in children.iter() {
			self.remove_recursive(child.as_ref());
		}
	}

	/// Leaves the root node.
	pub fn clear(&mut self) {
		self.files.truncate(1);

		let root = &mut self.files[0];
		let File::Directory(children) = root else { unreachable!() };
		children.clear();
	}

	fn unparent(parent: &mut File, child_path: impl AsRef<VPath>) {
		if let File::Directory(children) = parent {
			children.remove(child_path.as_ref());
		} else {
			unreachable!()
		}
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
	pub fn files_len(&self) -> usize {
		self.files.len()
	}

	#[must_use]
	pub fn mem_usage(&self) -> usize {
		self.files
			.par_iter()
			.fold(|| 0_usize, |acc, (_, file)| acc + file.byte_len())
			.sum()
	}

	pub(super) fn insert_dashmap(
		&mut self,
		files: DashMap<FileKey, File>,
		subtree_root_path: &VPath,
	) {
		for (key, new_file) in files {
			match self.files.entry(key) {
				indexmap::map::Entry::Occupied(occu) => panic!(
					"A VFS bulk insertion displaced entry: {}",
					occu.key().display(),
				),
				indexmap::map::Entry::Vacant(vacant) => {
					vacant.insert(new_file);
				}
			}
		}

		let subtree_parent_path = subtree_root_path.parent().unwrap();
		let subtree_parent = self.files.get_mut(subtree_parent_path).unwrap();

		if let File::Directory(children) = subtree_parent {
			children.insert(subtree_root_path.to_path_buf().into());
			children.par_sort_unstable();
		} else {
			unreachable!()
		}
	}

	// Developer GUI ///////////////////////////////////////////////////////////

	pub fn ui(&self, _ctx: &egui::Context, ui: &mut egui::Ui) {
		ui.heading("Virtual File System");

		let gui_sel = self.gui_sel.load(atomic::Ordering::Relaxed);

		if gui_sel >= self.files.len() {
			self.gui_sel.store(0, atomic::Ordering::Relaxed);
		}

		egui::ScrollArea::vertical().show(ui, |ui| {
			let kvp = self.files.get_index(gui_sel).unwrap();
			self.ui_nav(ui, kvp, gui_sel);
			let (_, file) = kvp;

			match &file {
				File::Binary(bytes) => {
					ui.label("Binary");
					let mut unit = "B";
					let mut len = bytes.len() as f64;

					if len > 1024.0 {
						len /= 1024.0;
						unit = "KB";
					}

					if len > 1024.0 {
						len /= 1024.0;
						unit = "MB";
					}

					if len > 1024.0 {
						len /= 1024.0;
						unit = "GB";
					}

					ui.label(&format!("{len:.2} {unit}"));
				}
				File::Text(string) => {
					ui.label("Text");
					ui.label(&format!("{} B", string.len()));
				}
				File::Empty => {
					ui.label("Empty");
				}
				File::Directory(dir) => {
					if dir.len() == 1 {
						ui.label("Directory: 1 child");
					} else {
						ui.label(&format!("Directory: {} children", dir.len()));
					}

					for path in dir {
						let label = egui::Label::new(path.to_string_lossy().as_ref())
							.sense(egui::Sense::click());

						let resp = ui.add(label);

						let resp = if resp.hovered() {
							resp.highlight()
						} else {
							resp
						};

						if resp.clicked() {
							let idx = self.files.get_index_of(path);
							self.gui_sel.store(idx.unwrap(), atomic::Ordering::Relaxed);
						}

						resp.on_hover_text("View");
					}
				}
			}
		});
	}

	fn ui_nav(&self, ui: &mut egui::Ui, kvp: (&FileKey, &File), gui_sel: usize) {
		let (path, _) = kvp;

		ui.horizontal(|ui| {
			ui.add_enabled_ui(gui_sel != 0, |ui| {
				if ui
					.button("\u{2B06}")
					.on_hover_text("Go to Parent")
					.clicked()
				{
					let idx = self.files.get_index_of(path.parent().unwrap());
					self.gui_sel.store(idx.unwrap(), atomic::Ordering::Relaxed);
				}
			});

			for (i, comp) in path.components().enumerate() {
				let label = egui::Label::new(comp.as_os_str().to_string_lossy().as_ref())
					.sense(egui::Sense::click());

				let resp = ui.add(label);

				let resp = if resp.hovered() {
					resp.highlight()
				} else {
					resp
				};

				if resp.clicked() {
					let p: VPathBuf = path.components().take(i + 1).collect();
					let idx = self.files.get_index_of::<VPath>(p.as_ref());
					self.gui_sel.store(idx.unwrap(), atomic::Ordering::Relaxed);
				}

				resp.on_hover_text("Go to");

				if !matches!(comp, std::path::Component::RootDir) {
					ui.label("/");
				}
			}
		});
	}
}

impl Default for VirtualFs {
	fn default() -> Self {
		let path = VPathBuf::from("/").into();
		let root = File::Directory(indexmap::indexset! {});

		Self {
			files: indexmap::indexmap! { path => root },
			gui_sel: AtomicUsize::new(0),
		}
	}
}

/// A virtual directory or virtual proxy for a physical file or archive entry.
#[derive(Debug)]
pub enum File {
	/// Fallback storage type for physical files or archive entries that can't be
	/// identified as anything else and don't pass UTF-8 validation.
	Binary(Box<[u8]>),
	/// All files that pass UTF-8 validation end up stored as one of these.
	Text(Box<str>),
	/// If a file's length is exactly 0, it's stored as this kind.
	Empty,
	/// The content of mounts that are not single files, as well as the root VFS
	/// node. Elements are ordered using `Path`'s `Ord`, except in the case of
	/// the contents of WADS, which retain their defined order.
	Directory(IndexSet<FileKey>),
}

impl File {
	/// Note that being a leaf node does not necessarily mean that this file is
	/// [readable](Self::is_readable).
	#[must_use]
	pub fn is_leaf(&self) -> bool {
		!self.is_dir()
	}

	#[must_use]
	pub fn is_dir(&self) -> bool {
		matches!(self, Self::Directory(..))
	}

	#[must_use]
	pub fn is_binary(&self) -> bool {
		matches!(self, Self::Binary(..))
	}

	#[must_use]
	pub fn is_text(&self) -> bool {
		matches!(self, Self::Text(..))
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		matches!(self, Self::Empty)
	}

	/// Returns `true` if this is a binary or text file.
	#[must_use]
	pub fn is_readable(&self) -> bool {
		self.is_binary() || self.is_text()
	}

	/// Returns [`VfsError::ByteReadFail`] if this entry is a directory,
	/// or otherwise has no byte content.
	pub fn try_read_bytes(&self) -> Result<&[u8], VfsError> {
		match self {
			Self::Binary(bytes) => Ok(bytes),
			Self::Text(string) => Ok(string.as_bytes()),
			_ => Err(VfsError::ByteReadFail),
		}
	}

	/// Like [`Self::try_read_bytes`] but panics if this is a directory,
	/// or otherwise has no byte content.
	#[must_use]
	pub fn read_bytes(&self) -> &[u8] {
		match self {
			Self::Binary(bytes) => bytes,
			Self::Text(string) => string.as_bytes(),
			_ => panic!("Tried to read the bytes of a VFS entry with no byte content."),
		}
	}

	/// Returns [`VfsError::StringReadFail`]
	/// if this is a directory, binary, or empty entry.
	pub fn try_read_str(&self) -> Result<&str, VfsError> {
		match self {
			Self::Text(string) => Ok(string.as_ref()),
			_ => Err(VfsError::StringReadFail),
		}
	}

	/// Like [`Self::try_read_str`], but panics
	/// if this is a directory, binary, or empty entry.
	#[must_use]
	pub fn read_str(&self) -> &str {
		match self {
			Self::Text(string) => string.as_ref(),
			_ => panic!("Tried to read text from a VFS entry without UTF-8 content."),
		}
	}

	/// Returns 0 for directories and empty files.
	#[must_use]
	pub fn byte_len(&self) -> usize {
		match self {
			Self::Binary(bytes) => bytes.len(),
			Self::Text(string) => string.len(),
			_ => 0,
		}
	}

	#[must_use]
	pub fn child_paths(&self) -> Option<impl Iterator<Item = &VPath>> {
		match self {
			Self::Directory(children) => Some(children.iter().map(|arc| arc.as_ref())),
			_ => None,
		}
	}

	/// Returns 0 if this is a leaf node or an empty directory.
	#[must_use]
	pub fn child_count(&self) -> usize {
		match self {
			Self::Directory(children) => children.len(),
			_ => 0,
		}
	}
}

/// [`Arc`] is used instead of [`PathBuf`] to slightly reduce duplication between
/// the file map and directory sets.
pub type FileKey = Arc<VPath>;

// FileRef /////////////////////////////////////////////////////////////////////

/// The primary interface for quick introspection into the virtual file system.
///
/// Provides read access to one entry and the VFS itself. Prefer these over working
/// directly with references to [`File`]s, since this can trace inter-file relationships.
#[derive(Debug, Clone, Copy)]
pub struct FileRef<'vfs> {
	pub(super) vfs: &'vfs VirtualFs,
	pub(super) path: &'vfs FileKey,
	pub(super) file: &'vfs File,
}

impl<'vfs> FileRef<'vfs> {
	#[must_use]
	pub fn vfs(&self) -> &VirtualFs {
		self.vfs
	}

	// Path ////////////////////////////////////////////////////////////////////

	#[must_use]
	pub fn path(&self) -> &VPath {
		self.path.as_ref()
	}

	/// See [`std::path::Path::file_name`].
	///
	/// Returns a string slice instead of an OS string slice since mounted paths
	/// are pre-sanitized.
	///
	/// Panics if this is the root.
	#[must_use]
	pub fn file_name(&self) -> &str {
		if self.path.is_root() {
			return "/";
		}

		self.path
			.file_name()
			.expect("A VFS path wasn't sanitised (OS).")
			.to_str()
			.expect("A VFS path wasn't sanitised (UTF-8).")
	}

	/// See [`std::path::Path::file_stem`].
	///
	/// Returns a string slice instead of an OS string slice since mounted paths
	/// are pre-sanitized.
	///
	/// Panics if this is the root.
	#[must_use]
	pub fn file_stem(&self) -> &str {
		if self.path.is_root() {
			return "/";
		}

		self.path
			.file_stem()
			.expect("A VFS path wasn't sanitised (OS).")
			.to_str()
			.expect("A VFS path wasn't sanitised (UTF-8).")
	}

	/// See [`std::path::Path::file_prefix`].
	///
	/// Returns a string slice instead of an OS string slice since mounted paths
	/// are pre-sanitized.
	///
	/// Panics if this is the root.
	pub fn file_prefix(&self) -> &str {
		if self.path.is_root() {
			return "/";
		}

		self.path
			.file_stem()
			.expect("A VFS path wasn't sanitised (OS).")
			.to_str()
			.expect("A VFS path wasn't sanitised (UTF-8).")
			.split('.')
			.next()
			.unwrap()
	}

	/// Quickly gets the full path as a string slice.
	/// This is infallible, since mounted paths are pre-sanitized.
	#[must_use]
	pub fn path_str(&self) -> &str {
		self.path
			.to_str()
			.expect("A VFS path wasn't UTF-8 sanitised.")
	}

	/// See [`std::path::Path::extension`].
	#[must_use]
	pub fn path_extension(&self) -> Option<&str> {
		self.path.extension().map(|os_str| {
			os_str
				.to_str()
				.expect("A VFS path wasn't sanitised (UTF-8).")
		})
	}

	/// See [`std::path::Path::parent`]. Only returns `None` if this is the root directory.
	#[must_use]
	pub fn parent_path(&self) -> Option<&VPath> {
		self.path.parent()
	}

	// Relationships ///////////////////////////////////////////////////////////

	/// This only returns `None` if this file is the root directory.
	#[must_use]
	pub fn parent(&self) -> Option<Self> {
		self.parent_path().map(|parent| {
			self.vfs
				.get(parent)
				.expect("A VFS file's parent path is invalid.")
		})
	}

	/// Non-recursive; only gets immediate children. Returns `None` if this
	/// file is not a directory; returns an empty iterator if this file is an
	/// empty directory.
	///
	/// Files are yielded in the order specified by [`std::path::Path::cmp`],
	/// unless this is a directory representing a WAD file.
	pub fn children(&self) -> Option<impl Iterator<Item = FileRef>> {
		match self.file {
			File::Directory(children) => Some(children.iter().map(|key| {
				self.vfs
					.get(key.as_ref())
					.expect("A VFS directory has a dangling child key.")
			})),
			_ => None,
		}
	}

	/// Files are yielded in the order specified by [`std::path::Path::cmp`],
	/// unless this is a directory representing a WAD file.
	pub fn children_glob(&self, pattern: Glob) -> Option<impl Iterator<Item = FileRef>> {
		let glob = pattern.compile_matcher();

		self.children()
			.map(|iter| iter.filter(move |file| glob.is_match(file.path_str())))
	}

	/// Shorthand for `children_glob().par_bridge()`.
	#[must_use = "iterators are lazy and do nothing unless consumed"]
	pub fn children_glob_par(
		&self,
		pattern: Glob,
	) -> Option<impl ParallelIterator<Item = FileRef>> {
		self.children_glob(pattern).map(|iter| iter.par_bridge())
	}

	/// Files are yielded in the order specified by [`std::path::Path::cmp`],
	/// unless this is a directory representing a WAD file.
	pub fn children_regex(&self, pattern: Regex) -> Option<impl Iterator<Item = FileRef>> {
		self.children()
			.map(|iter| iter.filter(move |file| pattern.is_match(file.path_str())))
	}

	/// Shorthand for `children_regex().par_bridge()`.
	#[must_use = "iterators are lazy and do nothing unless consumed"]
	pub fn children_regex_par(
		&self,
		pattern: Regex,
	) -> Option<impl ParallelIterator<Item = FileRef>> {
		self.children_regex(pattern).map(|iter| iter.par_bridge())
	}

	/// Returns 0 if this is a leaf node or an empty directory.
	#[must_use]
	pub fn child_count(&self) -> usize {
		match self.file {
			File::Directory(children) => children.len(),
			_ => 0,
		}
	}

	/// Panics if this is not a directory node, or if `path`'s parent is not equal
	/// to this file's path.
	#[must_use]
	pub fn child_index(&self, path: impl AsRef<VPath>) -> Option<usize> {
		let path = path.as_ref();

		if path.parent().filter(|&p| p == self.path()).is_none() {
			panic!("`child_index` expects `path` to be a child of `self.path`.");
		}

		if let File::Directory(children) = self.file {
			children.get_index_of(path)
		} else {
			panic!("`child_index` expects `self` to be a directory.");
		}
	}
}

impl std::ops::Deref for FileRef<'_> {
	type Target = File;

	fn deref(&self) -> &Self::Target {
		self.file
	}
}

impl PartialEq for FileRef<'_> {
	/// Check that these two `FileRef`s point to the same file in the same VFS.
	fn eq(&self, other: &Self) -> bool {
		std::ptr::eq(self.vfs, other.vfs) && std::ptr::eq(self.file, other.file)
	}
}

impl Eq for FileRef<'_> {}

// Internal ////////////////////////////////////////////////////////////////////

pub(super) fn sort_dirs_dashmap(files: &DashMap<FileKey, File>) {
	files.par_iter_mut().for_each(|mut kvp| {
		if let File::Directory(children) = kvp.value_mut() {
			children.par_sort_unstable();
		}
	});
}
