//! "Game data" means audio, graphics, levels, ECS definitions, localization
//! strings, and structures for representing the packages they come in.

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

pub mod asset;

use std::{collections::HashMap, path::PathBuf};

use globset::Glob;
use kira::sound::static_sound::StaticSoundData;
use log::{error, warn};
use serde::Deserialize;

use crate::{
	ecs::Blueprint,
	game::{ActorStateMachine, DamageType, SkillInfo, Species},
	gfx::doom::{ColorMap, Endoom, Palette},
	level::{self, Cluster, Episode},
	newtype,
	vfs::{ImpureVfs, VirtualFs},
	zscript::{self, parser::error::ParsingErrorLevel as ZsParseErrorLevel},
	VfsHandle,
};

pub use asset::{
	Error as AssetError, Flags as AssetFlags, Handle as AssetHandle, Wrapper as AssetWrapper,
};

use self::asset::{Asset, IdHash};

newtype!(pub struct Music(StaticSoundData));
newtype!(pub struct Sound(StaticSoundData));

/// Note that all user-facing string fields within may be IDs or expanded.
pub struct GameDataMeta {
	pub uuid: String,
	pub kind: GameDataKind,
	pub version: String,
	/// Display name presented to users.
	pub name: String,
	pub description: String,
	pub authors: Vec<String>,
	pub copyright: String,
	/// Allow a package to link to its forum post/homepage/Discord server/etc.
	pub links: Vec<String>,
}

