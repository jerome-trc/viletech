//! Functions for inspecting and manipulating byte slices.

use std::{
	io::{self, Cursor},
	marker::PhantomData,
};

use bytemuck::AnyBitPattern;
use byteorder::{ByteOrder, LittleEndian};

/// Strongly-typed cursor, for easy migration of pointer arithmetic code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TCursor<'b, T>(Cursor<&'b [u8]>, PhantomData<T>)
where
	T: Eq + Copy;

impl<'b, T> TCursor<'b, T>
where
	T: Eq + Copy,
{
	/// `pos` is in bytes, regardless of the type of `T`.
	#[must_use]
	pub fn new(inner: &'b [u8], pos: u64) -> Self {
		let mut ret = Self(Cursor::new(inner), PhantomData);
		ret.0.set_position(pos);
		ret
	}

	/// The returned value is always in bytes, regardless of the type of `T`.
	#[must_use]
	pub fn pos(&self) -> u64 {
		self.0.position()
	}
}

impl TCursor<'_, u8> {
	/// Does not advance the cursor. Panics if out-of-bounds.
	#[must_use]
	pub fn peek_u8_le(&self) -> u8 {
		self.0.get_ref()[self.pos() as usize]
	}

	/// Increase this cursor's position by `w8` bytes.
	pub fn advance(&mut self, w8: u64) -> &mut Self {
		self.0.set_position(self.0.position() + w8);
		self
	}

	/// Advances the cursor by `std::mem::size_of::<A>()` bytes.
	pub fn read_from_bytes<A: AnyBitPattern>(&mut self) -> &A {
		let pos = self.pos() as usize;
		let size = std::mem::size_of::<A>();
		self.advance(size as u64);
		bytemuck::from_bytes(&self.0.get_ref()[pos..(pos + size)])
	}

	#[must_use]
	pub fn to_tcursor32(&self) -> TCursor<u32> {
		let inner = *self.0.get_ref();
		TCursor::new(inner, self.pos() / 4)
	}
}

impl TCursor<'_, u32> {
	/// Does not advance the cursor. Panics if out-of-bounds.
	#[must_use]
	pub fn peek_u32_le(&self) -> u32 {
		let inner = *self.0.get_ref();
		let range = (self.pos() as usize)..(self.pos() as usize + 4);
		LittleEndian::read_u32(&inner[range])
	}

	/// Uses this target's native endianness. Does not advance the cursor.
	/// Panics if out-of-bounds.
	#[must_use]
	pub fn peek_u32(&self) -> u32 {
		let inner = *self.0.get_ref();
		let pos = self.pos() as usize;
		u32::from_ne_bytes([inner[pos], inner[pos + 1], inner[pos + 2], inner[pos + 3]])
	}

	/// Note that `offs` is in terms of `u32`-widths, not bytes.
	/// Uses this target's native endianness. Does not move the cursor.
	/// Panics if out-of-bounds.
	#[must_use]
	pub fn peek_u32_offs(&self, offs: isize) -> u32 {
		let inner = *self.0.get_ref();
		let pos = ((self.pos() as isize) + (offs * 4)) as usize;
		u32::from_ne_bytes([inner[pos], inner[pos + 1], inner[pos + 2], inner[pos + 3]])
	}

	/// Note that `offs` is in terms of `u32`-widths, not bytes.
	/// Does not move the cursor. Panics if out-of-bounds.
	#[must_use]
	pub fn peek_u32_offs_le(&self, offs: isize) -> u32 {
		let inner = *self.0.get_ref();
		let pos = ((self.pos() as isize) + (offs * 4)) as usize;
		u32::from_le_bytes([inner[pos], inner[pos + 1], inner[pos + 2], inner[pos + 3]])
	}

	/// Increase this cursor's position by `w32 * 4` bytes.
	pub fn advance(&mut self, w32: u64) -> &mut Self {
		self.0.set_position(self.0.position() + (w32 * 4));
		self
	}

	#[must_use]
	pub fn to_tcursor8(&self) -> TCursor<u32> {
		let inner = *self.0.get_ref();
		TCursor::new(inner, self.pos() * 4)
	}
}

/// Extends [`Cursor`] with some convenience functions.
pub trait CursorExt {
	/// Shorthand for `self.set_position(self.position() + bytes)`.
	fn advance(&mut self, bytes: u64);

	/// See [`bytemuck::from_bytes`]. Advances the cursor by the size of `A`.
	#[must_use]
	fn read_from_bytes<A: AnyBitPattern>(&mut self) -> &A;
}

impl<T> CursorExt for Cursor<T>
where
	T: AsRef<[u8]>,
{
	fn advance(&mut self, bytes: u64) {
		self.set_position(self.position() + bytes)
	}

	fn read_from_bytes<A: AnyBitPattern>(&mut self) -> &A {
		let pos = self.position() as usize;
		let size = std::mem::size_of::<A>();
		self.advance(size as u64);
		bytemuck::from_bytes(&self.get_ref().as_ref()[pos..(pos + size)])
	}
}

