//! Functions used to prepare for starting the sim.

pub(super) mod level;
pub(super) mod line;
pub(super) mod sector;

use bevy::prelude::*;

use crate::{data::Catalog, gfx::TerrainMaterial};

pub struct Context<'w> {
	pub catalog: &'w Catalog,
	pub meshes: ResMut<'w, Assets<Mesh>>,
	pub materials: ResMut<'w, Assets<TerrainMaterial>>,
	pub images: ResMut<'w, Assets<Image>>,
}
