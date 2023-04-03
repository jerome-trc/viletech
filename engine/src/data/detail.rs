//! Internal implementation details that don't belong anywhere else.

use std::hash::{Hash, Hasher};

use bevy_egui::egui;
use fasthash::SeaHasher;
use rayon::prelude::*;
use serde::Deserialize;

use crate::{VPath, VPathBuf};

use super::{Asset, Catalog, File, FileKind};

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
			if let FileKind::Directory(children) = &mut file.kind {
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
				continue; // No parent; `self.files[index]` is the root node.
			};

			let (&key, _) = self.files.get_index(index).unwrap();
			let parent = self.files.get_mut(&parent).unwrap();

			if let FileKind::Directory(children) = &mut parent.kind {
				children.push(key);
			} else {
				unreachable!()
			}
		}
	}

	pub(super) fn clean(&mut self) {
		self.nicknames.par_iter_mut().for_each(|mut kvp| {
			kvp.value_mut().retain(|weak| weak.upgrade().is_some());
		});

		self.nicknames.retain(|_, v| !v.is_empty());

		self.editor_nums.par_iter_mut().for_each(|mut kvp| {
			kvp.value_mut().retain(|weak| weak.upgrade().is_some());
		});

		self.editor_nums.retain(|_, v| !v.is_empty());

		self.spawn_nums.par_iter_mut().for_each(|mut kvp| {
			kvp.value_mut().retain(|weak| weak.upgrade().is_some());
		});

		self.spawn_nums.retain(|_, v| !v.is_empty());
	}

	pub(super) fn ui_vfs_impl(&self, ctx: &egui::Context, ui: &mut egui::Ui) {
		ui.heading("Virtual File System");

		egui::ScrollArea::vertical().show(ui, |ui| {
			for file in self.files.values() {
				let resp = ui.label(file.path_str());

				let resp = if resp.hovered() {
					resp.highlight()
				} else {
					resp
				};

				resp.on_hover_ui_at_pointer(|ui| {
					egui::Area::new("vtec_vfs_tt").show(ctx, |_| {
						Self::ui_file_tooltip(ui, file);
					});
				});
			}
		});
	}

	fn ui_file_tooltip(ui: &mut egui::Ui, file: &File) {
		match &file.kind {
			FileKind::Binary(bytes) => {
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
			FileKind::Text(string) => {
				ui.label("Text");
				ui.label(&format!("{} B", string.len()));
			}
			FileKind::Empty => {
				ui.label("Empty");
			}
			FileKind::Directory(dir) => {
				ui.label("Directory");

				if dir.len() == 1 {
					ui.label("1 child");
				} else {
					ui.label(&format!("{} children", dir.len()));
				}
			}
		}
	}

	pub(super) fn ui_assets_impl(&self, ctx: &egui::Context, ui: &mut egui::Ui) {
		ui.heading("Assets");

		egui::ScrollArea::vertical().show(ui, |ui| {
			for r in &self.assets {
				let resp = ui.label(r.value().id());

				let resp = if resp.hovered() {
					resp.highlight()
				} else {
					resp
				};

				resp.on_hover_ui_at_pointer(|ui| {
					egui::Area::new("vtec_asset_tt").show(ctx, |_| {
						ui.label(format!("{:?}", r.value().kind()));
					});
				});
			}
		});
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
#[repr(transparent)]
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
#[repr(transparent)]
pub(super) struct AssetKey(u64);

impl AssetKey {
	#[must_use]
	pub(super) fn new<A: Asset>(id: &str) -> Self {
		let mut hasher = SeaHasher::default();
		id.hash(&mut hasher);
		A::KIND.hash(&mut hasher);
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
	pub script_root: Option<VPathBuf>,
}
