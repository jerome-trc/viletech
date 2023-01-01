//! "Game data" means audio, graphics, levels, ECS definitions, localization
//! strings, and structures for representing the packages they come in.

pub mod asset;

use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};

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
	lith, newtype,
	utils::string,
	vfs::{FileRef, VirtualFs, VirtualFsExt},
	zscript,
};

pub use asset::{
	Error as AssetError, Flags as AssetFlags, Handle as AssetHandle, Wrapper as AssetWrapper,
};

use self::asset::Asset;

// TODO: Unify whenever ZMusic gets replaced
newtype!(pub struct Music(StaticSoundData));
newtype!(pub struct Sound(StaticSoundData));

/// Note that all user-facing string fields within may be IDs or expanded.
#[derive(Debug)]
pub struct MountMeta {
	id: String,
	kind: MountKind,
	version: String,
	/// Display name presented to users.
	name: String,
	_description: String,
	_authors: Vec<String>,
	_copyright: String,
	/// Allow a package to link to its forum post/homepage/Discord server/etc.
	_links: Vec<String>,
	virt_path: PathBuf,
}

impl MountMeta {
	#[must_use]
	pub fn id(&self) -> &str {
		&self.id
	}

	#[must_use]
	pub fn kind(&self) -> &MountKind {
		&self.kind
	}

	#[must_use]
	pub fn version(&self) -> &str {
		&self.version
	}

	#[must_use]
	pub fn name(&self) -> &str {
		&self.name
	}

	#[must_use]
	pub fn virt_path(&self) -> &Path {
		&self.virt_path
	}

	#[must_use]
	pub fn from_ingest(ingest: MountMetaIngest, kind: MountKind) -> Self {
		Self {
			id: ingest.id,
			kind,
			version: ingest.version,
			name: ingest.name,
			_description: ingest.description,
			_authors: ingest.authors,
			_copyright: ingest.copyright,
			_links: ingest.links,
			virt_path: ingest.virt_path,
		}
	}
}

/// Intermediate format to keep the code path from mounting to asset loading cleaner.
/// Mostly for TOML parsing, but gets generated and consumed even for loading
/// non-VileTech-packages.
#[derive(Debug, Default, Deserialize)]
pub struct MountMetaIngest {
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
	#[serde(skip)]
	pub virt_path: PathBuf,
}

/// Allows game data objects to define high-level, all-encompassing information
/// relevant to making games as opposed to mods.
#[derive(Debug)]
pub struct GameInfo {
	_steam_app_id: Option<u32>,
	_discord_app_id: Option<String>,
}

/// Determines the system used for loading assets from a mounted game data object.
#[derive(Debug)]
pub enum MountKind {
	/// The file is read to determine what kind of assets are in it,
	/// and loading is handled accordingly.
	File,
	/// Every file in this WAD is loaded into an asset based on
	/// the kind of file it is.
	Wad { internal: bool },
	/// Assets are loaded from this archive/directory based on the
	/// manifest specified by the meta.toml file.
	VileTech { manifest: PathBuf },
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
	pub meta: MountMeta,
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
	pub endooms: AssetVec<Endoom>,
	pub palettes: AssetVec<Palette>,
}

impl Namespace {
	#[must_use]
	pub fn new(metadata: MountMeta) -> Self {
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
			endooms: Default::default(),
			palettes: Default::default(),
		}
	}
}

#[derive(Default)]
pub struct DataCore {
	/// Element 0 should _always_ be the engine's own data, ID "viletech".
	/// Everything afterwards is ordered as per the user's specification.
	pub namespaces: Vec<Namespace>,
	/// IDs are derived from virtual file system paths. The asset ID for
	/// `/viletech/textures/default.png` is exactly the same string; the ID for
	/// blueprint `Imp` defined in file `/viletech/blueprints/doom/imp.lith` is
	/// `/viletech/blueprints/imp/Imp`, since multiple classes can be defined in
	/// one LithScript translation unit.
	pub asset_map: HashMap<String, AssetHandle>,
	/// Doom and its source ports work on a simple data replacement system; a name
	/// points to the last map/texture/song/etc. loaded by that name. VileTech offers
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
	/// - That `ingests[0]` is the parsed metadata for the VileTech data package.
	pub fn populate(
		&mut self,
		mut ingests: Vec<MountMetaIngest>,
		vfs: &VirtualFs,
	) -> Result<(), Error> {
		assert!(
			self.namespaces.is_empty(),
			"Attempting to populate a non-empty `DataCore`."
		);

		assert!(
			!ingests.is_empty(),
			"Called `DataCore::populate` with no mount metadata."
		);

		assert!(
			ingests[0].id == "viletech",
			"`DataCore::populate` should receive the VileTech metadata first."
		);

		for (_index, meta_in) in ingests.drain(..).enumerate() {
			let _fref = vfs
				.lookup(&meta_in.id)
				.expect("Failed to find a namespace's VFS fileref for data core population.");

			let kind = vfs.gamedata_kind(&meta_in.id);

			match kind {
				MountKind::ZDoom => {
					let namespace = Namespace::new(MountMeta::from_ingest(meta_in, kind));
					self.populate_zdoom(vfs, namespace)?;
				}
				MountKind::Wad { .. } => {
					// ???
				}
				MountKind::VileTech { .. } => {
					// ???
				}
				MountKind::File => {
					// ???
				}
				MountKind::Eternity => {
					// ???
				}
			};
		}

		Ok(())
	}
}

impl DataCore {
	/// This method is atomic; `self` is left unmodified in the event of an error.
	fn populate_zdoom(&mut self, vfs: &VirtualFs, namespace: Namespace) -> Result<(), Error> {
		let mount = vfs.lookup(namespace.meta.virt_path()).unwrap();

		let mut dec_root_opt = None;
		let mut zs_root_opt = None;

		for child in mount.child_entries() {
			let file_stem = child.file_stem();
			let lmpname = string::subslice(file_stem, 8);

			if lmpname.eq_ignore_ascii_case("DECORATE") && child.is_string() {
				dec_root_opt = Some(child);
			}

			if lmpname.eq_ignore_ascii_case("ZSCRIPT") && child.is_string() {
				zs_root_opt = Some(child);
			}
		}

		if let Some(dec_root) = dec_root_opt {
			let _content = dec_root.read_str();
			// Soon!
		}

		if let Some(zs_root) = zs_root_opt {
			let _content = zs_root.read_str();
			// Soon!
		}

		Ok(())
	}

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

/// Things that can go wrong during asset loading or access.
#[derive(Debug)]
pub enum Error {
	/// An error occurred at some point during the LithScript compilation pipeline.
	Lith(lith::Error),
	ZScript(zscript::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Lith(err) => err.fmt(f),
			Self::ZScript(err) => err.fmt(f),
		}
	}
}
