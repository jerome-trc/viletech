//! Textures, sprites, brightmaps, polygonal models, voxel models.

use std::io::Cursor;

use bevy::prelude::Vec2;
use image::{error::ImageFormatHint, ImageError, Rgba32FImage};

use crate::{
	catalog::{PrepError, PrepErrorKind},
	vfs::FileRef,
};

#[derive(Debug)]
pub struct Image {
	pub inner: Rgba32FImage,
	pub offset: Vec2,
}

impl Image {
	/// Returns `None` if the format of `bytes` is unsupported by the `image` crate
	/// (meaning it is likely a picture-format image or not an image at all).
	#[must_use]
	pub fn try_decode(file: FileRef, bytes: &[u8]) -> Option<Result<Image, PrepError>> {
		match image::io::Reader::new(Cursor::new(bytes)).with_guessed_format() {
			Ok(imgr) => match imgr.decode() {
				Ok(img) => Some(Ok(Image {
					inner: img.into_rgba32f(),
					offset: glam::Vec2::ZERO,
				})),
				Err(err) => {
					if let ImageError::Unsupported(e) = &err {
						if e.format_hint() == ImageFormatHint::Unknown {
							return None;
						}
					}

					Some(Err(PrepError {
						path: file.path().to_path_buf(),
						kind: PrepErrorKind::Image(err),
					}))
				}
			},
			Err(err) => Some(Err(PrepError {
				path: file.path().to_path_buf(),
				kind: PrepErrorKind::Io(err),
			})),
		}
	}
}

/// A placeholder type.
#[derive(Debug)]
pub struct PolyModel;

/// A placeholder type.
#[derive(Debug)]
pub struct VoxelModel;
