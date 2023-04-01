//! Textures, sprites, brightmaps, polygonal models, voxel models.

use crate::newtype;

use super::{Asset, AssetKind, Record};

newtype! {
	/// Stored in RGBA8 format.
	#[derive(Debug)]
	pub struct Image(pub image::RgbaImage)
}

impl Asset for Image {
	const KIND: AssetKind = AssetKind::Image;

	unsafe fn get(record: &Record) -> &Self {
		&record.asset.image
	}

	unsafe fn get_mut(record: &mut Record) -> &mut Self {
		&mut record.asset.image
	}
}

/// A placeholder type.
#[derive(Debug)]
pub struct PolyModel;

impl Asset for PolyModel {
	const KIND: AssetKind = AssetKind::PolyModel;

	unsafe fn get(record: &Record) -> &Self {
		&record.asset.poly_model
	}

	unsafe fn get_mut(record: &mut Record) -> &mut Self {
		&mut record.asset.poly_model
	}
}

/// A placeholder type.
#[derive(Debug)]
pub struct VoxelModel;

impl Asset for VoxelModel {
	const KIND: AssetKind = AssetKind::VoxelModel;

	unsafe fn get(record: &Record) -> &Self {
		&record.asset.voxel_model
	}

	unsafe fn get_mut(record: &mut Record) -> &mut Self {
		&mut record.asset.voxel_model
	}
}
