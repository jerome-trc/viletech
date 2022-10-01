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

use std::{collections::HashMap, fmt, hash::Hash, path::PathBuf};

use fasthash::metro;
use globset::Glob;
use kira::sound::static_sound::StaticSoundData;
use log::{error, warn};
use serde::Deserialize;
use zsparse::err::ParsingErrorLevel as ZsParsingErrorLevel;

use crate::{
	ecs::Blueprint,
	game::{ActorStateMachine, DamageType, SkillInfo, Species},
	gfx::doom::{ColorMap, Endoom, Palette},
	level::Episode,
	vfs::VirtualFs,
	zscript, LevelCluster, LevelMetadata, VfsHandle,
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
	#[must_use]
	fn from_id_pair<A: Asset>(namespace_id: &str, asset_id: &str) -> Self {
		let mut ret = metro::hash64(namespace_id);
		ret ^= metro::hash64(A::DOMAIN_STRING);
		ret ^= metro::hash64(asset_id);

		Self(ret)
	}

	fn from_id<A: Asset>(string: &str) -> Result<Self, AssetError> {
		let mut split = string.split(':');

		let nsid = split.next().ok_or(AssetError::HashEmptyString)?;
		let aid = split.next().ok_or(AssetError::IdMissingPostfix)?;

		Ok(Self::from_id_pair::<A>(nsid, aid))
	}
}

#[derive(Debug)]
pub enum AssetError {
	HashEmptyString,
	IdMissingPostfix,
	IdNotFound,
	IdClobber,
	NamespaceNotFound,
}

impl std::error::Error for AssetError {}

