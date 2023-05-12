//! Internal implementation details that don't belong anywhere else.

use std::{
	any::TypeId,
	hash::{Hash, Hasher},
};

use bevy_egui::egui::{self, TextStyle};
use fasthash::SeaHasher;
use regex::Regex;
use serde::Deserialize;

use crate::{VPath, VPathBuf};

use super::{dobj::DATUM_TYPE_NAMES, Catalog, Datum};

/// State storage for the catalog's developer GUI.
#[derive(Debug)]
pub(super) struct DeveloperGui {
	search_buf: String,
	search: Regex,
}

impl DeveloperGui {
	fn update_search_regex(&mut self) {
		let mut esc = regex::escape(&self.search_buf);
		esc.insert_str(0, "(?i)"); // Case insensitivity
		self.search = Regex::new(&esc).unwrap();
	}
}

impl Default for DeveloperGui {
	fn default() -> Self {
		Self {
			search_buf: String::new(),
			search: Regex::new("").unwrap(),
		}
	}
}

impl Catalog {
	pub(super) fn ui_impl(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
		ui.heading("Game Data");

		ui.horizontal(|ui| {
			ui.label("Search");

			if ui.text_edit_singleline(&mut self.gui.search_buf).changed() {
				self.gui.update_search_regex();
			}

			if ui.button("Clear").clicked() {
				self.gui.search_buf.clear();
				self.gui.update_search_regex();
			}
		});

		egui::ScrollArea::vertical()
			.auto_shrink([false; 2])
			.show_rows(
				ui,
				ui.text_style_height(&TextStyle::Body),
				self.objs.len(),
				|ui, row_range| {
					for (_, (_, store)) in self
						.mounts
						.iter()
						.flat_map(|mnt| &mnt.objs)
						.enumerate()
						.skip(row_range.start)
					{
						let id = store.id();

						if !self.gui.search.is_match(id) {
							continue;
						}

						let resp = ui.label(id);

						let resp = if resp.hovered() {
							resp.highlight()
						} else {
							resp
						};

						resp.on_hover_ui_at_pointer(|ui| {
							egui::Area::new("vtec_datum_tt").show(ctx, |_| {
								let &type_name =
									DATUM_TYPE_NAMES.get(&store.datum_typeid()).unwrap();
								ui.label(type_name);
							});
						});
					}
				},
			);
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

/// Field `1` is a hash of the datum's ID string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct DatumKey(TypeId, u64);

impl DatumKey {
	#[must_use]
	pub(super) fn new<D: Datum>(id: &str) -> Self {
		let mut hasher = SeaHasher::default();
		id.hash(&mut hasher);
		Self(TypeId::of::<D>(), hasher.finish())
	}
}

slotmap::new_key_type! {
	/// See [`crate::data::Mount`].
	pub(super) struct DatumSlotKey;
}

/// Intermediate format for parsing parts of [`MountMeta`] from `meta.toml` files.
///
/// [`MountMeta`]: super::MountMeta
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
	pub vzscript: Option<MountMetaIngestVzs>,
}

#[derive(Debug, Deserialize)]
pub(super) struct MountMetaIngestVzs {
	pub folder: VPathBuf,
	pub namespace: Option<String>,
	pub version: String,
}

/// For representing all the possible endings for most load operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub(super) enum Outcome<T, E> {
	Cancelled,
	None,
	Err(E),
	Ok(T),
}
