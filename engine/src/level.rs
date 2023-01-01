use bitflags::bitflags;
use glam::IVec2;

use crate::data::AssetHandle;

pub struct Vertex {
	x: f64,
	y: f64,
}

#[derive(Default)]
pub struct LineDef {
	id: i32,
	v1: i32,
	v2: i32,
	flags: LineDefFlags,
	special: i32,
	args: [i32; 5],
	side_front: i32,
	side_back: i32,
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
		/// If set, drawn as 1S on map.
		const SECRET = 1 << 5;
		/// If set, blocks sound propagation.
		const BLOCK_SOUND = 1 << 6;
		/// If set, line is never drawn on map.
		const DONT_DRAW = 1 << 7;
		/// If set, line always appears on map.
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

pub struct SideDef {
	offset: IVec2,
	tex_top: AssetHandle,
	tex_bottom: AssetHandle,
	tex_mid: AssetHandle,
	sector: i32,
}

pub struct Sector {
	height_floor: i32,
	height_ceiling: i32,
	tex_floor: AssetHandle,
	tex_ceiling: AssetHandle,
	light_level: i32,
	special: i32,
	id: i32,
}

pub struct Metadata {
	/// Displayed to the user. May be a string ID.
	pub name: String,
	/// Prepended to the level name on the automap. May be a string ID.
	pub label: String,
	/// May be a string ID.
	pub author_name: String,
	pub music: Option<AssetHandle>,
	/// The map that players are taken to upon passing through the normal exit.
	pub next: Option<AssetHandle>,
	/// The map to which the secret exit leads, if any.
	pub next_secret: Option<AssetHandle>,
	/// In seconds.
	pub par_time: u32,
	/// Only used by ACS.
	pub special_num: i32,
	pub flags: Flags,
}

bitflags! {
	#[derive(Default)]
	pub struct Flags: u8 {
		/// Switch lines must be vertically reachable to allow interaction.
		const CHECK_SWITCH_RANGE = 1 << 0;
	}
}

pub struct Episode {
	/// Displayed to the user. May be a string ID.
	pub name: String,
	pub start_map: AssetHandle,
	pub background: AssetHandle,
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

pub struct Cluster {
	/// Displayed to the user. May be a string ID.
	text_enter: String,
	/// Displayed to the user. May be a string ID.
	text_exit: String,
	flags: ClusterFlags,
	music: AssetHandle,
	background: AssetHandle,
}

bitflags! {
	#[derive(Default)]
	pub struct ClusterFlags: u8 {
		const IS_HUB = 1 << 0;
		const ALLOW_INTERMISSION = 1 << 1;
	}
}
