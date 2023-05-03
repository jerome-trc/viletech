//! Internal implementation details that don't belong anywhere else.

use std::{
	any::TypeId,
	hash::{Hash, Hasher},
};

use bevy_egui::egui;
use fasthash::SeaHasher;
use serde::Deserialize;

use crate::{VPath, VPathBuf};

use super::{Asset, Catalog};

impl Catalog {
	pub(super) fn ui_assets_impl(&self, ctx: &egui::Context, ui: &mut egui::Ui) {
		ui.heading("Assets");

		egui::ScrollArea::vertical().show(ui, |ui| {
			for (_, mount) in &self.mounts {
				for (_, asset) in &mount.assets {
					let resp = ui.label(&asset.header().id);

					let resp = if resp.hovered() {
						resp.highlight()
					} else {
						resp
					};

					resp.on_hover_ui_at_pointer(|ui| {
						egui::Area::new("vtec_asset_tt").show(ctx, |_| {
							ui.label(format!("{:?}", asset.type_name()));
						});
					});
				}

				ui.separator();
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
/// hashing the ID string (or part of the ID string, in the case of the nickname
/// lookup table) and then the type ID, in that order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct AssetKey(u64);

impl AssetKey {
	#[must_use]
	pub(super) fn new<A: Asset>(id: &str) -> Self {
		let mut hasher = SeaHasher::default();
		id.hash(&mut hasher);
		TypeId::of::<A>().hash(&mut hasher);
		Self(hasher.finish())
	}
}

slotmap::new_key_type! {
	pub(super) struct MountSlotKey;
	/// See [`crate::data::Mount`].
	pub(super) struct AssetSlotKey;
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
