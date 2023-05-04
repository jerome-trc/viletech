//! Level (a.k.a. "map") data.

use std::{num::NonZeroU32, sync::Arc};

use bevy::prelude::{IVec2, Vec3};
use bitflags::bitflags;
use image::Rgb;

use crate::{
	sim::{level::Vertex, line::LineFlags},
	EditorNum, ShortId,
};

use super::{super::InHandle, AssetHeader, Audio, Blueprint, Image};

#[derive(Debug)]
pub struct LineDef {
	pub id: i32,
	pub vert_from: i32,
	pub vert_to: i32,
	pub flags: LineFlags,
	pub special: LineSpecial,
	pub args: [i32; 5],
	pub side_right: i32,
	pub side_left: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineSpecial {
	Ceiling {
		/// If `false`, this special is activated by walking over the line.
		switch: bool,
		one_off: bool,
	},
	Crusher {
		one_off: bool,
		speed: f32,
	},
	Door {
		key: DoorKey,
		flags: DoorFlags,
		speed: f32,
	},
	Exit {
		secret: bool,
		/// If `false`, this special is activated by walking over the line.
		switch: bool,
	},
	Floor {
		one_off: bool,
		/// If `false`, this special is activated by walking over the line.
		switch: bool,
		speed: f32,
		tsector: TransferSector,
	},
	Lift {
		one_off: bool,
		/// If `false`, this special is activated by walking over the line.
		switch: bool,
		speed: f32,
	},
	Light {
		one_off: bool,
	},
	ScrollingWall,
	Teleport {
		one_off: bool,
		monsters_only: bool,
	},
	Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransferSector {
	None,
	Trigger,
	Numeric,
}

bitflags! {
	pub struct DoorFlags: u8 {
		const ONE_OFF = 1 << 0;
		const MONSTER_USABLE = 1 << 1;
		const REMOTE = 1 << 2;
	}
}

impl LineSpecial {
	// TODO: Determine values for these constants.
	// Will depend on actual units used by the sim, needs of UDMF vs. vanilla, etc.

	pub const DOOR_SPEED_NORMAL: f32 = 0.0;
	pub const DOOR_SPEED_FAST: f32 = 0.0;

	pub const LIFT_SPEED_NORMAL: f32 = 0.0;
	pub const LIFT_SPEED_FAST: f32 = 0.0;

	pub const FLOOR_SPEED_SLOW: f32 = 0.0;
	pub const FLOOR_SPEED_MED: f32 = 0.0;
	pub const FLOOR_SPEED_FAST: f32 = 0.0;
	pub const FLOOR_SPEED_XFAST: f32 = 0.0;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorKey {
	None,
	Blue,
	Red,
	Yellow,
}

impl LineDef {
	#[must_use]
	pub(in super::super) fn _from_vanilla(
		vert_from: i16,
		vert_to: i16,
		flags: i16,
		special: LineSpecial,
		side_right: i16,
		side_left: i16,
	) -> Self {
		Self {
			id: -1,
			vert_from: vert_from as i32,
			vert_to: vert_to as i32,
			flags: LineFlags::from_bits_truncate(flags as u32),
			special,
			args: [0; 5],
			side_right: side_right as i32,
			side_left: side_left as i32,
		}
	}
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
	pub sector: i32,
}

#[derive(Debug)]
pub struct Sector {
	pub id: i32,
	pub height_floor: i32,
	pub height_ceil: i32,
	/// The ID within maps to a WAD entry's name.
	pub tex_floor: Option<ShortId>,
	/// The ID within maps to a WAD entry's name.
	pub tex_ceil: Option<ShortId>,
	pub light_level: i32,
	pub special: SectorSpecial,
	pub tag: i16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SectorSpecial {
	None,
	Damage {
		dealt: i32,
	},
	DamagingLight {
		dealt: i32,
		blink_interval: f32,
	},
	Door {
		opens_after: Option<NonZeroU32>,
		closes_after: Option<NonZeroU32>,
	},
	/// If a player's health is lowered to less than 11 while standing in this
	/// sector, the level ends.
	End {
		damage: i32,
	},
	Light {
		/// In seconds.
		blink_interval: f32,
	},
	Secret,
	Unknown,
}

#[derive(Debug)]
pub struct Seg {
	pub vert_start: i32,
	pub vert_end: i32,
	/// A binary angle measurement (or "BAMS",
	/// see <https://en.wikipedia.org/wiki/Binary_angular_measurement>).
	pub angle: i16,
	pub linedef: i32,
	pub direction: i16,
	pub offset: i16,
}

#[derive(Debug)]
pub struct SubSector {
	pub seg_count: i32,
	pub seg: i32,
}

#[derive(Debug)]
pub struct Thing {
	pub num: EditorNum,
	pub pos: Vec3,
	pub angle: f64,
	pub flags: ThingFlags,
}

bitflags! {
	pub struct ThingFlags: u16 {
		const SKILL_1 = 1 << 0;
		const SKILL_2 = 1 << 1;
		const SKILL_3 = 1 << 2;
		const SKILL_4 = 1 << 3;
		const SKILL_5 = 1 << 4;
		/// Alternatively "deaf".
		const AMBUSH = 1 << 5;
		const SINGLEPLAY = 1 << 6;
		const DEATHMATCH = 1 << 7;
		const COOP = 1 << 8;
		const FRIEND = 1 << 9;
		const DORMANT = 1 << 10;
		const CLASS_1 = 1 << 11;
		const CLASS_2 = 1 << 12;
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
	pub udmf_namespace: Option<UdmfNamespace>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UdmfNamespace {
	Doom,
	Heretic,
	Hexen,
	Strife,
	ZDoom,
	ZDoomTranslated,
	Vavoom,
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
