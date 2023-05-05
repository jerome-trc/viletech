//! Functions for processing formats shipped with the original commercial IWADs.

use std::io::Cursor;

use arrayvec::ArrayVec;
use bevy::prelude::{IVec2, UVec2};
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use image::Rgba;

use crate::{
	data::{
		detail::Outcome, prep::read_shortid, vfs::FileRef, AssetHeader, Catalog, ColorMap, EnDoom,
		Image, Palette, PaletteSet, PrepError, PrepErrorKind,
	},
	utils::io::CursorExt,
	ShortId,
};

use super::SubContext;

/// See <https://doomwiki.org/wiki/PNAMES>.
#[derive(Debug)]
pub(super) struct PatchTable(pub(super) Vec<ShortId>);

/// See <https://doomwiki.org/wiki/TEXTURE1_and_TEXTURE2>.
#[derive(Debug)]
pub(super) struct TextureX(pub(super) Vec<PatchedTex>);

#[derive(Debug)]
pub(super) struct PatchedTex {
	pub(super) _name: ShortId,
	pub(super) _size: UVec2,
	pub(super) _patches: Vec<TexPatch>,
}

#[derive(Debug)]
pub(super) struct TexPatch {
	/// Offset of this patch relative to the upper-left of the whole texture.
	pub(super) _origin: IVec2,
	/// Index into [`PatchTable`].
	pub(super) _index: usize,
}

impl Catalog {
	pub(super) fn prep_colormap(
		&self,
		lump: FileRef,
		bytes: &[u8],
	) -> Result<ColorMap, Box<PrepError>> {
		if bytes.len() != (34 * 256) {
			return Err(Box::new(PrepError {
				path: lump.path.to_path_buf(),
				kind: PrepErrorKind::ColorMap(bytes.len()),
				fatal: false,
			}));
		}

		let mut ret = ColorMap::black();
		let mut i = 0;

		for subarr in ret.0.iter_mut() {
			for byte in subarr {
				*byte = bytes[i];
				i += 1;
			}
		}

		Ok(ret)
	}

	pub(super) fn prep_endoom(
		&self,
		lump: FileRef,
		bytes: &[u8],
	) -> Result<EnDoom, Box<PrepError>> {
		if bytes.len() != 4000 {
			return Err(Box::new(PrepError {
				path: lump.path.to_path_buf(),
				kind: PrepErrorKind::EnDoom(bytes.len()),
				fatal: false,
			}));
		}

		let mut ret = EnDoom {
			colors: [0; 2000],
			text: [0; 2000],
		};

		let mut r_i = 0;
		let mut b_i = 0;

		while b_i < 4000 {
			ret.colors[r_i] = bytes[b_i];
			ret.text[r_i] = bytes[b_i + 1];
			r_i += 1;
			b_i += 2;
		}

		Ok(ret)
	}

	/// Returns `None` to indicate that `bytes` was checked
	/// and determined to not be a picture.
	#[must_use]
	pub(super) fn prep_picture(&self, ctx: &SubContext, bytes: &[u8], id: &str) -> Option<()> {
		let palettes = self.last_paletteset().unwrap();

		if let Some(image) = Image::try_from_picture(bytes, &palettes.0[0]) {
			ctx.add_asset::<Image>(Image {
				header: AssetHeader {
					id: format!("{mount_id}/{id}", mount_id = ctx.mntinfo.id()),
				},
				inner: image.0,
				offset: image.1,
			});

			Some(())
		} else {
			None
		}
	}

	pub(super) fn prep_playpal(
		&self,
		_ctx: &SubContext,
		bytes: &[u8],
	) -> std::io::Result<PaletteSet> {
		let mut palettes = ArrayVec::<_, 14>::default();
		let mut cursor = Cursor::new(bytes);

		for _ in 0..14 {
			let mut pal = Palette::black();

			for ii in 0..256 {
				let r = cursor.read_u8()?;
				let g = cursor.read_u8()?;
				let b = cursor.read_u8()?;
				pal.0[ii] = Rgba([r, g, b, 255]);
			}

			palettes.push(pal);
		}

		Ok(PaletteSet(palettes.into_inner().unwrap()))
	}

