//! Level (a.k.a. "map") data.

use bitflags::bitflags;
use glam::IVec2;

use super::{super::InHandle, Asset, AssetKind, Audio, Image, Record};

#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {
	x: f64,
	y: f64,
}

#[derive(Debug)]
pub struct LineDef {
	pub id: i32,
	pub v1: i32,
	pub v2: i32,
	pub flags: LineDefFlags,
	pub special: i32,
	pub args: [i32; 5],
	pub side_front: i32,
	pub side_back: i32,
}

bitflags! {
	#[derive(Default)]
	pub struct LineDefFlags: u32 {
		/// If set, line blocks things.
		const BLOCK_THINGS = 1 << 0;
		/// If set, line blocks monsters.
		const BLOCK_MONS = 1 << 1;
		/// If set, line is 2S.
		const TWO_SIDED = 1 << 2;
		/// If set, upper texture is unpegged.
		const DONT_PEG_TOP = 1 << 3;
		/// If set, lower texture is unpegged.
		const DONT_PEG_BOTTOM = 1 << 4;
		/// If set, drawn as 1S on the map.
		const SECRET = 1 << 5;
		/// If set, blocks sound propagation.
		const BLOCK_SOUND = 1 << 6;
		/// If set, line is never drawn on the map.
		const DONT_DRAW = 1 << 7;
		/// If set, line always appears on the map.
		const MAPPED = 1 << 8;
		/// If set, linedef passes use action.
		const PASS_USE = 1 << 9;
		/// Strife translucency.
		const TRANSLUCENT = 1 << 10;
		/// Strife railing.
		const JUMPOVER = 1 << 11;
		/// Strife floater-blocker.
		const BLOCK_FLOATERS = 1 << 12;
		/// Player can cross.
		const ALLOW_PLAYER_CROSS = 1 << 13;
		/// Player can use.
		const ALLOW_PLAYER_USE = 1 << 14;
		/// Monsters can cross.
		const ALLOW_MONS_CROSS = 1 << 15;
		/// Monsters can use.
		const ALLOW_MONS_USE = 1 << 16;
		/// Projectile can activate.
		const IMPACT = 1 << 17;
		/// Player can push.
		const ALLOW_PLAYER_PUSH = 1 << 18;
		/// Monsters can push.
		const ALLOW_MONS_PUSH = 1 << 19;
		/// Projectiles can cross.
		const ALLOW_MISSILE_CROSS = 1 << 20;
		/// Repeatable special.
		const REPEAT_SPECIAL = 1 << 21;
	}
}

#[derive(Debug)]
pub struct SideDef {
	pub offset: IVec2,
	pub tex_top: Option<InHandle<Image>>,
	pub tex_bottom: Option<InHandle<Image>>,
	pub tex_mid: Option<InHandle<Image>>,
	pub sector: i32,
}

#[derive(Debug)]
pub struct Sector {
	pub height_floor: i32,
	pub height_ceiling: i32,
	pub tex_floor: Option<InHandle<Image>>,
	pub tex_ceiling: Option<InHandle<Image>>,
	pub light_level: i32,
	pub special: i32,
	pub id: i32,
}

#[derive(Debug)]
pub struct Level {
	pub meta: LevelMeta,
	pub udmf_namespace: Option<UdmfNamespace>,
}

impl Asset for Level {
	const KIND: AssetKind = AssetKind::Level;

	unsafe fn get(record: &Record) -> &Self {
		&record.asset.level
	}

	unsafe fn get_mut(record: &mut Record) -> &mut Self {
		&mut record.asset.level
	}
}

/// Comes from a map entry in a MAPINFO lump.
#[derive(Debug)]
pub struct LevelMeta {
	/// Displayed to the user. May be a string ID.
	pub name: String,
	/// Prepended to the level name on the automap. May be a string ID.
	pub label: String,
	/// May be a string ID.
	pub author_name: String,
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
