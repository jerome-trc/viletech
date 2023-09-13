//! Graphics-related representations.

use std::io::Cursor;

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use glam::{IVec2, UVec2};
use image::Rgba;
use util::{io::CursorExt, Id8};

use crate::Error;

/// See <https://doomwiki.org/wiki/COLORMAP>.
#[derive(Debug, Clone)]
pub struct ColorMap(
	/// A canonical COLORMAP has 34 elements in this slice.
	pub Box<[[u8; 256]]>,
);

impl ColorMap {
	pub fn new(bytes: &[u8]) -> Result<Self, Error> {
		if bytes.len() != (34 * 256) {
			return Err(Error::SizeMismatch {
				expected: 34 * 256,
				actual: bytes.len(),
			});
		}

		let mut ret = vec![];
		let mut i = 0;

		for _ in 0..34 {
			let mut subarr = [0; 256];

			for byte in subarr.iter_mut() {
				*byte = bytes[i];
				i += 1;
			}

			ret.push(subarr);
		}

		Ok(Self(ret.into_boxed_slice()))
	}
}

/// See <https://doomwiki.org/wiki/ENDOOM>.
#[derive(Debug, Clone)]
pub struct EnDoom {
	pub colors: Box<[u8; 2000]>,
	pub text: Box<[u8; 2000]>,
}

impl EnDoom {
	pub fn new(bytes: &[u8]) -> Result<Self, Error> {
		if bytes.len() != 4000 {
			return Err(Error::SizeMismatch {
				expected: 4000,
				actual: bytes.len(),
			});
		}

		let mut ret = Self {
			colors: Box::new([0; 2000]),
			text: Box::new([0; 2000]),
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

	#[must_use]
	pub fn is_blinking(&self, index: usize) -> bool {
		assert!(index < 2000);
		self.colors[index] & (1 << 7) == (1 << 7)
	}
}

#[derive(Debug)]
pub struct Palette(pub [Rgba<f32>; 256]);

impl Palette {
	/// A sensible default for internal use. All colors are `0.0 0.0 0.0 1.0`.
	#[must_use]
	fn black() -> Self {
		Self([Rgba([0.0, 0.0, 0.0, 1.0]); 256])
	}
}

#[derive(Debug)]
pub struct PaletteSet(
	/// A canonical PLAYPAL set has 14 elements in this slice.
	pub Box<[Palette]>,
);

impl PaletteSet {
	pub fn new(bytes: &[u8]) -> Result<Self, Error> {
		let mut palettes = vec![];
		let mut cursor = Cursor::new(bytes);

		for _ in 0..14 {
			let mut pal = Palette::black();
			let expected = (cursor.position() + 3) as usize;

			for ii in 0..256 {
				let r = (cursor.read_u8().map_err(|_| Error::MissingRecord {
					expected,
					actual: bytes.len(),
				})? as f32) / 255.0;
				let g = (cursor.read_u8().map_err(|_| Error::MissingRecord {
					expected,
					actual: bytes.len(),
				})? as f32) / 255.0;
				let b = (cursor.read_u8().map_err(|_| Error::MissingRecord {
					expected,
					actual: bytes.len(),
				})? as f32) / 255.0;

				pal.0[ii] = Rgba([r, g, b, 255.0]);
			}

			palettes.push(pal);
		}

		Ok(PaletteSet(palettes.into_boxed_slice()))
	}
}

/// See <https://doomwiki.org/wiki/PNAMES>.
#[derive(Debug, Default)]
pub struct PatchTable(pub Vec<Id8>);

impl std::ops::Deref for PatchTable {
	type Target = Vec<Id8>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl std::ops::DerefMut for PatchTable {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl PatchTable {
	/// Returns `Ok(None)` if the given PNAMES lump is valid,
	/// but reports itself to have 0 records in it.
	pub fn new(bytes: &[u8]) -> Result<Option<Self>, Error> {
		const RECORD_SIZE: usize = 8;

		let mut invalid = false;

		invalid |= bytes.len() < RECORD_SIZE;
		invalid |= ((bytes.len() - 4) % RECORD_SIZE) != 0;

		let len = LittleEndian::read_u32(bytes) as usize;

		if len == 0 {
			return Ok(None);
		}

		let expected = (len * RECORD_SIZE) + 4;
		invalid |= bytes.len() != expected;

		if invalid {
			return Err(Error::SizeMismatch {
				expected,
				actual: bytes.len(),
			});
		}

		let mut ret = Vec::with_capacity(len);
		let mut pos = 4;

		while pos < bytes.len() {
			let raw = bytemuck::from_bytes::<[u8; RECORD_SIZE]>(&bytes[pos..(pos + RECORD_SIZE)]);

			if let Some(pname) = util::read_id8(*raw) {
				ret.push(pname);
			}

			pos += RECORD_SIZE;
		}

		Ok(Some(Self(ret)))
	}
}

/// See <https://doomwiki.org/wiki/TEXTURE1_and_TEXTURE2>.
#[derive(Debug, Default)]
pub struct TextureX(pub Vec<PatchedTex>);

impl std::ops::Deref for TextureX {
	type Target = Vec<PatchedTex>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl std::ops::DerefMut for TextureX {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

/// See [`TextureX`].
#[derive(Debug)]
pub struct PatchedTex {
	pub name: Id8,
	pub size: UVec2,
	pub patches: Vec<TexPatch>,
}

/// See [`PatchedTex`].
#[derive(Debug)]
pub struct TexPatch {
	/// Offset of this patch relative to the upper-left of the whole texture.
	pub origin: IVec2,
	/// Index into [`PatchTable`].
	pub index: usize,
}

impl TextureX {
	pub fn new(bytes: &[u8]) -> Result<Option<Self>, Error> {
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

		if bytes.len() < 4 {
			return Err(Error::MissingHeader { expected: 4 });
		}

		let num_textures = LittleEndian::read_u32(bytes) as usize;

		if num_textures == 0 {
			return Ok(None);
		}

		let mut curs_maptex = Cursor::new(bytes);
		curs_maptex.set_position(curs_maptex.position() + 4);

		let mut ret = Vec::with_capacity(num_textures);

		for _ in 0..num_textures {
			let start = curs_maptex.read_i32::<LittleEndian>().unwrap() as usize;
			let end = start + 22;

			if end > bytes.len() {
				return Err(Error::MissingRecord {
					expected: end,
					actual: bytes.len(),
				});
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
					return Err(Error::MissingRecord {
						expected: end as usize,
						actual: bytes.len(),
					});
				}

				let range = (curs_patch.position() as usize)..(end as usize);
				let raw_patch = bytemuck::from_bytes::<RawMapPatch>(&bytes[range]);

				patches.push(TexPatch {
					origin: glam::ivec2(raw_patch.origin_x as i32, raw_patch.origin_y as i32),
					index: raw_patch.patch as usize,
				});

				curs_patch.set_position(end);
			}

			ret.push(PatchedTex {
				name: util::read_id8(raw_tex.name).unwrap_or_default(),
				size: glam::uvec2(raw_tex.width as u32, raw_tex.height as u32),
				patches,
			});
		}

		Ok(Some(Self(ret)))
	}
}
