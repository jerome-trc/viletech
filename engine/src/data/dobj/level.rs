//! Level (a.k.a. "map") data.

use std::{collections::HashMap, sync::Arc};

use bevy::prelude::{IVec2, Vec2, Vec3};
use bitflags::bitflags;
use image::Rgb;
use serde::{Deserialize, Serialize};

use crate::{
	math::MinMaxBox,
	sim::{level::Vertex, line::Flags},
	EditorNum, Id8, SmallString,
};

use super::{Audio, Blueprint, Image, InHandle};

/// See <https://doomwiki.org/wiki/Node>.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BspNode {
	pub seg_start: Vec2,
	pub seg_end: Vec2,
	pub child_r: BspNodeChild,
	pub child_l: BspNodeChild,
}

/// See [`BspNode`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BspNodeChild {
	SubSector(usize),
	SubNode(usize),
}

/// See <https://doomwiki.org/wiki/Linedef>.
#[derive(Debug)]
pub struct LineDef {
	pub udmf_id: i32,
	pub vert_start: usize,
	pub vert_end: usize,
	pub flags: Flags,
	pub special: u16,
	/// Corresponds to the field of [`Sector`] with the same name.
	pub trigger: u16,
	/// Defined in UDMF.
	pub args: Option<[i32; 5]>,
	pub side_right: usize,
	pub side_left: Option<usize>,
}

/// See <https://doomwiki.org/wiki/Sidedef>.
#[derive(Debug)]
pub struct SideDef {
	pub offset: IVec2,
	/// The ID within maps to a TEXTURE1/2 entry.
	pub tex_top: Option<Id8>,
	/// The ID within maps to a TEXTURE1/2 entry.
	pub tex_bottom: Option<Id8>,
	/// The ID within maps to a TEXTURE1/2 entry.
	pub tex_mid: Option<Id8>,
	pub sector: usize,
}

/// See <https://doomwiki.org/wiki/Sector>.
#[derive(Debug)]
pub struct Sector {
	pub udmf_id: i32,
	pub height_floor: f32,
	pub height_ceil: f32,
	/// The ID within maps to a WAD entry's name.
	pub tex_floor: Option<Id8>,
	/// The ID within maps to a WAD entry's name.
	pub tex_ceil: Option<Id8>,
	pub light_level: i32,
	pub special: u16,
	/// Corresponds to the field of [`LineDef`] with the same name.
	pub trigger: u16,
}

/// See <https://doomwiki.org/wiki/Seg>.
#[derive(Debug, Hash, Serialize, Deserialize)]
pub struct Seg {
	pub vert_start: usize,
	pub vert_end: usize,
	/// A binary angle measurement (or "BAMS",
	/// see <https://en.wikipedia.org/wiki/Binary_angular_measurement>).
	pub angle: i16,
	pub linedef: usize,
	pub direction: SegDirection,
	pub offset: i16,
}

/// See [`Seg`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SegDirection {
	/// This seg runs along the right of a linedef.
	Front,
	/// This seg runs along the left of a linedef.
	Back,
}

/// See <https://doomwiki.org/wiki/Subsector>.
#[derive(Debug)]
pub struct SubSector {
	pub seg0: usize,
	pub seg_count: usize,
}

/// See <https://doomwiki.org/wiki/Thing>.
#[derive(Debug)]
pub struct Thing {
	pub tid: i32,
	pub num: EditorNum,
	/// Reader's note: Bevy's coordinate system is right-handed Y-up.
	pub pos: Vec3,
	/// In degrees.
	pub angle: u16,
	pub flags: ThingFlags,
	pub args: [i32; 5],
}

bitflags! {
	/// See [`Thing`].
	pub struct ThingFlags: u16 {
		const SKILL_1 = 1 << 0;
		const SKILL_2 = 1 << 1;
		const SKILL_3 = 1 << 2;
		const SKILL_4 = 1 << 3;
		const SKILL_5 = 1 << 4;
		/// Alternatively "deaf", but not in terms of sound propagation.
		const AMBUSH = 1 << 5;
		const SINGLEPLAY = 1 << 6;
		const DEATHMATCH = 1 << 7;
		const COOP = 1 << 8;
		const FRIEND = 1 << 9;
		const DORMANT = 1 << 10;
		/// If unset, this is absent to e.g. Hexen's Fighter class.
		const CLASS_1 = 1 << 11;
		/// If unset, this is absent to e.g. Hexen's Cleric class.
		const CLASS_2 = 1 << 12;
		/// If unset, this is absent to e.g. Hexen's Mage class.
		const CLASS_3 = 1 << 13;
	}
}

/// Adapted one-to-one from GZ. See <https://zdoom.org/wiki/LOCKDEFS>.
#[derive(Debug)]
pub struct LockDef {
	pub reqs: Vec<KeyReq>,
	/// Printed when trying to open a door without having the required keys.
	pub interact_msg: Arc<str>,
	/// Printed when trying to press a remote switch without having the required keys.
	pub remote_msg: Arc<str>,
	/// Played when trying to open this door without having the required keys.
	pub sound: Option<InHandle<Audio>>,
	/// Lines with this lock are drawn as this color on the automap.
	pub automap_color: Rgb<u8>,
}

