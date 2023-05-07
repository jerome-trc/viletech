//! Level (a.k.a. "map") data.

use std::sync::Arc;

use bevy::prelude::{IVec2, Vec3};
use bitflags::bitflags;
use image::Rgb;
use serde::{Deserialize, Serialize};

use crate::{
	sim::level::{line::LineFlags, Vertex},
	EditorNum, ShortId,
};

use super::{AssetHeader, Audio, Blueprint, Image, InHandle};

#[derive(Debug)]
pub struct LineDef {
	pub udmf_id: i32,
	pub vert_start: usize,
	pub vert_end: usize,
	pub flags: LineFlags,
	pub special: u16,
	/// Corresponds to the field of [`Sector`] with the same name.
	pub trigger: u16,
	/// Defined in UDMF.
	pub args: Option<[i32; 5]>,
	pub side_right: usize,
	pub side_left: Option<usize>,
}

#[derive(Debug)]
pub struct SideDef {
	pub offset: IVec2,
	/// The ID within maps to a TEXTURE1/2 entry.
	pub tex_top: Option<ShortId>,
	/// The ID within maps to a TEXTURE1/2 entry.
	pub tex_bottom: Option<ShortId>,
	/// The ID within maps to a TEXTURE1/2 entry.
	pub tex_mid: Option<ShortId>,
	pub sector: usize,
}

#[derive(Debug)]
pub struct Sector {
	pub udmf_id: i32,
	pub height_floor: i32,
	pub height_ceil: i32,
	/// The ID within maps to a WAD entry's name.
	pub tex_floor: Option<ShortId>,
	/// The ID within maps to a WAD entry's name.
	pub tex_ceil: Option<ShortId>,
	pub light_level: i32,
	pub special: u16,
	/// Corresponds to the field of [`LineDef`] with the same name.
	pub trigger: u16,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SegDirection {
	/// This seg runs along the right of a linedef.
	Front,
	/// This seg runs along the left of a linedef.
	Back,
}

#[derive(Debug)]
pub struct SubSector {
	pub seg_count: i32,
	pub seg: usize,
}

#[derive(Debug)]
pub struct Thing {
	pub tid: i32,
	pub num: EditorNum,
	pub pos: Vec3,
	/// In degrees.
	pub angle: u16,
	pub flags: ThingFlags,
	pub args: [i32; 5],
}

bitflags! {
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
	pub header: AssetHeader,
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

#[derive(Debug)]
pub struct Level {
	pub header: AssetHeader,
	pub meta: LevelMeta,
	pub format: LevelFormat,
	pub vertices: Vec<Vertex>,
	pub linedefs: Vec<LineDef>,
	pub sectors: Vec<Sector>,
	pub segs: Vec<Seg>,
	pub sidedefs: Vec<SideDef>,
	pub subsectors: Vec<SubSector>,
	pub things: Vec<Thing>,
}

/// Comes from a map entry in a MAPINFO lump.
#[derive(Debug)]
pub struct LevelMeta {
	/// e.g. "Entryway". Displayed to the user. May be a string ID. The asset ID
	/// will be, for example, "DOOM2/MAP01" and gets stored in the [header].
	///
	/// [header]: AssetHeader
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LevelFormat {
	Doom,
	Hexen,
	Udmf(UdmfNamespace),
}

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
	#[derive(Default)]
	pub struct ClusterFlags: u8 {
		const IS_HUB = 1 << 0;
		const ALLOW_INTERMISSION = 1 << 1;
	}
}
