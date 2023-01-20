//! Textures, sprites, brightmaps, polygonal models, voxel models.

use crate::newtype;

use super::Asset;

newtype! {
	/// Stored in RGBA8 format.
	#[derive(Debug)]
	pub struct Image(pub image::RgbaImage)
}

impl Asset for Image {}

/// A placeholder type.
#[derive(Debug)]
pub struct PolyModel;

impl Asset for PolyModel {}

/// A placeholder type.
#[derive(Debug)]
pub struct VoxelModel;

impl Asset for VoxelModel {}
