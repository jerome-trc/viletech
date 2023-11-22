//! Graphics-related representations.

use std::io::Cursor;

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use util::{io::CursorExt, Id8};

use crate::Error;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Rgb8 {
	pub r: u8,
	pub g: u8,
	pub b: u8,
}

/// See <https://doomwiki.org/wiki/COLORMAP> (and [`ColorMapSet`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorMap(pub [u8; 256]);

impl std::ops::Deref for ColorMap {
	type Target = [u8; 256];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl std::ops::DerefMut for ColorMap {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

/// See <https://doomwiki.org/wiki/COLORMAP>.
///
/// This type is meant to resemble a [`std::borrow::Cow`], since its content
/// can be cheaply cast directly from raw bytes and used thus.
#[derive(Debug, Clone)]
pub enum ColorMapSet<'b> {
	Borrowed(&'b [ColorMap; 34]),
	Owned(Box<[ColorMap; 34]>),
}

impl ColorMapSet<'_> {
	/// This returns a [`Self::Borrowed`] by casting the bytes directly,
	/// so it is allocation-free and very cheap.
	pub fn new(bytes: &[u8]) -> Result<Self, Error> {
		#[repr(C)]
		#[derive(Clone, Copy, bytemuck::AnyBitPattern)]
		struct Raw([[u8; 256]; 34]);

		if bytes.len() < (256 * 34) {
			return Err(Error::SizeMismatch {
				expected: 256 * 34,
				actual: bytes.len(),
			});
		}

		let raw = bytemuck::from_bytes::<Raw>(bytes);

		// SAFETY: `Raw` and `[ColorMap; 34]` have identical representations.
		unsafe { Ok(Self::Borrowed(std::mem::transmute(raw))) }
	}

	#[must_use]
	pub fn into_owned(self) -> [ColorMap; 34] {
		match self {
			Self::Borrowed(b) => b.to_owned(),
			Self::Owned(o) => *o,
		}
	}
}

impl std::ops::Index<usize> for ColorMapSet<'_> {
	type Output = ColorMap;

