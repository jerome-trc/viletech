//! Entity-components making up lines (part of level geometry).

use bevy::prelude::{Component, Entity};
use level::repr::{LineFlags, LockDef};
use serde::{Deserialize, Serialize};

use crate::data::dobj;

use super::level::{SideIndex, VertIndex};

/// Strongly-typed [`Entity`] wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Line(pub(in crate::sim) Entity);

/// Ties line effects to [sectors](crate::sim::sector) with the same trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Trigger(pub(in crate::sim) u16);

#[derive(Component, Debug)]
pub struct Core {
	pub udmf_id: i32,
	pub vert_start: VertIndex,
	pub vert_end: VertIndex,
	pub flags: LineFlags,
	pub side_right: SideIndex,
	pub side_left: Option<SideIndex>,
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
	pub lock: Option<dobj::Handle<LockDef>>,
}

impl Door {
	// TODO: Determine values for these constants.
	// Will depend on actual units used by the sim.
	pub const SPEED_NORMAL: f32 = 0.0;
	pub const SPEED_FAST: f32 = 0.0;
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
