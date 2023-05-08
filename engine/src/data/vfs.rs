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

use super::{detail::VfsKey, VfsError};

#[derive(Debug)]
pub struct VirtualFs {
	/// Element 0 is always the root node, under virtual path `/`.
	files: IndexMap<VfsKey, File>,
	gui_sel: AtomicUsize,
}

impl VirtualFs {
	#[must_use]
	pub fn root(&self) -> FileRef {
		FileRef {
			vfs: self,
			file: &self.files[0],
		}
	}

	#[must_use]
	pub fn get(&self, path: impl AsRef<VPath>) -> Option<FileRef> {
		self.files
			.get(&VfsKey::new(path))
			.map(|file| FileRef { vfs: self, file })
	}

	#[must_use]
	pub fn contains(&self, path: impl AsRef<VPath>) -> bool {
		self.files.contains_key(&VfsKey::new(path))
	}

	#[must_use]
	pub fn is_dir(&self, path: impl AsRef<VPath>) -> bool {
		self.files
			.get(&VfsKey::new(path))
			.filter(|f| f.is_dir())
			.is_some()
	}

	/// Yields every file, root included, in an unspecified order.
	pub fn iter(&self) -> impl Iterator<Item = FileRef> {
		self.files.values().map(|file| FileRef { vfs: self, file })
	}

	/// Shorthand for `all_files().par_bridge()`.
	#[must_use = "iterators are lazy and do nothing unless consumed"]
	pub fn par_iter(&self) -> impl ParallelIterator<Item = FileRef> {
		self.iter().par_bridge()
	}

	pub fn insert(&mut self, file: File) -> Option<File> {
		self.files.insert(VfsKey::new(file.path()), file)
	}

	/// Panics if attempting to remove the root node (path `/` or an empty path),
	/// or attempting to remove a directory which still has children.
	pub fn remove(&mut self, path: impl AsRef<VPath>) -> Option<File> {
		assert!(!path.is_root(), "Tried to remove the root node from a VFS.");

		let key = VfsKey::new(path);
		let entry = self.files.entry(key);

		let removed = match entry {
			indexmap::map::Entry::Occupied(occ) => {
				assert_eq!(
					occ.get().child_count(),
					0,
					"Tried to remove a VFS directory with children."
				);
				occ.remove()
			}
			indexmap::map::Entry::Vacant(_) => {
				return None;
			}
		};

		let parent_path = removed.parent_path().unwrap();
		let parent = self.files.get_mut(&VfsKey::new(parent_path)).unwrap();
		Self::unparent(parent, &removed.path);

		Some(removed)
	}

	/// Panics if attempting to remove the root node (path `/` or an empty path).
	pub fn remove_recursive(&mut self, path: impl AsRef<VPath>) {
		assert!(!path.is_root(), "Tried to remove the root node from a VFS.");

		let key = VfsKey::new(path);
		let entry = self.files.entry(key);

		let mut removed = match entry {
			indexmap::map::Entry::Occupied(occ) => occ.remove(),
			indexmap::map::Entry::Vacant(_) => return,
		};

		if let FileContent::Directory(children) = &mut removed.content {
			for child in children.iter() {
				self.remove_recursive(child.as_ref());
			}
		}

		let parent_path = removed.parent_path().unwrap();
		let parent = self.files.get_mut(&VfsKey::new(parent_path)).unwrap();
		Self::unparent(parent, &removed.path);
	}

