//! Functions for processing formats shipped with the original commercial IWADs.

use data::gfx::{ColorMap, EnDoom, PaletteSet, PatchTable, TextureX};
use glam::Vec2;
use image::ImageBuffer;
use util::Outcome;

use crate::{
	catalog::{dobj::Image, Catalog, PrepError, PrepErrorKind},
	vfs::FileRef,
};

use super::SubContext;

impl Catalog {
	pub(super) fn prep_colormap(&self, lump: FileRef, bytes: &[u8]) -> Result<ColorMap, PrepError> {
		match ColorMap::new(bytes) {
			Ok(cmap) => Ok(cmap),
			Err(err) => Err(PrepError {
				path: lump.path().to_path_buf(),
				kind: PrepErrorKind::ColorMap(err),
			}),
		}
	}

	pub(super) fn prep_endoom(
		&self,
		lump: FileRef,
		bytes: &[u8],
	) -> Result<EnDoom, Box<PrepError>> {
		match EnDoom::new(bytes) {
			Ok(endoom) => Ok(endoom),
			Err(err) => Err(Box::new(PrepError {
				path: lump.path().to_path_buf(),
				kind: PrepErrorKind::EnDoom(err),
			})),
		}
	}

	pub(super) fn prep_flat(
		&self,
		ctx: &SubContext,
		lump: FileRef,
		bytes: &[u8],
	) -> Result<Image, Box<PrepError>> {
		if bytes.len() != (64 * 64) {
			return Err(Box::new(PrepError {
				path: lump.path().to_path_buf(),
				kind: PrepErrorKind::Flat,
			}));
		}

		let palettes = ctx.higher.last_paletteset().unwrap();
		let palette = &palettes.0[0];
		let mut ret = ImageBuffer::new(64, 64);

		for y in 0..64 {
			for x in 0..64 {
				let palndx = bytes[x + (y * 64)];
				let pixel = palette.0[palndx as usize];
				ret.put_pixel(x as u32, y as u32, pixel);
			}
		}

		Ok(Image {
			inner: ret,
			offset: Vec2::default(),
		})
	}

	/// Returns `None` to indicate that `bytes` was checked
	/// and determined to not be a picture.
	#[must_use]
	pub(super) fn prep_picture(&self, ctx: &SubContext, bytes: &[u8]) -> Option<Image> {
		let palettes = ctx.higher.last_paletteset().unwrap();
		let opt = data::gfx::try_from_picture(bytes, &palettes.0[0]);
		opt.map(|(ibuf, offs)| Image {
			inner: ibuf,
			offset: offs,
		})
	}

	pub(super) fn prep_playpal(
		&self,
		_ctx: &SubContext,
		lump: FileRef,
		bytes: &[u8],
	) -> Outcome<PaletteSet, PrepError> {
		match PaletteSet::new(bytes) {
			Ok(palset) => Outcome::Ok(palset),
			Err(err) => Outcome::Err(PrepError {
				path: lump.path().to_path_buf(),
				kind: PrepErrorKind::PNames(err),
			}),
		}
	}

	/// Returns `None` if the given PNAMES lump is valid, but reports itself to
	/// have 0 records in it.
	pub(super) fn prep_pnames(
		&self,
		_ctx: &SubContext,
		lump: FileRef,
		bytes: &[u8],
	) -> Outcome<PatchTable, PrepError> {
		match PatchTable::new(bytes) {
			Ok(option) => match option {
				Some(pnames) => Outcome::Ok(pnames),
				None => Outcome::None,
			},
			Err(err) => Outcome::Err(PrepError {
				path: lump.path().to_path_buf(),
				kind: PrepErrorKind::PNames(err),
			}),
		}
	}

	pub(super) fn prep_texturex(
		&self,
		_ctx: &SubContext,
		lump: FileRef,
		bytes: &[u8],
	) -> Outcome<TextureX, PrepError> {
		match TextureX::new(bytes) {
			Ok(option) => match option {
				Some(texx) => Outcome::Ok(texx),
				None => Outcome::None,
			},
			Err(err) => Outcome::Err(PrepError {
				path: lump.path().to_path_buf(),
				kind: PrepErrorKind::TextureX(err),
			}),
		}
	}
}
