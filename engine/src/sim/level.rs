//! Level state for the playsim and renderer.
//!
//! While not strictly necessarily, making this a part of the ECS allows use of
//! Bevy's ECS hierarchies to easily clean up an entire level recursively with
//! one call.

use std::{collections::HashMap, hash::Hash, sync::Arc};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use util::math::Fixed32;

use crate::{
	data::dobj::{self, UdmfValue},
	sparse::{SparseSet, SparseSetIndex},
};

use super::{line, sector::Sector};

/// Strongly-typed [`Entity`] wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Level(Entity);

/// The principal component in a level entity.
#[derive(Component, Debug)]
pub struct Core {
	pub base: Option<dobj::Handle<dobj::Level>>,
	pub flags: Flags,
	/// Time spent in this level thus far.
	pub ticks_elapsed: u64,
	pub geom: Geometry,
}

/// Sub-structure for composing [`Core`].
///
/// The vertex array, trigger map, and some counters.
#[derive(Debug)]
pub struct Geometry {
	pub mesh: Handle<Mesh>,
	pub verts: SparseSet<VertIndex, Vertex>,
	pub sides: SparseSet<SideIndex, Side>,
	/// Each stored entity ID points to a sector.
	///
	/// When a line is triggered (walked over, interacted-with, shot), all sectors
	/// in the corresponding array have all "activatable" components get activated.
	pub triggers: HashMap<line::Trigger, Vec<Sector>>,
	/// Updated as map geometry changes.
	pub num_sectors: usize,
}

bitflags::bitflags! {
	#[derive(Default)]
	pub struct Flags: u8 {
		// From GZ. Purpose unclear.
		const FROZEN_LOCAL = 1 << 0;
		// From GZ. Purpose unclear.
		const FROZEN_GLOBAL = 1 << 1;
		/// Monsters which teleport so as to have bounding box intersection with
		/// a player actor kill that actor. Primarily for use in Doom 2's MAP30.
		const MONSTERS_TELEFRAG = 1 << 2;
	}
}

/// If a piece of level geometry changes during a sim tick so as to require an
/// update to one of its vertex attributes, this component is added by the sim.
#[derive(Component, Debug)]
#[component(storage = "SparseSet")]
pub struct Dirty;

// Vertex information //////////////////////////////////////////////////////////

#[derive(Debug, Deref, DerefMut, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vertex(pub Vec4);

impl Vertex {
	/// a.k.a. "floor" or "ground". Corresponds to the vector's `y` component.
	#[must_use]
	pub fn bottom(self) -> f32 {
		self.0.y
	}

	#[must_use]
	pub fn bottom_mut(&mut self) -> &mut f32 {
		&mut self.0.y
	}

	/// a.k.a. "ceiling" or "sky". Corresponds to the vector's `w` component.
	#[must_use]
	pub fn top(self) -> f32 {
		self.0.w
	}

	#[must_use]
	pub fn top_mut(&mut self) -> &mut f32 {
		&mut self.0.w
	}

	#[must_use]
	pub fn x_fixed(self) -> Fixed32 {
		Fixed32::from_num(self.0.x)
	}

	#[must_use]
	pub fn z_fixed(self) -> Fixed32 {
		Fixed32::from_num(self.0.z)
	}
}

impl From<Vertex> for Vec4 {
	fn from(value: Vertex) -> Self {
		value.0
	}
}

impl From<Vec4> for Vertex {
	fn from(value: Vec4) -> Self {
		Self(value)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VertIndex(pub(super) usize);

impl From<VertIndex> for usize {
	fn from(value: VertIndex) -> Self {
		value.0
	}
}

impl SparseSetIndex for VertIndex {}

// Line sides //////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Side {
	pub offset: IVec2,
	pub sector: Sector,
	pub udmf: Udmf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SideIndex(pub(super) usize);

impl From<SideIndex> for usize {
	fn from(value: SideIndex) -> Self {
		value.0
	}
}

impl SparseSetIndex for SideIndex {}

// UDMF ////////////////////////////////////////////////////////////////////////

/// A map of arbitrary string-keyed values defined in a UDMF TEXTMAP file.
///
/// Can be attached to a line, side, or sector.
#[derive(Component, Debug, Default)]
pub struct Udmf(HashMap<Arc<str>, UdmfValue>);
