use bevy::{
	prelude::*,
	render::{
		render_resource::{Extent3d, TextureDimension, TextureFormat},
		texture::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
	},
};
use data::gfx::{ColorMap, Palette, PictureReader};
use image::{Rgba, Rgba32FImage};

pub fn flat_to_image(
	bytes: &[u8],
	palette: &Palette,
	colormap: &ColorMap,
	label: Option<String>,
) -> Image {
	debug_assert_eq!(bytes.len(), 4096);

	let mut img_buf = Rgba32FImage::new(64, 64);

	for y in 0..64 {
		for x in 0..64 {
			let i = (y * 64) + x;
			let map_entry = bytes[i];
			let pal_entry = colormap[map_entry as usize];
			let pixel = palette[pal_entry as usize];

			img_buf.put_pixel(
				x as u32,
				y as u32,
				Rgba([
					(pixel[0] as f32) / 255.0,
					(pixel[1] as f32) / 255.0,
					(pixel[2] as f32) / 255.0,
					1.0,
				]),
			);
		}
	}

	let mut img = Image::new(
		Extent3d {
			width: img_buf.width(),
			height: img_buf.height(),
			depth_or_array_layers: 1,
		},
		TextureDimension::D2,
		img_buf
			.into_vec()
			.into_iter()
			.flat_map(|float| float.to_ne_bytes())
			.collect(), // TODO: can we avoid re-allocating here?
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
	let pic_reader = PictureReader::new(bytes, palette, colormap)?;
	let img_buf = pic_reader.read_rgba32()?;

	let mut img = Image::new(
		Extent3d {
			width: img_buf.width(),
			height: img_buf.height(),
			depth_or_array_layers: 1,
		},
		TextureDimension::D2,
		img_buf
			.into_vec()
			.into_iter()
			.flat_map(|float| float.to_ne_bytes())
			.collect(), // TODO: can we avoid re-allocating here?
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