	/// Returns `None` if the given PNAMES lump is valid, but reports itself to
	/// have 0 records in it.
	pub(super) fn prep_pnames(
		&self,
		_ctx: &SubContext,
		lump: FileRef,
		bytes: &[u8],
	) -> Outcome<PatchTable, PrepError> {
		const RECORD_SIZE: usize = 8;

		let mut invalid = false;

		invalid |= bytes.len() < RECORD_SIZE;
		invalid |= ((bytes.len() - 4) % RECORD_SIZE) != 0;

		let len = LittleEndian::read_u32(bytes) as usize;

		if len == 0 {
			return Outcome::None;
		}

		invalid |= bytes.len() != ((len * RECORD_SIZE) + 4);

		if invalid {
			return Outcome::Err(PrepError {
				path: lump.path.to_path_buf(),
				kind: PrepErrorKind::PNames,
				fatal: false,
			});
		}

		let mut ret = Vec::with_capacity(len);
		let mut pos = 4;

		while pos < bytes.len() {
			let raw = bytemuck::from_bytes::<[u8; RECORD_SIZE]>(&bytes[pos..(pos + RECORD_SIZE)]);

			if let Some(pname) = read_shortid(*raw) {
				ret.push(pname);
			}

			pos += RECORD_SIZE;
		}

		Outcome::Ok(PatchTable(ret))
	}

	pub(super) fn prep_texturex(
		&self,
		_ctx: &SubContext,
		lump: FileRef,
		bytes: &[u8],
	) -> Outcome<TextureX, PrepError> {
		#[repr(C)]
		#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
		struct RawMapTexture {
			name: [u8; 8],
			/// C boolean, unused.
			masked: i32,
			width: i16,
			height: i16,
			/// Unused.
			col_dir: i32,
			patch_count: i16,
		}

		#[repr(C)]
		#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
		struct RawMapPatch {
			origin_x: i16,
			origin_y: i16,
			/// Index into [`PatchTable`].
			patch: i16,
			/// Unused.
			stepdir: i16,
			/// Unused.
			colormap: i16,
		}

		let err_fn = || PrepError {
			path: lump.path.to_path_buf(),
			kind: PrepErrorKind::TextureX,
			fatal: false,
		};

		if bytes.len() < 4 {
			return Outcome::Err(err_fn());
		}

		let num_textures = LittleEndian::read_u32(bytes) as usize;

		if num_textures == 0 {
			return Outcome::None;
		}

		let mut curs_maptex = Cursor::new(bytes);
		curs_maptex.set_position(curs_maptex.position() + 4);

		let mut ret = Vec::with_capacity(num_textures);

		for _ in 0..num_textures {
			let start = curs_maptex.read_i32::<LittleEndian>().unwrap() as usize;
			let end = start + 22;

			if end > bytes.len() {
				return Outcome::Err(err_fn());
			}

			let mut curs_patch = Cursor::new(bytes);
			curs_patch.set_position(start as u64);

			let raw_tex = RawMapTexture {
				name: *curs_patch.read_from_bytes::<[u8; 8]>(),
				masked: curs_patch.read_i32::<LittleEndian>().unwrap(),
				width: curs_patch.read_i16::<LittleEndian>().unwrap(),
				height: curs_patch.read_i16::<LittleEndian>().unwrap(),
				col_dir: curs_patch.read_i32::<LittleEndian>().unwrap(),
				patch_count: curs_patch.read_i16::<LittleEndian>().unwrap(),
			};

			let patch_count = raw_tex.patch_count as usize;

			debug_assert_eq!(curs_patch.position(), end as u64);

			let mut patches = Vec::with_capacity(patch_count);

			for _ in 0..patch_count {
				let end = curs_patch.position() + (std::mem::size_of::<RawMapPatch>() as u64);

				if end as usize > bytes.len() {
					return Outcome::Err(err_fn());
				}

				let range = (curs_patch.position() as usize)..(end as usize);
				let raw_patch = bytemuck::from_bytes::<RawMapPatch>(&bytes[range]);

				patches.push(TexPatch {
					_origin: glam::ivec2(raw_patch.origin_x as i32, raw_patch.origin_y as i32),
					_index: raw_patch.patch as usize,
				});

				curs_patch.set_position(end);
			}

			ret.push(PatchedTex {
				_name: read_shortid(raw_tex.name).unwrap_or_default(),
				_size: glam::uvec2(raw_tex.width as u32, raw_tex.height as u32),
				_patches: patches,
			});
		}

		Outcome::Ok(TextureX(ret))
	}
}