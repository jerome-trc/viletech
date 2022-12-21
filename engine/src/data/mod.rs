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

use doom_front::zscript::err::ParsingErrorLevel as ZsParseIssueLevel;
use globset::Glob;
use kira::sound::static_sound::StaticSoundData;
use log::{error, warn};
use regex::Regex;
use serde::Deserialize;

use crate::{
	ecs::Blueprint,
	game::{ActorStateMachine, DamageType, SkillInfo, Species},
	gfx::doom::{ColorMap, Endoom, Palette},
	level::{self, Cluster, Episode},
	newtype,
	vfs::{FileRef, ImpureVfs, VirtualFs},
	zscript,
};

pub use asset::{
	Error as AssetError, Flags as AssetFlags, Handle as AssetHandle, Wrapper as AssetWrapper,
};

use self::asset::Asset;

newtype!(pub struct Music(StaticSoundData));
newtype!(pub struct Sound(StaticSoundData));

/// Note that all user-facing string fields within may be IDs or expanded.
pub struct GameDataMeta {
	pub id: String,
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
	pub fn new(id: String, version: String, kind: GameDataKind) -> Self {
		GameDataMeta {
			id,
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
			id: toml.id,
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
	pub id: String,
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
	_steam_app_id: Option<u32>,
	_discord_app_id: Option<String>,
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
	/// Element 0 should _always_ be the engine's own data, ID "impure".
	/// Everything afterwards is ordered as per the user's specification.
	pub namespaces: Vec<Namespace>,
	/// IDs are derived from virtual file system paths. The asset ID for
	/// `/impure/textures/default.png` is exactly the same string; the ID for
	/// blueprint `Imp` defined in file `/impure/blueprints/imp.zs` is
	/// `/impure/blueprints/imp/Imp`, since multiple classes can be defined in
	/// one ZScript translation unit.
	pub asset_map: HashMap<String, AssetHandle>,
	/// Doom and its source ports work on a simple data replacement system; a name
	/// points to the last map/texture/song/etc. loaded by that name. Impure offers
	/// full namespacing via asset IDs, but also has the concept of a short asset ID (SAID)
	/// which mimics Doom's behaviour for interop purposes.
	pub short_id_maps: [HashMap<String, AssetHandle>; asset::COLLECTION_COUNT],

	pub editor_numbers: HashMap<u16, AssetHandle>,
	pub spawn_numbers: HashMap<u16, AssetHandle>,
}

// Public interface.
impl DataCore {
	#[must_use]
	pub fn get_namespace(&self, id: &str) -> Option<&Namespace> {
		self.namespaces.iter().find(|ns| ns.meta.id == id)
	}

	#[must_use]
	pub fn get_namespace_glob(&self, glob: Glob) -> Option<&Namespace> {
		let matcher = glob.compile_matcher();
		self.namespaces
			.iter()
			.find(|ns| matcher.is_match(&ns.meta.id))
	}

	#[must_use]
	pub fn get_namespace_regex(&self, regex: Regex) -> Option<&Namespace> {
		self.namespaces
			.iter()
			.find(|ns| regex.is_match(&ns.meta.id))
	}

	#[must_use]
	pub fn get_namespace_mut(&mut self, id: &str) -> Option<&mut Namespace> {
		self.namespaces.iter_mut().find(|ns| ns.meta.id == id)
	}

	#[must_use]
	pub fn get_namespace_mut_glob(&mut self, glob: Glob) -> Option<&mut Namespace> {
		let matcher = glob.compile_matcher();
		self.namespaces
			.iter_mut()
			.find(|ns| matcher.is_match(&ns.meta.id))
	}

	#[must_use]
	pub fn get_namespace_mut_regex(&mut self, regex: Regex) -> Option<&mut Namespace> {
		self.namespaces
			.iter_mut()
			.find(|ns| regex.is_match(&ns.meta.id))
	}

	#[must_use]
	pub fn namespace_exists(&self, id: &str) -> bool {
		self.namespaces.iter().any(|ns| ns.meta.id == id)
	}

	#[must_use]
	pub fn namespace_exists_glob(&self, glob: Glob) -> bool {
		let matcher = glob.compile_matcher();
		self.namespaces
			.iter()
			.any(|ns| matcher.is_match(&ns.meta.id))
	}

	#[must_use]
	pub fn namespace_exists_regex(&self, regex: Regex) -> bool {
		self.namespaces.iter().any(|ns| regex.is_match(&ns.meta.id))
	}

	pub fn add<A: Asset>(
		&mut self,
		asset: A,
		namespace: usize,
		id: &str,
		short_id: &str,
	) -> Result<(), AssetError> {
		let ns_index = namespace;
		let namespace = &mut self.namespaces[ns_index];
		let coll = A::collection_mut(namespace);
		let asset_index = coll.len();

		coll.push(AssetWrapper {
			inner: asset,
			_flags: AssetFlags::empty(),
		});

		let ndx_pair = AssetHandle {
			namespace: ns_index,
			element: asset_index,
		};

		if self.asset_map.contains_key(id) {
			return Err(AssetError::IdClobber);
		}

		self.asset_map.insert(id.to_string(), ndx_pair);
		self.short_id_maps[A::INDEX].insert(short_id.to_string(), ndx_pair);

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
		let handle = self.asset_map.get(id).ok_or(AssetError::IdNotFound)?;
		let collection = A::collection(&self.namespaces[handle.namespace]);

		match collection.get(handle.element) {
			Some(r) => Ok(&r.inner),
			None => Err(AssetError::IdNotFound),
		}
	}

	/// Tries to find an asset by its short ID (no namespace qualification).
	pub fn lookup_global<A: Asset>(&self, short_id: &str) -> Result<&A, AssetError> {
		let handle = self.short_id_maps[A::INDEX]
			.get(short_id)
			.ok_or(AssetError::IdNotFound)?;
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
		debug_assert!(metas[0].id == "impure");

		for (_index, meta) in metas.drain(..).enumerate() {
			let _fref = vfs
				.lookup(&meta.id)
				.expect("Failed to find a namespace's VFS fileref for data core population.");

			let kind = vfs.gamedata_kind(&meta.id);

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
	#[allow(dead_code)]
	fn try_load_zscript(namespace: &mut Namespace, file: &FileRef) {
		let parse_out = zscript::parse(file.clone());
		let nsid = &namespace.meta.id;

		let any_parse_errors = parse_out
			.issues
			.iter()
			.any(|e| e.level == ZsParseIssueLevel::Error);

		if any_parse_errors {
			error!(
				"{} errors during ZScript transpile, parse phase: {}",
				parse_out.issues.len(),
				nsid
			);
		}

		for issue in parse_out
			.issues
			.iter()
			.filter(|e| e.level == ZsParseIssueLevel::Error)
		{
			let file = &parse_out.files[issue.main_spans[0].get_file()];
			error!("{}", zscript::prettify_parse_issue(nsid, file, issue));
		}

		let any_parse_warnings = parse_out
			.issues
			.iter()
			.any(|e| e.level == ZsParseIssueLevel::Warning);

		if any_parse_warnings {
			warn!(
				"{} warnings during ZScript transpile, parse phase: {}",
				parse_out.issues.len(),
				nsid
			);
		}

		for warn in parse_out
			.issues
			.iter()
			.filter(|e| e.level == ZsParseIssueLevel::Warning)
		{
			let file = &parse_out.files[warn.main_spans[0].get_file()];
			warn!("{}", zscript::prettify_parse_issue(nsid, file, warn));
		}

		if any_parse_errors {
			return;
		}

		todo!()
	}
}
