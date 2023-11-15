//! Code for assembling and representing levels in the ECS.
//!
//! Note that any relevant game simulation code is in [`crate::sim`] instead;
//! this module sub-tree is only for symbols that are useful to both the sim
//! and the level editor.

pub mod mesh;

use bevy::prelude::*;
use data::level::read::VertexRaw;
use util::sparseset::SparseSetIndex;

/// See <https://doomwiki.org/wiki/Map_format>.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LevelFormat {
	/// Level has an ordered sequence of lumps from `THINGS` to `BLOCKMAP`.
	Doom,
	/// a.k.a. the "Hexen" format.
	/// Like [`LevelFormat::Doom`] but with `BEHAVIOR` after `BLOCKMAP`.
	Extended,
	/// Starts with a marker and a `TEXTMAP` lump; ends with an `ENDMAP` lump.
	Udmf(UdmfNamespace),
}

/// See <https://doomwiki.org/wiki/UDMF>.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

/// All 16-bit integer position values get cast to `f32` and then reduced by this
/// to fit VileTech's floating-point space.
pub const FSCALE: f32 = 0.01;

/// A strongly-typed [`Entity`] wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ELevel(Entity);

/// A strongly-typed [`Entity`] wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ESector(Entity);

/// A strongly-typed [`Entity`] wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ELine(Entity);

/// A "flag" component for marking entities as being part of an active level.
///
/// Level geometry and actors without this are not subject to per-tick iteration.
#[derive(Component, Debug)]
#[component(storage = "SparseSet")]
pub struct ActiveMarker;

#[derive(Debug, Clone, Copy, PartialEq, Deref, DerefMut)]
pub struct Vertex(pub Vec4);

impl From<VertexRaw> for Vertex {
	fn from(value: VertexRaw) -> Self {
		let pos = value.position();

		Self(Vec4::new(
			(pos[0] as f32) * FSCALE,
			(pos[1] as f32) * FSCALE,
			0.0,
			0.0,
		))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VertIx(u32);

impl From<VertIx> for usize {
	fn from(value: VertIx) -> Self {
		value.0 as usize
	}
}

impl SparseSetIndex for VertIx {}

pub fn level_bundle_base() -> impl Bundle {
	(
		GlobalTransform::default(),
		InheritedVisibility::default(),
		ViewVisibility::default(),
		ActiveMarker,
	)
}
