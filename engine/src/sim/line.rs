//! Entity-components making up lines (part of level geometry).
//!
//! A sector only has the contents of a [`MaterialMeshBundle`] if it can move.

use bevy::prelude::{Component, Entity};
use serde::{Deserialize, Serialize};

use crate::data::{self, asset};

use super::level::{Level, SideIndex, VertIndex};

/// Strongly-typed [`Entity`] wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Line(Entity);

/// Ties line effects to [sectors](crate::sim::sector) with the same trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Trigger(u16);

#[derive(Component, Debug)]
pub struct Core {
	/// Which level does this line belong to?
	pub level: Level,
	/// From UDMF; deliberately very distinct from the Bevy entity.
	pub id: i32,
	pub vert_from: VertIndex,
	pub vert_to: VertIndex,
	pub flags: LineFlags,
	pub side_right: SideIndex,
	pub side_left: SideIndex,
}

bitflags::bitflags! {
	#[derive(Default)]
	pub struct LineFlags: u32 {
		/// Line blocks things (i.e. player, missiles, and monsters).
		const IMPASSIBLE = 1 << 0;
		/// Line blocks monsters.
		const BLOCK_MONS = 1 << 1;
		/// Line's two sides can have the "transparent texture".
		const TWO_SIDED = 1 << 2;
		/// Upper texture is pasted onto wall from the top down instead of bottom-up.
		const UPPER_UNPEGGED = 1 << 3;
		/// Lower and middle textures are drawn from the bottom up instead of top-down.
		const LOWER_UNPEGGED = 1 << 4;
		/// If set, drawn as 1S on the map.
		const SECRET = 1 << 5;
		/// If set, blocks sound propagation.
		const BLOCK_SOUND = 1 << 6;
		/// If set, line is never drawn on the automap,
		/// even if the computer area map power-up is acquired.
		const UNMAPPED = 1 << 7;
		/// If set, line always appears on the automap,
		/// even if no player has seen it yet.
		const PRE_MAPPED = 1 << 8;
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
		const ALLOW_PLAYER_PUSH = 1 << 18;
		const ALLOW_MONS_PUSH = 1 << 19;
		const ALLOW_PROJ_CROSS = 1 << 20;
		const REPEAT_SPECIAL = 1 << 21;
	}
}

// Optional components /////////////////////////////////////////////////////////

#[derive(Component, Debug)]
pub struct Activation {
	pub walk_over: bool,
	pub shoot: bool,
	pub one_off: bool,
}

/// Defined by a UDMF TEXTMAP file. Primarily for ACS use.
#[derive(Component, Debug)]
pub struct Args([i32; 5]);

#[derive(Component, Debug)]
pub struct Door {
	/// In ticks. If the door's "at-rest" state is closed, this is the time it
	/// waits while open to close again, and vice-versa.
	pub stay_time: u32,
	/// In ticks. Gets set to `stay_time` and counted down to 0.
	pub stay_timer: u32,
	pub one_off: bool,
	pub monster_usable: bool,
	pub remote: bool,
	/// In map units.
	pub speed: f32,
	pub lock: Option<data::Handle<asset::LockDef>>,
}

#[derive(Component, Debug)]
pub struct Exit {
	pub secret: bool,
	/// If `false`, this special is activated by walking over the line.
	pub switch: bool,
}

/// To support GZ's destructible map geometry features.
#[derive(Component, Debug)]
pub struct Health {
	pub current: i32,
}

#[derive(Component, Debug)]
pub struct Light;

#[derive(Component, Debug)]
pub struct Teleport {
	pub one_off: bool,
	pub monsters_only: bool,
}
