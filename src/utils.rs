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

use byteorder::{ByteOrder, LittleEndian};
use lazy_static::lazy_static;
use log::warn;
use regex::Regex;
use std::{
	env,
	fs::File,
	io::{self, Read},
	path::{Path, PathBuf},
};

pub fn exe_dir() -> PathBuf {
	let mut ret = env::current_exe().expect("Failed to get executable's directory.");
	ret.pop();
	ret
}

pub trait PathEx {
	fn extension_is(&self, path: &Path) -> bool;
}

impl PathEx for Path {
	fn extension_is(&self, path: &Path) -> bool {
		let ext = match self.extension() {
			Some(e) => e,
			None => {
				return false;
			}
		};

		return ext.eq_ignore_ascii_case(path.as_os_str());
	}
}

pub fn has_wad_extension(path: impl AsRef<Path>) -> bool {
	let p = path.as_ref();
	let ext = p.extension().unwrap_or_default();
	let extstr = ext.to_str().unwrap_or_default();

	lazy_static! {
		static ref RGX_WADEXT: Regex = Regex::new(r"[PpIi]*[Ww][Aa][Dd]")
			.expect("Failed to evaluate `has_wad_extension::RGX_WADEXT`.");
	};

	RGX_WADEXT.is_match(extstr)
}

pub fn has_zip_extension(path: impl AsRef<Path>) -> bool {
	let p = path.as_ref();
	let ext = p.extension().unwrap_or_default();
	let extstr = ext.to_str().unwrap_or_default();

	lazy_static! {
		static ref RGX_ZIPEXT: Regex = Regex::new(r"[Zz][Ii][Pp]")
			.expect("Failed to evaluate `has_zip_extension::RGX_ZIPEXT`.");
	};

	RGX_ZIPEXT.is_match(extstr)
}

pub fn has_gzdoom_extension(path: impl AsRef<Path>) -> bool {
	let p = path.as_ref();
	let ext = p.extension().unwrap_or_default();
	let extstr = ext.to_str().unwrap_or_default();

	lazy_static! {
		static ref RGX_GZDEXT: Regex = Regex::new(r"[Ii]*[Pp][Kk][37]")
			.expect("Failed to evaluate `has_zip_extension::RGX_GZDEXT`.");
	};

	RGX_GZDEXT.is_match(extstr)
}

pub fn has_eternity_extension(path: impl AsRef<Path>) -> bool {
	let p = path.as_ref();
	let ext = p.extension().unwrap_or_default();
	let extstr = ext.to_str().unwrap_or_default();

	lazy_static! {
		static ref RGX_ETERNEXT: Regex = Regex::new(r"[Pp][Kk][Ee3]")
			.expect("Failed to evaluate `has_zip_extension::RGX_ETERNEXT`.");
	};

	RGX_ETERNEXT.is_match(extstr)
}

pub fn str_iter_from_path(path: &Path) -> impl Iterator<Item = &str> {
	path.iter().filter_map(|c| match c.to_str() {
		Some(c) => Some(c),
		None => {
			warn!(
				"`str_iter_from_path()` 
				failed to convert a path component to UTF-8."
			);
			None
		}
	})
}

pub fn is_valid_wad(path: impl AsRef<Path>) -> Result<bool, io::Error> {
	let p = path.as_ref();

	if !p.exists() {
		return Err(io::ErrorKind::NotFound.into());
	}

	let mut buffer: [u8; 12] = [0; 12];
	let mut file = File::open(path)?;
	let metadata = file.metadata()?;

	let bytes_read = file.read(&mut buffer)?;

	if bytes_read < buffer.len() {
		return Ok(false);
	}

	match &buffer[0..4] {
		b"IWAD" | b"PWAD" => {}
		_ => {
			return Ok(false);
		}
	};

	let num_entries = LittleEndian::read_i32(&buffer[4..8]);
	let dir_offs = LittleEndian::read_i32(&buffer[8..12]);

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

	Ok(metadata.len() >= expected_bin_len as u64)
}