impl fmt::Display for AssetError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::HashEmptyString => {
				write!(f, "Cannot form an asset hash from an empty ID string.")
			}
			Self::IdMissingPostfix => {
				write!(f, "Asset ID is malformed, and lacks a postfix.")
			}
			Self::IdNotFound => {
				write!(f, "The given asset ID did not match any existing asset.")
			}
			Self::IdClobber => {
				write!(f, "Attempted to overwrite an existing asset ID map key.")
			}
			Self::NamespaceNotFound => {
				write!(
					f,
					"The given namespace ID did not match any existing game data object's UUID."
				)
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
	#[serde(skip)]
	pub kind: GameDataKind,
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
	pub fn new(uuid: String, version: String, kind: GameDataKind) -> Self {
		Metadata {
			uuid,
			version,
			kind,
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
	/// manifest specified by the meta.toml file.
	Impure { manifest: PathBuf },
	/// Assets are loaded from this archive/directory based on the
	/// ZDoom sub-directory namespacing system. Sounds outside of `sounds/`,
	/// for example, don't get loaded at all.
	ZDoom,
	/// Assets are loaded from this archive/directory based on the
	/// Eternity Engine sub-directory namespacing system. Sounds outside of
	/// `sounds/`, for example, don't get loaded at all.
	Eternity,
}

impl Default for GameDataKind {
	fn default() -> Self {
		Self::Impure {
			manifest: PathBuf::default(),
		}
	}
}

/// Represents anything that the user added to their load order.
/// Comes with a certain degree of compartmentalization: for example,
/// MAPINFO loaded as part of a WAD will only apply to maps in that WAD.
pub struct Namespace<'lua> {
	pub meta: Metadata,
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
	#[must_use]
	pub fn new(metadata: Metadata) -> Self {
		Namespace {
			meta: metadata,

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
	#[must_use]
	pub fn get_namespace(&self, uuid: &str) -> Option<&Namespace> {
		self.namespaces.iter().find(|ns| ns.meta.uuid == uuid)
	}

	/// Note: UUIDs are checked for an exact match.
	#[must_use]
	pub fn get_namespace_mut(&'lua mut self, uuid: &str) -> Option<&mut Namespace> {
		self.namespaces.iter_mut().find(|ns| ns.meta.uuid == uuid)
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

	pub fn add<'s, A: Asset>(
		&mut self,
		asset: A,
		namespace_id: &'s str,
		asset_id: &'s str,
	) -> Result<(), AssetError> {
		let ns_index = self
			.namespaces
			.iter_mut()
			.position(|o| o.meta.uuid == namespace_id)
			.ok_or(AssetError::NamespaceNotFound)?;

		let namespace = &mut self.namespaces[ns_index];
		let asset_index = A::add_impl(namespace, asset);
		let hash = AssetHash::from_id_pair::<A>(namespace_id, asset_id);

		let ndx_pair = AssetIndex {
			namespace: ns_index,
			element: asset_index,
		};

		if self.asset_map.contains_key(&hash) {
			return Err(AssetError::IdClobber);
		}

		self.asset_map.insert(hash, ndx_pair);
		let lump_name = asset_id.split('.').next().unwrap();
		let strlen = lump_name.chars().count().min(8);
		self.lump_map
			.insert(lump_name[..strlen].to_string(), ndx_pair);

		Ok(())
	}

	#[must_use]
	pub fn get<A: Asset>(&self, index: AssetIndex) -> Option<&A> {
		A::get_impl(&self.namespaces[index.namespace], index.element)
	}

	pub fn lookup<A: Asset>(&self, id: &str) -> Result<&A, AssetError> {
		let hash = AssetHash::from_id::<A>(id)?;
		let ipair = self.asset_map.get(&hash).ok_or(AssetError::IdNotFound)?;
		Ok(A::get_impl(&self.namespaces[ipair.namespace], ipair.element).unwrap())
	}

	pub fn populate(&mut self, mut metas: Vec<Metadata>, vfs: &VirtualFs) {
		debug_assert!(self.namespaces.is_empty());
		debug_assert!(!metas.is_empty());
		debug_assert!(metas[0].uuid == "impure");

		for (_index, meta) in metas.drain(..).enumerate() {
			let _handle = vfs
				.lookup(&meta.uuid)
				.expect("Failed to find a namespace's VFS handle for data core population.");

			match meta.kind {
				GameDataKind::ZDoom => {
					// ???
				}
				GameDataKind::Wad => {
					// ???
				}
				GameDataKind::Impure { manifest: _ } => {
					// ???
				}
				GameDataKind::File => {
					// ???
				}
				GameDataKind::Eternity => {
					// ???
				}
			};
		}
	}
}

impl DataCore<'_> {
	fn try_load_zscript(namespace: &mut Namespace, handle: &VfsHandle) {
		let parse_out = zscript::parse(handle.clone());
		let nsid = &namespace.meta.uuid;

		let any_parse_errors = parse_out
			.errors
			.iter()
			.any(|e| e.level == ZsParsingErrorLevel::Error);

		if any_parse_errors {
			error!(
				"{} errors during ZScript transpile, parse phase: {}",
				parse_out.errors.len(),
				nsid
			);
		}

		for err in parse_out
			.errors
			.iter()
			.filter(|e| e.level == ZsParsingErrorLevel::Error)
		{
			let file = &parse_out.files[err.main_spans[0].get_file()];
			error!("{}", zscript::prettify_error(nsid, file, err));
		}

		let any_parse_warnings = parse_out
			.errors
			.iter()
			.any(|e| e.level == ZsParsingErrorLevel::Warning);

		if any_parse_warnings {
			warn!(
				"{} warnings during ZScript transpile, parse phase: {}",
				parse_out.errors.len(),
				nsid
			);
		}

		for warn in parse_out
			.errors
			.iter()
			.filter(|e| e.level == ZsParsingErrorLevel::Warning)
		{
			let file = &parse_out.files[warn.main_spans[0].get_file()];
			warn!("{}", zscript::prettify_error(nsid, file, warn));
		}

		if any_parse_errors {
			return;
		}

		todo!()
	}
}
