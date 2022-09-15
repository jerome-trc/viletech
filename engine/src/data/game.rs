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

use std::{collections::HashMap, fmt, hash::Hash};

use fasthash::metro;
use globset::Glob;
use kira::sound::static_sound::StaticSoundData;
use serde::Deserialize;

use crate::{
	ecs::Blueprint,
	game::{DamageType, SkillInfo, Species, ActorStateMachine},
	gfx::doom::{ColorMap, Endoom, Palette},
	level::Episode,
	LevelCluster, LevelMetadata,
};

use super::asset::Asset;

/// `namespace` corresponds to one of the elements in [`DataCore::namespaces`].
/// `elem` corresponds to an element in the relevant sub-vector of the namespace.
/// For "singleton" assets like palettes, `elem` will always be 0.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AssetIndex {
	namespace: usize,
	element: usize,
}

/// Wraps a hash, generated from an asset ID string, used as a key in [`DataCore::asset_map`].
/// Scripts call asset-domain-specific functions and pass in strings like
/// `"namespace:sound_id"`, so mixing in the domain's string (e.g. "snd") ensures
/// uniqueness in one hash map amongst other assets.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AssetHash(pub(crate) u64);

impl AssetHash {
	fn from_id_pair<A: Asset>(namespace_id: &str, asset_id: &str) -> Self {
		let mut ret = metro::hash64(namespace_id);
		ret ^= metro::hash64(A::DOMAIN_STRING);
		ret ^= metro::hash64(asset_id);

		Self(ret)
	}

	fn from_id<A: Asset>(string: &str) -> Result<Self, AssetIdError> {
		let mut split = string.split(':');

		let nsid = split.next().ok_or(AssetIdError::EmptyString)?;
		let aid = split.next().ok_or(AssetIdError::MissingPostfix)?;

		Ok(Self::from_id_pair::<A>(nsid, aid))
	}
}

#[derive(Debug)]
pub enum AssetIdError {
	EmptyString,
	MissingPostfix,
	NotFound,
}

impl std::error::Error for AssetIdError {}

impl fmt::Display for AssetIdError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::EmptyString => {
				write!(f, "Cannot form an asset hash from an empty ID string.")
			}
			Self::MissingPostfix => {
				write!(f, "Asset ID is malformed, and lacks a postfix.")
			}
			Self::NotFound => {
				write!(f, "The given asset ID did not match any existing asset.")
			}
		}
	}
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

/// Allows game data objects to define high-level, all-encompassing information
/// relevant to making games as opposed to mods.
pub struct GameInfo {
	steam_app_id: Option<u32>,
	discord_app_id: Option<String>,
}

/// Determines the system used for loading assets from a mounted game data object.
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
	Eternity,
}

/// Represents anything that the user added to their load order.
/// Comes with a certain degree of compartmentalization:
/// for example,  Acts as a namespace of sorts; for example,
/// MAPINFO loaded as part of a WAD will only apply to maps in that WAD.
pub struct Namespace<'lua> {
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
	pub state_machines: Vec<ActorStateMachine<'lua>>,
	// Client-only
	pub language: Vec<String>,
	pub music: Vec<Music>,
	pub sounds: Vec<Sound>,
	pub colormap: Option<ColorMap>,
	pub endoom: Option<Endoom>,
	pub palette: Option<Palette>,
}

impl<'lua> Namespace<'lua> {
	pub fn new(metadata: Metadata, kind: GameDataKind) -> Self {
		Namespace {
			meta: metadata,
			kind,

			blueprints: Default::default(),
			clusters: Default::default(),
			damage_types: Default::default(),
			episodes: Default::default(),
			levels: Default::default(),
			skills: Default::default(),
			species: Default::default(),
			state_machines: Default::default(),

			language: Default::default(),
			music: Default::default(),
			sounds: Default::default(),
			colormap: None,
			endoom: None,
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
pub struct DataCore<'lua> {
	/// Element 0 should _always_ be the engine's own data, UUID "impure".
	/// Everything afterwards is ordered as per the user's specification.
	pub namespaces: Vec<Namespace<'lua>>,
	pub asset_map: HashMap<AssetHash, AssetIndex>,
	/// Like [`DataCore::asset_map`], but without namespacing. Reflects the last thing
	/// under any given UUID in the load order. For use in interop, since, for
	/// example, GZDoom mods will expect that port's overlay/replacement system.
	pub lump_map: HashMap<String, AssetIndex>,

	pub editor_numbers: HashMap<u16, AssetIndex>,
	pub spawn_numbers: HashMap<u16, AssetIndex>,
}

// Public interface.
impl<'lua> DataCore<'lua> {
	/// Note: UUIDs are checked for an exact match.
	pub fn get_namespace(&self, uuid: &str) -> Option<&Namespace> {
		for namespace in &self.namespaces {
			if namespace.meta.uuid == uuid {
				return Some(namespace);
			}
		}

		None
	}

	/// Note: UUIDs are checked for an exact match.
	pub fn get_namespace_mut(&'lua mut self, uuid: &str) -> Option<&mut Namespace> {
		for namespace in &mut self.namespaces {
			if namespace.meta.uuid == uuid {
				return Some(namespace);
			}
		}

		None
	}

	// Takes a glob pattern.
	pub fn namespace_exists(&self, pattern: &str) -> Result<bool, globset::Error> {
		let glob = Glob::new(pattern)?.compile_matcher();

		for namespace in &self.namespaces {
			if glob.is_match(&namespace.meta.uuid) {
				return Ok(true);
			}
		}

		Ok(false)
	}

	pub fn add<'s, A: Asset>(&mut self, asset: A, namespace_id: &'s str, asset_id: &'s str) {
		let ns_index = match self
			.namespaces
			.iter_mut()
			.position(|o| o.meta.uuid == namespace_id)
		{
			Some(o) => o,
			None => {
				// Caller should always pre-validate here
				panic!(
					"Attempted to add asset under invalid UUID: {}",
					namespace_id
				);
			}
		};

		let namespace = &mut self.namespaces[ns_index];
		let asset_index = A::add_impl(namespace, asset);
		let hash = AssetHash::from_id_pair::<A>(namespace_id, asset_id);

		let ndx_pair = AssetIndex {
			namespace: ns_index,
			element: asset_index,
		};

		self.asset_map.insert(hash, ndx_pair);
		self.lump_map.insert(asset_index.to_string(), ndx_pair);
	}

	pub fn get<A: Asset>(&self, index: AssetIndex) -> Option<&A> {
		A::get_impl(&self.namespaces[index.namespace], index.element)
	}

	pub fn lookup<A: Asset>(&self, id: &str) -> Result<&A, AssetIdError> {
		let hash = AssetHash::from_id::<A>(id)?;
		let ipair = self.asset_map.get(&hash).ok_or(AssetIdError::NotFound)?;
		Ok(A::get_impl(&self.namespaces[ipair.namespace], ipair.element).unwrap())
	}
}
