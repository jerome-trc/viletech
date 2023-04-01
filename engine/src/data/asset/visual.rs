//! Textures, sprites, brightmaps, polygonal models, voxel models.

use std::io::Cursor;

use bevy::prelude::Vec2;
use byteorder::{LittleEndian, ReadBytesExt};
use image::{ImageBuffer, Rgba};

use super::{Asset, AssetKind, Record};

/// Stored in RGBA8 format.
#[derive(Debug)]
pub struct Image {
	pub inner: image::RgbaImage,
	pub offset: Vec2,
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

impl Image {
	/// See <https://doomwiki.org/wiki/Picture_format>. At the moment,
	/// the only possible errors arise if `bytes` is smaller than expected.
	pub fn from_picture(bytes: &[u8], palette: &Palette) -> std::io::Result<Self> {
		let mut cursor = Cursor::new(bytes);

		let width = cursor.read_u16::<LittleEndian>()?;
		let height = cursor.read_u16::<LittleEndian>()?;
		let left = cursor.read_u16::<LittleEndian>()?;
		let top = cursor.read_u16::<LittleEndian>()?;

		let mut ret = ImageBuffer::new(width as u32, height as u32);
		let mut columns = Vec::with_capacity(width as usize);

		for _ in 0..(width as usize - 1) {
			columns.push(cursor.read_u32::<LittleEndian>()? as u64);
		}

		for (i, column) in columns.iter().enumerate() {
			cursor.set_position(*column);

			let mut row_start = 0;

			while row_start != 255 {
				row_start = cursor.read_u8()?;

				if row_start == 255 {
					break;
				}

				let pixel_count = cursor.read_u8()?;
				let _ = cursor.read_u8()?; // Dummy

				for j in 0..(pixel_count as usize) {
					let pal_entry = cursor.read_u8()?;
					let pixel = palette.0[pal_entry as usize];
					let row = i as u32;
					let col = (j as u32) + (row_start as u32);
					ret.put_pixel(col, row, pixel);
				}

				let _ = cursor.read_u8()?; // Dummy
			}
		}

		Ok(Self {
			inner: ret,
			offset: Vec2::new(left as f32, top as f32),
		})
	}
}

#[derive(Debug)]
pub struct Palette(pub [Rgba<u8>; 256]);

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
