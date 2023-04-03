//! Textures, sprites, brightmaps, polygonal models, voxel models.
//!
//! Code derived from [SLADE](https://slade.mancubus.net/) is used under the
//! GNU GPL v2.0. See <https://github.com/sirjuddington/SLADE/blob/master/LICENSE>.
//! A copy is attached in the `/legal` directory.

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
	/// See <https://doomwiki.org/wiki/Picture_format>.
	/// Partially adapted from SLADE's `DoomGfxDataFormat::isThisFormat`.
	/// Does not allocate until reasonably certain that `bytes` is a picture,
	/// so `try_from_picture.is_some()` can be used as a fairly cheap check.
	#[must_use]
	pub fn try_from_picture(bytes: &[u8], palette: &Palette) -> Option<Self> {
		const HEADER_SIZE: usize = std::mem::size_of::<u16>() * 4;

		if bytes.len() < HEADER_SIZE {
			return None;
		}

		let mut cursor_h = Cursor::new(bytes);

		let width = cursor_h.read_u16::<LittleEndian>().unwrap();
		let height = cursor_h.read_u16::<LittleEndian>().unwrap();
		let left = cursor_h.read_i16::<LittleEndian>().unwrap();
		let top = cursor_h.read_i16::<LittleEndian>().unwrap();

		// (SLADE) Sanity checks on dimensions and offsets.

		if width >= 4096 || height >= 4096 {
			return None;
		}

		if left <= -2000 || left >= 2000 {
			return None;
		}

		if top <= -2000 || top >= 2000 {
			return None;
		}

		if bytes.len() < (HEADER_SIZE + (width as usize * 4)) {
			return None;
		}

		let checkpoint = cursor_h.position();

		for _ in 0..width {
			let col_offs = cursor_h.read_u32::<LittleEndian>().unwrap() as usize;

			if col_offs > bytes.len() || col_offs < (HEADER_SIZE) {
				return None;
			}

			// (SLADE) Check if total size is reasonable; this computation corresponds
			// to the most inefficient possible use of space by the format
			// (horizontal stripes of 1 pixel, 1 pixel apart).
			let num_pixels = ((height + 2 + height % 2) / 2) as usize;
			let max_col_size = std::mem::size_of::<u32>() + (num_pixels * 5) + 1;

			if bytes.len() > HEADER_SIZE + (width as usize * max_col_size) {
				// Q: Unlikely, but possible. Should we try?
				return None;
			}
		}

		let mut ret = ImageBuffer::new(width as u32, height as u32);
		let mut cursor_pix = Cursor::new(bytes);
		cursor_h.set_position(checkpoint);

		for i in 0..width {
			let col_offs = cursor_h.read_u32::<LittleEndian>().unwrap() as u64;
			cursor_pix.set_position(col_offs);
			let mut row_start = 0;

			while row_start != 255 {
				row_start = cursor_pix.read_u8().unwrap();

				if row_start == 255 {
					break;
				}

				let pixel_count = cursor_pix.read_u8().unwrap();
				let _ = cursor_pix.read_u8().unwrap(); // Dummy

				for ii in 0..(pixel_count as usize) {
					let pal_entry = cursor_pix.read_u8().unwrap();
					let pixel = palette.0[pal_entry as usize];
					let row = i as u32;
					let col = (ii as u32) + (row_start as u32);
					// debug_assert!((col as u16) < width);
					// debug_assert!((row as u16) < height);
					ret.put_pixel(row, col, pixel);
				}

				let _ = cursor_pix.read_u8().unwrap(); // Dummy
			}
		}

		Some(Self {
			inner: ret,
			offset: Vec2::new(left as f32, top as f32),
		})
	}
}

#[derive(Debug)]
pub struct Palette(pub [Rgba<u8>; 256]);

impl Palette {
	/// A sensible default for internal use. All colors are `0 0 0 255`.
	#[must_use]
	pub(in super::super) fn black() -> Self {
		Self([Rgba([0, 0, 0, 255]); 256])
	}
}

#[derive(Debug)]
pub struct PaletteSet(pub [Palette; 14]);

impl Asset for PaletteSet {
	const KIND: AssetKind = AssetKind::Palettes;

	unsafe fn get(record: &Record) -> &Self {
		&record.asset.palettes
	}

	unsafe fn get_mut(record: &mut Record) -> &mut Self {
		&mut record.asset.palettes
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