	fn index(&self, index: usize) -> &Self::Output {
		match self {
			Self::Borrowed(r) => &r[index],
			Self::Owned(b) => &b[index],
		}
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

/// See <https://doomwiki.org/wiki/PLAYPAL> (and [`PaletteSet`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette(pub [Rgb8; 256]);

impl std::ops::Deref for Palette {
	type Target = [Rgb8; 256];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl std::ops::DerefMut for Palette {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

/// See <https://doomwiki.org/wiki/PLAYPAL>.
///
/// This type is meant to resemble a [`std::borrow::Cow`], since its content
/// can be cheaply cast directly from raw bytes and used thus.
#[derive(Debug)]
pub enum PaletteSet<'b> {
	Borrowed(&'b [Palette; 14]),
	Owned(Box<[Palette; 14]>),
}

impl PaletteSet<'_> {
	/// This returns a [`Self::Borrowed`] by casting the bytes directly to
	/// [`Palette`]s, so it is allocation-free and very cheap.
	pub fn new(bytes: &[u8]) -> Result<Self, Error> {
		#[repr(C)]
		#[derive(Clone, Copy, bytemuck::AnyBitPattern)]
		struct Raw([[[u8; 3]; 256]; 14]);

		if bytes.len() < (256 * 3 * 14) {
			return Err(Error::SizeMismatch {
				expected: 256 * 3 * 14,
				actual: bytes.len(),
			});
		}

		let raw = bytemuck::from_bytes::<Raw>(bytes);

		// SAFETY: `Raw` and `[Palette; 14]` have identical representations.
		unsafe { Ok(Self::Borrowed(std::mem::transmute(raw))) }
	}

	#[must_use]
	pub fn into_owned(self) -> [Palette; 14] {
		match self {
			Self::Borrowed(b) => b.to_owned(),
			Self::Owned(o) => *o,
		}
	}
}

impl std::ops::Index<usize> for PaletteSet<'_> {
	type Output = Palette;

	fn index(&self, index: usize) -> &Self::Output {
		match self {
			Self::Borrowed(r) => &r[index],
			Self::Owned(b) => &b[index],
		}
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

/// See <https://doomwiki.org/wiki/Picture_format>.
///
/// Partially adapted from SLADE's `DoomGfxDataFormat::isThisFormat`.
#[derive(Debug)]
pub struct PictureReader<'a> {
	bytes: &'a [u8],
	/// Short for "header cursor".
	cursor_h: Cursor<&'a [u8]>,
	/// The position just past the header.
	checkpoint: u64,
	width: u16,
	height: u16,
	left: i16,
	top: i16,
}

impl<'a> PictureReader<'a> {
	const HEADER_SIZE: usize = std::mem::size_of::<u16>() * 4;

	/// Ensure that `bytes` is the entire lump.
	/// This does not allocate, so `PictureReader::new.is_ok()` is a suitable
	/// way to check if a WAD entry is a picture-format graphic.
	pub fn new(bytes: &'a [u8]) -> Result<PictureReader<'a>, Error> {
		if bytes.len() < Self::HEADER_SIZE {
			return Err(Error::MissingHeader {
				expected: Self::HEADER_SIZE,
			});
		}

		let mut cursor_h = Cursor::new(bytes);

		let width = cursor_h.read_u16::<LittleEndian>().unwrap();
		let height = cursor_h.read_u16::<LittleEndian>().unwrap();
		let left = cursor_h.read_i16::<LittleEndian>().unwrap();
		let top = cursor_h.read_i16::<LittleEndian>().unwrap();

		// (SLADE) Sanity checks on dimensions and offsets.

		if width >= 4096 || height >= 4096 {
			return Err(Error::InvalidHeader {
				details: "width or height is >= 4096",
			});
		}

		if left <= -2000 || left >= 2000 {
			return Err(Error::InvalidHeader {
				details: "left <= -2000 or >= 2000",
			});
		}

		if top <= -2000 || top >= 2000 {
			return Err(Error::InvalidHeader {
				details: "top <= -2000 or >= 2000",
			});
		}

		if bytes.len() < (Self::HEADER_SIZE + (width as usize * 4)) {
			return Err(Error::InvalidHeader {
				details: "lump length < (header size + width)",
			});
		}

		let checkpoint = cursor_h.position(); // Just after the header.

		for _ in 0..width {
			let col_offs = cursor_h.read_u32::<LittleEndian>().unwrap() as usize;

			if col_offs > bytes.len() || col_offs < (Self::HEADER_SIZE) {
				return Err(Error::InvalidHeader {
					details: "column offset > lump length OR column offset < header size",
				});
			}

			// (SLADE) Check if total size is reasonable; this computation corresponds
			// to the most inefficient possible use of space by the format
			// (horizontal stripes of 1 pixel, 1 pixel apart).
			let num_pixels = ((height + 2 + height % 2) / 2) as usize;
			let max_col_size = std::mem::size_of::<u32>() + (num_pixels * 5) + 1;

			if bytes.len() > Self::HEADER_SIZE + (width as usize * max_col_size) {
				// Q: Unlikely, but possible. Should we try?
				return Err(Error::InvalidHeader {
					details: "lump length > (header size + (width times maximum column size))",
				});
			}
		}

		Ok(Self {
			bytes,
			cursor_h,
			checkpoint,
			width,
			height,
			left,
			top,
		})
	}

	#[must_use]
	pub fn width(&self) -> u16 {
		self.width
	}

	#[must_use]
	pub fn height(&self) -> u16 {
		self.height
	}

	/// The first element is the offset in pixels to the left of the origin;
	/// the second element is the offset below the origin.
	#[must_use]
	pub fn offset(&self) -> (i16, i16) {
		(self.left, self.top)
	}

	/// `callback`'s first two parameters are a row and column index respectively.
	pub fn read<F>(mut self, palette: &Palette, colormap: &ColorMap, mut callback: F)
	where
		F: FnMut(u32, u32, Rgb8),
	{
		let mut cursor_pix = Cursor::new(self.bytes);
		self.cursor_h.set_position(self.checkpoint);

		for i in 0..self.width {
			let col_offs = self.cursor_h.read_u32::<LittleEndian>().unwrap() as u64;
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
					let map_entry = cursor_pix.read_u8().unwrap();
					let pal_entry = colormap[map_entry as usize];
					let pixel = palette[pal_entry as usize];
					let row = i as u32;
					let col = (ii as u32) + (row_start as u32);
					callback(row, col, pixel);
				}

				let _ = cursor_pix.read_u8().unwrap(); // Dummy
			}
		}
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
	pub size_x: u32,
	pub size_y: u32,
	pub patches: Vec<TexPatch>,
}

/// See [`PatchedTex`].
#[derive(Debug)]
pub struct TexPatch {
	/// X-offset of this patch relative to the upper-left of the whole texture.
	pub origin_x: i32,
	/// Y-offset of this patch relative to the upper-left of the whole texture.
	pub origin_y: i32,
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
					origin_x: raw_patch.origin_x as i32,
					origin_y: raw_patch.origin_y as i32,
					index: raw_patch.patch as usize,
				});

				curs_patch.set_position(end);
			}

			ret.push(PatchedTex {
				name: util::read_id8(raw_tex.name).unwrap_or_default(),
				size_x: raw_tex.width as u32,
				size_y: raw_tex.height as u32,
				patches,
			});
		}

		Ok(Some(Self(ret)))
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn picture_reader() {
		let pic = include_bytes!("../../sample/freedoom/STFST01.lmp");
		let reader = PictureReader::new(pic).unwrap();
		assert_eq!(reader.width(), 24);
		assert_eq!(reader.height(), 29);
	}
}
