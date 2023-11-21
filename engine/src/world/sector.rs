//! Code integrating [level sectors] into the playsim ECS.
//!
//! [level sectors]: https://doomwiki.org/wiki/Sector

use bevy::prelude::*;

use crate::gfx::ImageSlot;

/// A strongly-typed [`Entity`] wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ESector(Entity);

impl From<Entity> for ESector {
	fn from(value: Entity) -> Self {
		Self(value)
	}
}

impl From<ESector> for Entity {
	fn from(value: ESector) -> Self {
		value.0
	}
}

#[derive(Component, Debug, Default, Clone)]
pub struct Mesh {
	/// Each element corresponds to three vertices in an attribute of a [`Mesh`].
	pub tris: Vec<usize>,
}

#[derive(Component, Debug, Default, Clone)]
pub struct Textures {
	pub floor: Option<ImageSlot>,
	pub ceiling: Option<ImageSlot>,
}
