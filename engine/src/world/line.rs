//! Code integrating [level linedefs] into the playsim ECS.
//!
//! [level linedefs]: https://doomwiki.org/wiki/Linedef

use bevy::prelude::*;

use crate::gfx::ImageSlot;

/// A strongly-typed [`Entity`] wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ELine(Entity);

impl From<Entity> for ELine {
	fn from(value: Entity) -> Self {
		Self(value)
	}
}

impl From<ELine> for Entity {
	fn from(value: ELine) -> Self {
		value.0
	}
}

#[derive(Component, Debug, Default, Clone)]
pub struct FrontMesh {
	/// Each element corresponds to three vertices in an attribute of a [`Mesh`].
	pub tris: Vec<usize>,
}

#[derive(Component, Debug, Default, Clone)]
pub struct BackMesh {
	/// Each element corresponds to three vertices in an attribute of a [`Mesh`].
	pub tris: Vec<usize>,
}

#[derive(Component, Debug, Default, Clone)]
pub struct FrontTextures {
	pub top: Option<ImageSlot>,
	pub mid: Option<ImageSlot>,
	pub bottom: Option<ImageSlot>,
}

#[derive(Component, Debug, Default, Clone)]
pub struct BackTextures {
	pub top: Option<ImageSlot>,
	pub mid: Option<ImageSlot>,
	pub bottom: Option<ImageSlot>,
}