/// Checks for a 4-byte magic number.
/// Ensure the given slice starts at the file's beginning.
#[must_use]
pub fn is_zip(bytes: &[u8]) -> bool {
	bytes.len() >= 4 && matches!(&bytes[0..4], &[0x50, 0x4b, 0x03, 0x04])
}

/// Checks for a 13-byte series of properties.
/// Ensure the given slice starts at the file's beginning.
#[must_use]
pub fn is_lzma(bytes: &[u8]) -> bool {
	// (RAT) I have limited reason to believe this is sound. No good formal spec for
	// LZMA's header seems to exist *anywhere*. I just compressed some files, passed
	// them through integrity tests via CLI, and then read the headers in those.
	bytes.len() >= 13
		&& matches!(
			&bytes[0..13],
			&[0x5D, 0x00, 0x00, 0x80, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
		)
}

/// Checks for:
/// - A byte length that's a multiple of 4.
/// - A 6-byte magic number in the header.
/// - A 2-byte magic number in the footer.
#[must_use]
pub fn is_xz(header: &[u8], footer: &[u8], file_len: u64) -> bool {
	// http://fileformats.archiveteam.org/wiki/XZ
	(file_len % 4) == 0
		&& header.len() >= 6
		&& footer.len() >= 2
		&& matches!(&header[0..6], &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00])
		&& matches!(&footer[(footer.len() - 2)..], &[0x59, 0x5A])
}

/// Checks for the 4-byte magic number, directory info, and that the file size is
/// as expected given the number of entries. `len` should be the entire WAD's file
/// length, regardless of the length of `bytes`.
pub fn is_valid_wad(bytes: &[u8], len: u64) -> io::Result<bool> {
	if len < 12 {
		return Ok(false);
	}

	match &bytes[0..4] {
		b"IWAD" | b"PWAD" => {}
		_ => {
			return Ok(false);
		}
	};

	let num_entries = LittleEndian::read_i32(&bytes[4..8]);
	let dir_offs = LittleEndian::read_i32(&bytes[8..12]);

	if num_entries < 0 || dir_offs < 0 {
		return Ok(false);
	}

	let expected_dir_len = match num_entries.checked_mul(16) {
		Some(edl) => edl,
		None => {
			return Ok(false);
		}
	};

	let expected_bin_len = match dir_offs.checked_add(expected_dir_len) {
		Some(ebl) => ebl,
		None => {
			return Ok(false);
		}
	};

	Ok(len >= expected_bin_len as u64)
}

/// Checks for an 8-byte signature.
#[must_use]
pub fn is_png(bytes: &[u8]) -> bool {
	bytes.len() > 8
		&& matches!(
			&bytes[0..8],
			&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
		)
}

/// Checks the header and total size. Ensure a slice over the entire lump is given.
#[must_use]
pub fn is_doom_gfx(bytes: &[u8]) -> bool {
	const HEADER_SIZE: usize = 8;

	if bytes.len() < HEADER_SIZE {
		return false;
	}

	let width = LittleEndian::read_i16(&bytes[0..2]);
	let height = LittleEndian::read_i16(&bytes[2..4]);
	let left = LittleEndian::read_i16(&bytes[4..6]);
	let top = LittleEndian::read_i16(&bytes[6..8]);

	// Sanity check on dimensions.
	if !(0..=4096).contains(&width) {
		return false;
	}

	if !(0..=4096).contains(&height) {
		return false;
	}

	if !(-2000..=2000).contains(&top) {
		return false;
	}

	if !(-2000..=2000).contains(&left) {
		return false;
	}

	if bytes.len() < (HEADER_SIZE + ((width as usize) * 4)) {
		return false;
	}

	for col in 0..width {
		let i = col as usize;
		let start = HEADER_SIZE + i;
		let end = start + 4;
		let col_offs = LittleEndian::read_u32(&bytes[start..end]) as usize;

		if col_offs > bytes.len() || col_offs < HEADER_SIZE {
			return false;
		}
	}

	let n_pix = ((height + 2) + (height % 2)) / 2;
	let max_col_size = (4 + (n_pix * 5) + 1) as usize;

	if bytes.len() > (HEADER_SIZE + (width as usize) * max_col_size) {
		return false;
	}

	true
}

/// Source: <https://docs.rs/infer/latest/src/infer/matchers/archive.rs.html#59-67>
#[must_use]
pub fn is_7z(bytes: &[u8]) -> bool {
	bytes.len() > 5
		&& bytes[0] == 0x37
		&& bytes[1] == 0x7A
		&& bytes[2] == 0xBC
		&& bytes[3] == 0xAF
		&& bytes[4] == 0x27
		&& bytes[5] == 0x1C
}