impl GameDataMeta {
	pub fn new(uuid: String, version: String, kind: GameDataKind) -> Self {
		GameDataMeta {
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

	pub fn from_toml(toml: MetaToml, manifest: PathBuf) -> Self {
		Self {
			uuid: toml.uuid,
			kind: GameDataKind::Impure { manifest },
			version: toml.version,
			name: toml.name,
			description: toml.description,
			authors: toml.authors,
			copyright: toml.copyright,
			links: toml.links,
		}
	}
}

#[derive(Default, Deserialize)]
pub struct MetaToml {
	pub uuid: String,
	#[serde(default)]
	pub version: String,
	#[serde(default)]
	pub name: String,
	#[serde(default)]
	pub description: String,
	#[serde(default)]
	pub authors: Vec<String>,
	#[serde(default)]
	pub copyright: String,
	#[serde(default)]
	pub links: Vec<String>,
	#[serde(default)]
	pub manifest: Option<PathBuf>,
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
	Wad { internal: bool },
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

pub type AssetVec<A> = Vec<asset::Wrapper<A>>;

/// Represents anything that the user added to their load order.
/// Comes with a certain degree of compartmentalization: for example,
/// MAPINFO loaded as part of a WAD will only apply to maps in that WAD.
pub struct Namespace {
	pub meta: GameDataMeta,
	// Needed for the sim
	pub blueprints: AssetVec<Blueprint>,
	pub clusters: AssetVec<Cluster>,
	pub damage_types: AssetVec<DamageType>,
	pub episodes: AssetVec<Episode>,
	pub levels: AssetVec<level::Metadata>,
	pub skills: AssetVec<SkillInfo>,
	pub species: AssetVec<Species>,
	pub state_machines: AssetVec<ActorStateMachine>,
	// Client-only
	pub language: AssetVec<String>,
	pub music: AssetVec<Music>,
	pub sounds: AssetVec<Sound>,
	pub colormap: AssetVec<ColorMap>,
	pub endoom: AssetVec<Endoom>,
	pub palette: AssetVec<Palette>,
}

impl Namespace {
	#[must_use]
	pub fn new(metadata: GameDataMeta) -> Self {
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
			colormap: Default::default(),
			endoom: Default::default(),
			palette: Default::default(),
		}
	}
}

#[derive(Default)]
pub struct DataCore {
	/// Element 0 should _always_ be the engine's own data, UUID "impure".
	/// Everything afterwards is ordered as per the user's specification.
	pub namespaces: Vec<Namespace>,
	pub asset_map: HashMap<IdHash, AssetHandle>,
	/// Like [`DataCore::asset_map`], but without namespacing. Reflects the last thing
	/// under any given UUID in the load order. For use in interop, since, for
	/// example, GZDoom mods will expect that port's overlay/replacement system.
	pub lump_map: HashMap<String, AssetHandle>,

	pub editor_numbers: HashMap<u16, AssetHandle>,
	pub spawn_numbers: HashMap<u16, AssetHandle>,
}

// Public interface.
impl DataCore {
	/// Note: UUIDs are checked for an exact match.
	#[must_use]
	pub fn get_namespace(&self, uuid: &str) -> Option<&Namespace> {
		self.namespaces.iter().find(|ns| ns.meta.uuid == uuid)
	}

	/// Note: UUIDs are checked for an exact match.
	#[must_use]
	pub fn get_namespace_mut(&mut self, uuid: &str) -> Option<&mut Namespace> {
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
			.iter()
			.position(|o| o.meta.uuid == namespace_id)
			.ok_or(AssetError::NamespaceNotFound)?;

		let namespace = &mut self.namespaces[ns_index];
		let coll = A::collection_mut(namespace);
		let asset_index = coll.len();
		let id_hash = IdHash::from_id_pair::<A>(namespace_id, asset_id);

		coll.push(AssetWrapper {
			inner: asset,
			hash: id_hash.0,
			flags: AssetFlags::empty(),
		});

		let ndx_pair = AssetHandle {
			namespace: ns_index,
			element: asset_index,
			hash: id_hash.0,
		};

		if self.asset_map.contains_key(&id_hash) {
			return Err(AssetError::IdClobber);
		}

		self.asset_map.insert(id_hash, ndx_pair);
		let lump_name = asset_id.split('.').next().unwrap();
		let strlen = lump_name.chars().count().min(8);
		self.lump_map
			.insert(lump_name[..strlen].to_string(), ndx_pair);

		Ok(())
	}

	#[must_use]
	pub fn try_get<A: Asset>(&self, handle: AssetHandle) -> Option<&A> {
		let collection = A::collection(&self.namespaces[handle.namespace]);

		match collection.get(handle.element) {
			Some(r) => Some(&r.inner),
			None => None,
		}
	}

	#[must_use]
	pub fn get<A: Asset>(&self, handle: AssetHandle) -> &A {
		let collection = A::collection(&self.namespaces[handle.namespace]);
		&collection[handle.element]
	}

	pub fn lookup<A: Asset>(&self, id: &str) -> Result<&A, AssetError> {
		let hash = IdHash::from_id::<A>(id)?;
		let handle = self.asset_map.get(&hash).ok_or(AssetError::IdNotFound)?;
		let collection = A::collection(&self.namespaces[handle.namespace]);

		match collection.get(handle.element) {
			Some(r) => Ok(&r.inner),
			None => Err(AssetError::IdNotFound),
		}
	}

	/// This function expects:
	/// - That `self.namespaces` is empty.
	/// - That `metas[0]` is the parsed metadata for the Impure data package.
	pub fn populate(&mut self, mut metas: Vec<MetaToml>, vfs: &VirtualFs) {
		debug_assert!(self.namespaces.is_empty());
		debug_assert!(!metas.is_empty());
		debug_assert!(metas[0].uuid == "impure");

		for (_index, meta) in metas.drain(..).enumerate() {
			let _handle = vfs
				.lookup(&meta.uuid)
				.expect("Failed to find a namespace's VFS handle for data core population.");

			let kind = vfs.gamedata_kind(&meta.uuid);

			match kind {
				GameDataKind::ZDoom => {
					// ???
				}
				GameDataKind::Wad { .. } => {
					// ???
				}
				GameDataKind::Impure { .. } => {
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

impl DataCore {
	fn try_load_zscript(namespace: &mut Namespace, handle: &VfsHandle) {
		let parse_out = zscript::parse(handle.clone());
		let nsid = &namespace.meta.uuid;

		let any_parse_errors = parse_out
			.errors
			.iter()
			.any(|e| e.level == ZsParseErrorLevel::Error);

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
			.filter(|e| e.level == ZsParseErrorLevel::Error)
		{
			let file = &parse_out.files[err.main_spans[0].get_file()];
			error!("{}", zscript::prettify_error(nsid, file, err));
		}

		let any_parse_warnings = parse_out
			.errors
			.iter()
			.any(|e| e.level == ZsParseErrorLevel::Warning);

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
			.filter(|e| e.level == ZsParseErrorLevel::Warning)
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
