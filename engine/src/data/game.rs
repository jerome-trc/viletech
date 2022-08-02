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

use std::collections::HashMap;

use globset::Glob;
use kira::sound::static_sound::StaticSoundData;
use serde::Deserialize;

use crate::{
	ecs::Blueprint,
	game::{DamageType, Species},
	gfx::{Endoom, Palette},
};

pub type AssetId = usize;

pub struct Music(StaticSoundData);
pub struct Sound(StaticSoundData);

#[derive(Deserialize)]
pub struct Metadata {
	pub uuid: String,
	#[serde(default)]
	pub version: String,
	/// Display name presented to users.
	#[serde(default)]
	pub name: String,
	#[serde(default)]
	pub description: String,
	#[serde(default)]
	pub authors: Vec<String>,
	#[serde(default)]
	pub copyright: String,
	/// e.g., for if the author wants a link to a mod/WAD's homepage/forum post.
	#[serde(default)]
	pub links: Vec<String>,
}

impl Metadata {
	pub fn new(uuid: String, version: String) -> Self {
		Metadata {
			uuid,
			version,
			name: String::default(),
			description: String::default(),
			authors: Vec::<String>::default(),
			copyright: String::default(),
			links: Vec::<String>::default(),
		}
	}
}

/// Represents anything that the user added to their load order.
/// Acts as a namespace of sorts; for example, MAPINFO loaded as part of
/// a WAD will only apply to maps in that WAD.
pub struct Object {
	pub meta: Metadata,
	// Needed for the sim
	pub blueprints: Vec<Blueprint>,
	pub damage_types: Vec<DamageType>,
	pub species: Vec<Species>,
	// Client-only
	pub language: Vec<String>,
	pub palettes: Vec<Palette>,
	pub music: Vec<Music>,
	pub sounds: Vec<Sound>,
	pub endoom: Option<Endoom>,
}

#[derive(Default)]
pub struct DataCore {
	/// Element 0 should _always_ be the engine's own data, UUID "impure".
	/// Everything afterwards is ordered as per the user's specification.
	pub objects: Vec<Object>,
	pub asset_map: HashMap<String, (usize, AssetId)>,
	pub lump_map: HashMap<String, (usize, AssetId)>,
}

impl DataCore {
	/// Note: UUIDs are checked for an exact match.
	pub fn get_obj(&self, uuid: &str) -> Option<&Object> {
		for obj in &self.objects {
			if obj.meta.uuid == uuid {
				return Some(obj);
			}
		}

		None
	}

	/// Note: UUIDs are checked for an exact match.
	pub fn get_obj_mut(&mut self, uuid: &str) -> Option<&mut Object> {
		for obj in &mut self.objects {
			if obj.meta.uuid == uuid {
				return Some(obj);
			}
		}

		None
	}

	// Takes a glob pattern.
	pub fn obj_exists(&self, pattern: &str) -> Result<bool, globset::Error> {
		let glob = Glob::new(pattern)?.compile_matcher();

		for obj in &self.objects {
			if glob.is_match(&obj.meta.uuid) {
				return Ok(true);
			}
		}

		Ok(false)
	}
}