/// See [`LockDef`].
#[derive(Debug)]
pub enum KeyReq {
	Exact(InHandle<Blueprint>),
	AnyOf(Vec<InHandle<Blueprint>>),
}

/// Alternatively a "map".
#[derive(Debug)]
pub struct Level {
	pub meta: LevelMeta,
	pub format: LevelFormat,
	pub linedefs: Vec<LineDef>,
	pub nodes: Vec<BspNode>,
	pub sectors: Vec<Sector>,
	pub segs: Vec<Seg>,
	pub sidedefs: Vec<SideDef>,
	pub subsectors: Vec<SubSector>,
	pub things: Vec<Thing>,
	pub vertices: Vec<Vertex>,
	pub bounds: MinMaxBox,
	pub udmf: HashMap<UdmfKey, UdmfValue>,
}

impl Level {
	#[must_use]
	pub fn new(format: LevelFormat) -> Self {
		Self {
			meta: LevelMeta {
				name: String::default().into(),
				author_name: String::default().into(),
				music: None,
				next: None,
				next_secret: None,
				par_time: 0,
				special_num: 0,
				flags: LevelFlags::empty(),
			},
			format,
			linedefs: vec![],
			nodes: vec![],
			sectors: vec![],
			segs: vec![],
			sidedefs: vec![],
			subsectors: vec![],
			things: vec![],
			vertices: vec![],
			bounds: MinMaxBox::default(),
			udmf: HashMap::new(),
		}
	}

	#[must_use]
	pub fn bounds(verts: &[Vertex]) -> MinMaxBox {
		let mut min = glam::vec3a(0.0, 0.0, 0.0);
		let mut max = glam::vec3a(0.0, 0.0, 0.0);

		for vert in verts {
			if vert.x < min.x {
				min.x = vert.x;
			} else if vert.x > max.x {
				max.x = vert.x;
			}

			if vert.bottom() < min.y {
				min.y = vert.y;
			} else if vert.bottom() > max.y {
				max.y = vert.y;
			}

			if vert.z < min.z {
				min.z = vert.z;
			} else if vert.z > max.z {
				max.z = vert.z;
			}
		}

		MinMaxBox { min, max }
	}
}

/// Comes from a map entry in a MAPINFO lump.
#[derive(Debug)]
pub struct LevelMeta {
	/// e.g. "Entryway". Displayed to the user. May be a string ID. The datum ID
	/// will be, for example, "DOOM2/MAP01" and gets stored in the [header].
	///
	/// [header]: DatumHeader
	pub name: Arc<str>,
	/// May be a string ID.
	pub author_name: Arc<str>,
	pub music: Option<InHandle<Audio>>,
	/// The level that players are taken to upon passing through the normal exit.
	pub next: Option<InHandle<Level>>,
	/// The level to which the secret exit leads, if any.
	pub next_secret: Option<InHandle<Level>>,
	/// In seconds.
	pub par_time: u32,
	/// Only used by ACS.
	pub special_num: i32,
	pub flags: LevelFlags,
}

bitflags! {
	#[derive(Default)]
	pub struct LevelFlags: u8 {
		/// Switch lines must be vertically reachable to allow interaction.
		const CHECK_SWITCH_RANGE = 1 << 0;
	}
}

/// See <https://doomwiki.org/wiki/Map_format>.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LevelFormat {
	Doom,
	Hexen,
	Udmf(UdmfNamespace),
}

/// See <https://doomwiki.org/wiki/UDMF>.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UdmfNamespace {
	Doom,
	Eternity,
	Heretic,
	Hexen,
	Strife,
	Vavoom,
	ZDoom,
	ZDoomTranslated,
}

#[derive(Debug)]
pub struct Episode {
	/// Displayed to the user. May be a string ID.
	pub name: String,
	pub start_level: Option<InHandle<Level>>,
	pub background: Option<InHandle<Image>>,
	pub flags: EpisodeFlags,
}

bitflags! {
	/// See [`Episode`].
	#[derive(Default)]
	pub struct EpisodeFlags: u8 {
		const NO_SKILL_MENU = 1 << 0;
		const OPTIONAL = 1 << 1;
		const EXTENDED = 1 << 2;
	}
}

#[derive(Debug)]
pub struct Cluster {
	/// Displayed to the user. May be a string ID.
	pub text_enter: String,
	/// Displayed to the user. May be a string ID.
	pub text_exit: String,
	pub flags: ClusterFlags,
	pub music: InHandle<Audio>,
	pub background: InHandle<Image>,
}

bitflags! {
	/// See [`Cluster`].
	#[derive(Default)]
	pub struct ClusterFlags: u8 {
		const IS_HUB = 1 << 0;
		const ALLOW_INTERMISSION = 1 << 1;
	}
}

/// In any given variant, `index` corresponds to one of the arrays in [`Level`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UdmfKey {
	Linedef { field: SmallString, index: usize },
	Sector { field: SmallString, index: usize },
	Sidedef { field: SmallString, index: usize },
	Thing { field: SmallString, index: usize },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UdmfValue {
	Bool(bool),
	Int(i32),
	Float(f64),
	String(SmallString), // Q: Intern these?
}
