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
	game::{DamageType, Species, SkillInfo},
	gfx::{Endoom, Palette, ColorMap},
	LevelCluster, level::Episode,
	LevelMetadata,
};

use super::asset::Asset;

/// `obj` corresponds to one of the elements in [`DataCore::objects`].
/// `elem` corresponds to an element in the relevant sub-vector of
/// the game data object. For "singleton" assets like palettes,
/// `elem` will always be 0.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AssetIndex {
	obj: usize,
	elem: usize
}

pub struct Music(StaticSoundData);
pub struct Sound(StaticSoundData);

/// Note that all user-facing string fields within may be IDs or expanded.
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
	/// Allow a package to link to its forum post/homepage/Discord server/etc.
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

/// Allows GDOs to define high-level, all-encompassing information
/// relevant to making games as opposed to mods.
pub struct GameInfo {
	steam_app_id: Option<u32>,
	discord_app_id: Option<String>
}

/// Determines the system used for loading assets from a mounted object.
pub enum GameDataKind {
	/// The file is read to determine what kind of assets are in it,
	/// and loading is handled accordingly.
	File,
	/// Every file in this WAD is loaded into an asset based on
	/// the kind of file it is.
	Wad,
	/// Assets are loaded from this archive/directory based on the
	/// manifests specified by the meta.toml file.
	Impure,
	/// Assets are loaded from this archive/directory based on the
	/// ZDoom sub-directory namespacing system. Sounds outside of `sounds/`,
	/// for example, don't get loaded at all.
	ZDoom,
	/// Assets are loaded from this archive/directory based on the
	/// Eternity Engine sub-directory namespacing system. Sounds outside of
	/// `sounds/`, for example, don't get loaded at all.
	Eternity
}

/// Represents anything that the user added to their load order.
/// Acts as a namespace of sorts; for example, MAPINFO loaded as part of
/// a WAD will only apply to maps in that WAD.
pub struct Object {
	pub meta: Metadata,
	pub kind: GameDataKind,
	// Needed for the sim
	pub blueprints: Vec<Blueprint>,
	pub clusters: Vec<LevelCluster>,
	pub damage_types: Vec<DamageType>,
	pub episodes: Vec<Episode>,
	pub levels: Vec<LevelMetadata>,
	pub skills: Vec<SkillInfo>,
	pub species: Vec<Species>,
	// Client-only
	pub language: Vec<String>,
	pub music: Vec<Music>,
	pub sounds: Vec<Sound>,
	pub colormap: Option<ColorMap>,
	pub endoom: Option<Endoom>,
	pub palette: Option<Palette>,
}

impl Object {
	pub fn new(metadata: Metadata, kind: GameDataKind) -> Self {
		Object {
			meta: metadata,
			kind,
			blueprints: Default::default(),
			damage_types: Default::default(),
			clusters: Default::default(),
			episodes: Default::default(),
			levels: Default::default(),
			skills: Default::default(),
			species: Default::default(),
			language: Default::default(),
			music: Default::default(),
			sounds: Default::default(),
			endoom: None,
			colormap: None,
			palette: None,
		}
	}

	pub fn clear(&mut self) {
		self.blueprints.clear();
		self.damage_types.clear();
		self.clusters.clear();
		self.episodes.clear();
		self.levels.clear();
		self.skills.clear();
		self.species.clear();
		self.language.clear();
		self.music.clear();
		self.sounds.clear();

		self.colormap.take();
		self.endoom.take();
		self.palette.take();
	}
}

#[derive(Default)]
pub struct DataCore {
	/// Element 0 should _always_ be the engine's own data, UUID "impure".
	/// Everything afterwards is ordered as per the user's specification.
	pub objects: Vec<Object>,
	/// Key structure: `gdo_uuid:domain.asset_id`.
	/// `gdo_uuid` will correspond to the mount point, and be something like
	/// `DOOM2`. `domain` will be something like `bp` or `mus`.
	pub asset_map: HashMap<String, AssetIndex>,
	/// Like [`DataCore::asset_map`], but without namespacing. Reflects the last thing
	/// under any given UUID in the load order. For use in interop, since, for
	/// example, GZDoom mods will expect that port's overlay/replacement system.
	pub lump_map: HashMap<String, AssetIndex>,

	pub editor_numbers: HashMap<u16, AssetIndex>,
	pub spawn_numbers: HashMap<u16, AssetIndex>,
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

	pub fn add<T: Asset>(&mut self, asset: T, obj_id: &str, asset_id: &str) {
		let obj_ndx = match self.objects.iter_mut().position(|o| o.meta.uuid == obj_id) {
			Some(o) => o,
			None => {
				// Caller should always pre-validate here
				panic!("Attempted to add asset under invalid UUID: {}", obj_id);
			}
		};

		let obj = &mut self.objects[obj_ndx];

		let asset_ndx = T::add_impl(obj, asset);
	
		let full_id = if T::DOMAIN_STRING.is_empty() {
			format!("{}:{}", obj_id, asset_id)
		} else {
			format!("{}:{}.{}", obj_id, T::DOMAIN_STRING, asset_id)
		};

		let ndx_pair = AssetIndex { obj: obj_ndx, elem: asset_ndx };

		self.asset_map.insert(full_id, ndx_pair.clone());
		self.lump_map.insert(asset_ndx.to_string(), ndx_pair);
	}

	pub fn get<T: Asset>(&self, id: &str) -> Option<&T> {
		let ndx_pair = &self.asset_map[id];
		T::get_impl(&self.objects[ndx_pair.obj], ndx_pair.elem)
	}
}
