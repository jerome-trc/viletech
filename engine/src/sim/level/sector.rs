//! Entity-components making up sectors (part of level geometry).
//!
//! In VileTech a sector is a floor and ceiling encompassed by [sides].
//! Vanilla Doom's restrictions do not apply; these are allowed to overlap vertically.
//!
//! [sides]: crate::sim::level::Side

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::line::Line;

/// Strongly-typed [`Entity`] wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Sector(pub(super) Entity);

/// Every sector has this component.
#[derive(Component, Debug)]
pub struct Core {
	/// Lines encompassing this sector.
	pub lines: Vec<Line>,
}

// Optional components /////////////////////////////////////////////////////////

#[derive(Component, Debug)]
pub struct CloseAfter {
	/// When this reaches 0, this component is removed
	/// and the sector closes as though it were a door.
	pub ticks: u32,
}

#[derive(Component, Debug)]
pub struct Damaging {
	/// Note that at the lowest skill level,
	/// the final damage applied to relevant actors is half of this.
	pub damage: i32,
	/// In ticks.
	pub interval: u16,
	/// Every 32 ticks of standing on a damaging floor, powerups which protect against
	/// these kinds of floors might be bypassed as per this chance.
	/// Expressed as an probability in 255, where 255 is guaranteed and 0 is "never".
	pub leak_chance: u8,
}

#[derive(Component, Debug)]
pub struct Ending {
	/// If the player actor's health goes below this while standing in the sector
	/// bearing this component, the level ends.
	///
	/// In vanilla maps, this is always 11, and paired with [`Damaging`].
	pub threshold: i32,
}

/// To support GZ's destructible map geometry features.
#[derive(Component, Debug)]
pub struct Health {
	pub current: i32,
}

/// Heals player actors standing on it.
#[derive(Component, Debug)]
pub struct Healing {
	/// 32 ticks by default.
	pub interval: u32,
	pub amount: i32,
}

#[derive(Component, Debug)]
pub struct Light {
	/// In seconds.
	pub blink_interval: f32,
}

#[derive(Component, Debug)]
pub struct Lightning {
	pub light_inc: u16,
}

#[derive(Component, Debug)]
pub struct OpenAfter {
	/// When this reaches 0, the component is removed and the sector opens like a door.
	pub ticks: u32,
}

/// Linear UV translation per second.
#[derive(Component, Debug)]
pub struct Scrolling {
	pub direction: Vec3,
}

/// A player actor must stand in this sector to get credit for finding this secret.
#[derive(Component, Debug)]
pub struct Secret;

#[derive(Component, Debug)]
pub struct Wind {
	pub direction: Vec3,
}
