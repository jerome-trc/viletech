//! Functions for inspecting and manipulating byte slices.

/*
Copyright (C) 2022 ***REMOVED***

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

use std::io;

use byteorder::{ByteOrder, LittleEndian};

/// Check if the given byte slice is not ASCII-encoded, UTF-8 encoded, or UTF-16 encoded.
pub fn is_binary(bytes: &[u8]) -> bool {
	if std::str::from_utf8(bytes).is_ok() || bytes.is_ascii() {
		return false;
	}

	let slice16 = match bytemuck::try_cast_slice::<u8, u16>(bytes) {
		Ok(s) => s,
		Err(_) => {
			return true;
		}
	};

	// Check for two consecutive NUL characters
	let mut nul = false;
	let end = std::cmp::min(slice16.len(), 512);

	for c16 in &slice16[0..end] {
		if nul {
			if *c16 == 0 {
				return false;
			} else {
				nul = false;
			}
		} else if *c16 == 0 {
			nul = true;
		}
	}

	true
}

/// Checks for a 4-byte magic number at the very beginning of the file.
pub fn is_zip(bytes: &[u8]) -> bool {
	bytes.len() >= 4 && matches!(&bytes[0..4], &[0x50, 0x4b, 0x03, 0x04])
}

/// Checks for the 4-byte magic number, directory info, and that the
/// file size is as expected given the number of entries. `len` should be the
/// length of the entire WAD's file length, regardless of the length of `bytes`.
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
pub fn is_png(bytes: &[u8]) -> bool {
	bytes.len() > 8
		&& matches!(
			&bytes[0..8],
			&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
		)
}

/// Checks the header and total size. Ensure a slice over the entire lump is given.
pub fn is_doom_gfx(bytes: &[u8]) -> bool {
	const HEADER_SIZE: usize = 8;

	if bytes.len() < HEADER_SIZE {
		return false;
	}

	let width = LittleEndian::read_i16(&bytes[0..2]);
	let height = LittleEndian::read_i16(&bytes[2..4]);
	let left = LittleEndian::read_i16(&bytes[4..6]);
	let top = LittleEndian::read_i16(&bytes[6..8]);

	// Sanity check on dimensions
	if !(0..=4096).contains(&width) {
		return false;
	}

	if !(0..=4096).contains(&height) {
		return false;
	}

	if top < -2000 || top > 2000 {
		return false;
	}

	if left < -2000 || left > 2000 {
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
