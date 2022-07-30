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

use crate::{
	ecs::Blueprint,
	game::{DamageType, Species},
};
use kira::sound::static_sound::StaticSoundData;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct VersionedId {
	// Note to reader: probably not going to go to the same extent as npm
	// semantic versioning but there should be some versioning tied to this
	pub uuid: String,
}

#[derive(Default, Deserialize, PartialEq)]
pub enum DataKind {
	/// Unidentifiable, or an executable, or something else.
	#[default]
	None,
	/// e.g. a DEHACKED lump.
	Text,
	/// Self-explanatory.
	Wad,
	/// This is an archive (compressed under a supported format) or directory,
	/// with a top-level meta.toml file conforming to a specification.
	Impure,
	/// This is an archive (compressed under a supported format) or directory,
	/// with structure/lumps that identify it as being for ZDoom/GZDoom.
	GzDoom,
	/// This is an archive (compressed under a supported format) or directory,
	/// with structure/lumps that identify it as being for the Eternity Engine.
	Eternity,
}

#[derive(Deserialize)]
/// Every game data object (GDO) mounted to the VFS gets one of these.
pub struct Metadata {
	/// If this isn't given by an Impure metadata table, it's the name of the
	/// mounted file stem (e.g. gzdoom.pk3 becomes gzdoom, DOOM2.WAD becomes DOOM2).
	pub uuid: String,
	pub version: String,
	/// Display name presented to users.
	pub name: String,
	#[serde(alias = "description")]
	pub desc: String,
	#[serde(default)]
	pub authors: Vec<String>,
	#[serde(default)]
	pub copyright: String,
	/// e.g., for if the author wants a link to a mod/WAD's homepage/forum post.
	#[serde(default)]
	pub links: Vec<String>,
	#[serde(skip)]
	pub kind: DataKind,
	#[serde(default)]
	pub dependencies: Vec<VersionedId>,
	/// Incompatibilities are "soft"; the user is warned when trying to mingle
	/// incompatible game data objects but can still proceed as normal.
	#[serde(default)]
	pub incompatibilities: Vec<VersionedId>,
}

impl Metadata {
	pub fn from_uuid(uuid: String, kind: DataKind) -> Self {
		Metadata {
			uuid,
			version: String::default(),
			name: String::default(),
			desc: String::default(),
			authors: Vec::<String>::default(),
			copyright: String::default(),
			links: Vec::<String>::default(),
			kind,
			dependencies: Vec::<VersionedId>::default(),
			incompatibilities: Vec::<VersionedId>::default(),
		}
	}
}

pub type AssetId = usize;

#[derive(Default)]
pub struct DataCore {
	/// Represents all mounted game data objects. `[0]` should *always* be the
	/// engine's own game data. Everything afterwards is in a user-decided order.
	pub objects: Vec<Metadata>,
	/// Key structure:
	/// "package_uuid.domain.asset_key"
	/// Package UUID will either come from an Impure package metadata file,
	/// or from the archive/directory name minus the extension if it's not
	/// Impure data (e.g. "DOOM2" from "DOOM2.WAD", "gzdoom" from "gzdoom.pk3").
	/// Domain will be something like "textures" or "blueprints".
	/// Asset key is derived from the file name.
	/// Each value maps to an index in one of the asset vectors.
	pub asset_map: HashMap<String, AssetId>,
	/// Like `asset_map`, but without any namespacing. If a key such as "MAP01"
	/// appears twice (e.g. once from DOOM2.WAD and again from my_house.wad) it
	/// will clobber whatever came before it.
	pub lump_map: HashMap<String, AssetId>,

	// Needed for the sim
	pub language: Vec<String>,
	pub blueprints: Vec<Blueprint>,
	pub damage_types: Vec<DamageType>,
	pub species: Vec<Species>,
	// Client-only
	pub music: Vec<StaticSoundData>,
	pub sounds: Vec<StaticSoundData>,
}

impl DataCore {
	pub fn is_mounted(&self, pattern: &str) -> Result<bool, regex::Error> {
		let regex = Regex::new(pattern)?;

		for obj in &self.objects {
			if regex.is_match(&obj.uuid) {
				return Ok(true);
			}
		}

		Ok(false)
	}
}
