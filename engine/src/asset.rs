//! Code bridging [`data`] and [`bevy::asset`].

use bevy::{
	prelude::*,
	render::{
		render_resource::{Extent3d, TextureDimension, TextureFormat},
		texture::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
	},
};
use data::gfx::{ColorMap, Palette, PaletteSet, PictureReader};

#[must_use]
pub fn flat_to_image(
	bytes: &[u8],
	palette: &Palette,
	colormap: &ColorMap,
	label: Option<String>,
) -> Image {
	debug_assert_eq!(bytes.len(), 4096);

	let mut buf = Vec::with_capacity(64 * 64);

	for y in 0..64 {
		for x in 0..64 {
			let i = (y * 64) + x;
			let map_entry = bytes[i];
			let pal_entry = colormap[map_entry as usize];
			let pixel = palette[pal_entry as usize];

			buf.push([
				((pixel.r as f32) / 255.0).to_ne_bytes(),
				((pixel.g as f32) / 255.0).to_ne_bytes(),
				((pixel.b as f32) / 255.0).to_ne_bytes(),
				1.0_f32.to_ne_bytes(),
			]);
		}
	}

	let mut img = Image::new(
		Extent3d {
			width: 64,
			height: 64,
			depth_or_array_layers: 1,
		},
		TextureDimension::D2,
		bytemuck::cast_vec(buf),
		TextureFormat::Rgba32Float,
	);

	let sampler = ImageSamplerDescriptor {
		label,
		mag_filter: ImageFilterMode::Nearest,
		min_filter: ImageFilterMode::Nearest,
		mipmap_filter: ImageFilterMode::Nearest,
		address_mode_u: ImageAddressMode::Repeat,
		address_mode_v: ImageAddressMode::Repeat,
		..Default::default()
	};

	img.sampler = ImageSampler::Descriptor(sampler);

	img
}

/// The returned image is a 2D texture array (256 wide, 256 tall, 14 layers)
/// in F32 RGBA format.
#[must_use]
pub fn palset_to_image(palset: &PaletteSet, label: Option<String>) -> Image {
	let mut buf = Vec::with_capacity(256 * 256 * 14);

	match palset {
		PaletteSet::Borrowed(palettes) => {
			for palette in palettes.iter() {
				for color in palette.0 {
					buf.push([
						((color.r as f32) / 255.0).to_ne_bytes(),
						((color.g as f32) / 255.0).to_ne_bytes(),
						((color.b as f32) / 255.0).to_ne_bytes(),
						1.0_f32.to_ne_bytes(),
					]);
				}
			}
		}
		PaletteSet::Owned(palettes) => {
			for palette in palettes.iter() {
				for color in palette.0 {
					buf.push([
						((color.r as f32) / 255.0).to_ne_bytes(),
						((color.g as f32) / 255.0).to_ne_bytes(),
						((color.b as f32) / 255.0).to_ne_bytes(),
						1.0_f32.to_ne_bytes(),
					]);
				}
			}
		}
	}

	let mut img = Image::new(
		Extent3d {
			width: 256,
			height: 256,
			depth_or_array_layers: 14,
		},
		TextureDimension::D2,
		bytemuck::cast_vec(buf),
		TextureFormat::Rgba32Float,
	);

	let sampler = ImageSamplerDescriptor {
		label,
		mag_filter: ImageFilterMode::Nearest,
		min_filter: ImageFilterMode::Nearest,
		mipmap_filter: ImageFilterMode::Nearest,
		address_mode_u: ImageAddressMode::Repeat,
		address_mode_v: ImageAddressMode::Repeat,
		..Default::default()
	};

	img.sampler = ImageSampler::Descriptor(sampler);

	img
}

pub fn picture_to_image(
	bytes: &[u8],
	palette: &Palette,
	colormap: &ColorMap,
	label: Option<String>,
) -> Result<Image, data::Error> {
	let pic_reader = PictureReader::new(bytes)?;

	let width = pic_reader.width() as usize;
	let height = pic_reader.height() as usize;
	let mut buf = vec![[[0_u8; 4]; 4]; width * height];

	pic_reader.read(palette, colormap, |row, col, pixel| {
		buf[col as usize * width + row as usize] = [
			((pixel.r as f32) / 255.0).to_ne_bytes(),
			((pixel.g as f32) / 255.0).to_ne_bytes(),
			((pixel.b as f32) / 255.0).to_ne_bytes(),
			1.0_f32.to_ne_bytes(),
		];
	});

	let mut img = Image::new(
		Extent3d {
			width: width as u32,
			height: height as u32,
			depth_or_array_layers: 1,
		},
		TextureDimension::D2,
		bytemuck::cast_vec(buf),
		TextureFormat::Rgba32Float,
	);

	let sampler = ImageSamplerDescriptor {
		label,
		mag_filter: ImageFilterMode::Nearest,
		min_filter: ImageFilterMode::Nearest,
		mipmap_filter: ImageFilterMode::Nearest,
		address_mode_u: ImageAddressMode::Repeat,
		address_mode_v: ImageAddressMode::Repeat,
		..Default::default()
	};

	img.sampler = ImageSampler::Descriptor(sampler);

	Ok(img)
}