	fn unparent(parent: &mut File, child_path: &Arc<VPath>) {
		if let FileContent::Directory(children) = &mut parent.content {
			children.remove(child_path);
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
		files: DashMap<VfsKey, File>,
		subtree_root_path: &VPath,
	) {
		for (key, new_file) in files {
			let displaced = self.files.insert(key, new_file);

			debug_assert!(
				displaced.is_none(),
				"A VFS mass insertion displaced entry: {}",
				displaced.unwrap().path_str()
			);
		}

		let subtree_root = self.files.get_mut(&VfsKey::new(subtree_root_path)).unwrap();
		let subtree_root_path = subtree_root.path.clone();

		let subtree_parent_path = subtree_root_path.parent().unwrap();
		let subtree_parent = self
			.files
			.get_mut(&VfsKey::new(subtree_parent_path))
			.unwrap();

		if let FileContent::Directory(children) = &mut subtree_parent.content {
			children.insert(subtree_root_path);
			children.par_sort_unstable_by(sort_children);
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
			let file = &self.files[gui_sel];

			self.ui_nav(ui, file, gui_sel);

			match &file.content {
				FileContent::Binary(bytes) => {
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
				FileContent::Text(string) => {
					ui.label("Text");
					ui.label(&format!("{} B", string.len()));
				}
				FileContent::Empty => {
					ui.label("Empty");
				}
				FileContent::Directory(dir) => {
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
							let idx = self.files.get_index_of(&VfsKey::new(path));
							self.gui_sel.store(idx.unwrap(), atomic::Ordering::Relaxed);
						}

						resp.on_hover_text("View");
					}
				}
			}
		});
	}

	fn ui_nav(&self, ui: &mut egui::Ui, file: &File, gui_sel: usize) {
		ui.horizontal(|ui| {
			ui.add_enabled_ui(gui_sel != 0, |ui| {
				if ui
					.button("\u{2B06}")
					.on_hover_text("Go to Parent")
					.clicked()
				{
					let idx = self
						.files
						.get_index_of(&VfsKey::new(file.parent_path().unwrap()));
					self.gui_sel.store(idx.unwrap(), atomic::Ordering::Relaxed);
				}
			});

			for (i, comp) in file.path.components().enumerate() {
				let label = egui::Label::new(comp.as_os_str().to_string_lossy().as_ref())
					.sense(egui::Sense::click());

				let resp = ui.add(label);

				let resp = if resp.hovered() {
					resp.highlight()
				} else {
					resp
				};

				if resp.clicked() {
					let p: VPathBuf = file.path.components().take(i + 1).collect();
					let idx = self.files.get_index_of(&VfsKey::new(&p));
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
		let root = File::new_dir(VPathBuf::from("/"));
		let key = VfsKey::new(root.path());

		Self {
			files: indexmap::indexmap! { key => root },
			gui_sel: AtomicUsize::new(0),
		}
	}
}

/// A virtual proxy for a physical file, physical directory, or archive entry.
#[derive(Debug)]
pub struct File {
	/// Virtual and absolute.
	/// Guaranteed to contain only valid UTF-8 and start with a root separator.
	pub(super) path: Arc<VPath>,
	pub(super) content: FileContent,
	// TODO: Reduce visibility when mounting becomes VFS behavior.
}

#[derive(Debug)]
pub(super) enum FileContent {
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
	Directory(IndexSet<Arc<VPath>>),
}

impl File {
	#[must_use]
	pub fn path(&self) -> &VPath {
		&self.path
	}

	/// See [`std::path::Path::file_name`]. Returns a string slice instead of an
	/// OS string slice since mounted paths are pre-sanitized.
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

	/// See [`std::path::Path::file_stem`]. Returns a string slice instead of an
	/// OS string slice since mounted paths are pre-sanitized.
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

	/// See [`std::path::Path::file_prefix`]. Returns a string slice instead of an
	/// OS string slice since mounted paths are pre-sanitized.
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

	/// See [`std::path::Path::parent`]. Only returns `None` if this is the root directory.
	#[must_use]
	pub fn parent_path(&self) -> Option<&VPath> {
		self.path.parent()
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

	/// Note that being a leaf node does not necessarily mean that this file is
	/// [readable](Self::is_readable).
	#[must_use]
	pub fn is_leaf(&self) -> bool {
		!self.is_dir()
	}

	#[must_use]
	pub fn is_dir(&self) -> bool {
		matches!(self.content, FileContent::Directory(..))
	}

	#[must_use]
	pub fn is_binary(&self) -> bool {
		matches!(self.content, FileContent::Binary(..))
	}

	#[must_use]
	pub fn is_text(&self) -> bool {
		matches!(self.content, FileContent::Text(..))
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		matches!(self.content, FileContent::Empty)
	}

	/// Returns `true` if this is a binary or text file.
	#[must_use]
	pub fn is_readable(&self) -> bool {
		self.is_binary() || self.is_text()
	}

	/// Returns [`VfsError::ByteReadFail`] if this entry is a directory,
	/// or otherwise has no byte content.
	pub fn try_read_bytes(&self) -> Result<&[u8], VfsError> {
		match &self.content {
			FileContent::Binary(bytes) => Ok(bytes),
			FileContent::Text(string) => Ok(string.as_bytes()),
			_ => Err(VfsError::ByteReadFail),
		}
	}

	/// Like [`Self::try_read_bytes`] but panics if this is a directory,
	/// or otherwise has no byte content.
	#[must_use]
	pub fn read_bytes(&self) -> &[u8] {
		match &self.content {
			FileContent::Binary(bytes) => bytes,
			FileContent::Text(string) => string.as_bytes(),
			_ => panic!("Tried to read the bytes of a VFS entry with no byte content."),
		}
	}

	/// Returns [`VfsError::StringReadFail`]
	/// if this is a directory, binary, or empty entry.
	pub fn try_read_str(&self) -> Result<&str, VfsError> {
		match &self.content {
			FileContent::Text(string) => Ok(string.as_ref()),
			_ => Err(VfsError::StringReadFail),
		}
	}

	/// Like [`Self::try_read_str`], but panics
	/// if this is a directory, binary, or empty entry.
	#[must_use]
	pub fn read_str(&self) -> &str {
		match &self.content {
			FileContent::Text(string) => string.as_ref(),
			_ => panic!("Tried to read text from a VFS entry without UTF-8 content."),
		}
	}

	/// Returns 0 for directories and empty files.
	#[must_use]
	pub fn byte_len(&self) -> usize {
		match &self.content {
			FileContent::Binary(bytes) => bytes.len(),
			FileContent::Text(string) => string.len(),
			_ => 0,
		}
	}

	#[must_use]
	pub fn child_paths(&self) -> Option<impl Iterator<Item = &VPath>> {
		match &self.content {
			FileContent::Directory(children) => Some(children.iter().map(|arc| arc.as_ref())),
			_ => None,
		}
	}

	/// Returns 0 if this is a leaf node or an empty directory.
	#[must_use]
	pub fn child_count(&self) -> usize {
		match &self.content {
			FileContent::Directory(children) => children.len(),
			_ => 0,
		}
	}

	#[must_use]
	pub(super) fn new_text(path: VPathBuf, string: Box<str>) -> Self {
		Self {
			path: path.into(),
			content: FileContent::Text(string),
		}
	}

	#[must_use]
	pub(super) fn new_binary(path: VPathBuf, bytes: Box<[u8]>) -> Self {
		Self {
			path: path.into(),
			content: FileContent::Binary(bytes),
		}
	}

	#[must_use]
	pub(super) fn new_empty(path: VPathBuf) -> Self {
		Self {
			path: path.into(),
			content: FileContent::Empty,
		}
	}

	#[must_use]
	pub(super) fn new_dir(path: VPathBuf) -> Self {
		Self {
			path: path.into(),
			content: FileContent::Directory(indexmap::indexset! {}),
		}
	}

	#[must_use]
	pub(super) fn path_raw(&self) -> &Arc<VPath> {
		&self.path
	}
}

// FileRef /////////////////////////////////////////////////////////////////////

/// The primary interface for quick introspection into the virtual file system.
///
/// Provides read access to one entry and the VFS itself. Prefer these over working
/// directly with references to [`File`]s, since this can trace inter-file relationships.
#[derive(Debug, Clone, Copy)]
pub struct FileRef<'vfs> {
	pub(super) vfs: &'vfs VirtualFs,
	pub(super) file: &'vfs File,
}

impl<'vfs> FileRef<'vfs> {
	#[must_use]
	pub fn vfs(&self) -> &VirtualFs {
		self.vfs
	}

	/// This only returns `None` if this file is the root directory.
	#[must_use]
	pub fn parent(&self) -> Option<&File> {
		if let Some(parent) = self.file.parent_path() {
			Some(
				self.vfs
					.files
					.get(&VfsKey::new(parent))
					.expect("A VFS entry has a dangling parent path."),
			)
		} else {
			None
		}
	}

	/// This only returns `None` if this file is the root directory.
	#[must_use]
	pub fn parent_ref(&'vfs self) -> Option<Self> {
		self.parent().map(|file| Self {
			vfs: self.vfs,
			file,
		})
	}

	/// Non-recursive; only gets immediate children. Returns `None` if this
	/// file is not a directory; returns an empty iterator if this file is an
	/// empty directory.
	///
	/// Files are yielded in the order specified by [`std::path::Path::cmp`],
	/// unless this is a directory representing a WAD file.
	pub fn children(&self) -> Option<impl Iterator<Item = &File>> {
		let closure = |path: &Arc<VPath>| {
			self.vfs
				.files
				.get(&VfsKey::new(path))
				.expect("A VFS directory has a dangling child key.")
		};

		match &self.file.content {
			FileContent::Directory(children) => Some(children.iter().map(closure)),
			_ => None,
		}
	}

	/// Calls [`Self::children`] and maps the yielded items to `FileRef`s.
	/// The same caveats apply.
	pub fn child_refs(&'vfs self) -> Option<impl Iterator<Item = FileRef<'vfs>> + '_> {
		self.children().map(|iter| {
			iter.map(|file| Self {
				vfs: self.vfs,
				file,
			})
		})
	}

	/// Files are yielded in the order specified by [`std::path::Path::cmp`],
	/// unless this is a directory representing a WAD file.
	pub fn children_glob(&self, pattern: Glob) -> Option<impl Iterator<Item = &File>> {
		let glob = pattern.compile_matcher();

		self.children()
			.map(|iter| iter.filter(move |file| glob.is_match(file.path_str())))
	}

	/// Shorthand for `children_glob().par_bridge()`.
	#[must_use = "iterators are lazy and do nothing unless consumed"]
	pub fn children_glob_par(&self, pattern: Glob) -> Option<impl ParallelIterator<Item = &File>> {
		self.children_glob(pattern).map(|iter| iter.par_bridge())
	}

	/// Files are yielded in the order specified by [`std::path::Path::cmp`],
	/// unless this is a directory representing a WAD file.
	pub fn children_regex(&self, pattern: Regex) -> Option<impl Iterator<Item = &File>> {
		self.children()
			.map(|iter| iter.filter(move |file| pattern.is_match(file.path_str())))
	}

	/// Shorthand for `children_regex().par_bridge()`.
	#[must_use = "iterators are lazy and do nothing unless consumed"]
	pub fn children_regex_par(
		&self,
		pattern: Regex,
	) -> Option<impl ParallelIterator<Item = &File>> {
		self.children_regex(pattern).map(|iter| iter.par_bridge())
	}

	/// Returns 0 if this is a leaf node or an empty directory.
	#[must_use]
	pub fn child_count(&self) -> usize {
		match &self.content {
			FileContent::Directory(children) => children.len(),
			_ => 0,
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
	fn eq(&self, other: &Self) -> bool {
		std::ptr::eq(self.vfs, other.vfs) && std::ptr::eq(self.file, other.file)
	}
}

impl Eq for FileRef<'_> {}

// Internal ////////////////////////////////////////////////////////////////////

pub(super) fn sort_dirs_dashmap(files: &DashMap<VfsKey, File>) {
	files.par_iter_mut().for_each(|mut kvp| {
		if let FileContent::Directory(children) = &mut kvp.content {
			children.par_sort_unstable_by(sort_children);
		}
	});
}

fn sort_children(p1: &Arc<VPath>, p2: &Arc<VPath>) -> std::cmp::Ordering {
	p1.cmp(p2)
}
