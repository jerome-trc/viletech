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
	fs::{self, File},
	io::{self, Read},
	path::{Path, PathBuf},
};

pub fn exe_dir() -> PathBuf {
	let mut ret = env::current_exe().expect("Failed to get executable's directory.");
	ret.pop();
	ret
}

pub trait PathEx {
	fn extension_is(&self, test: &str) -> bool;
	fn is_empty(&self) -> bool;
	fn is_root(&self) -> bool;
}

lazy_static! {
	static ref EMPTY_PATH: &'static Path = Path::new("");
	static ref ROOT_PATH: &'static Path = Path::new("/");
}

impl PathEx for Path {
	fn extension_is(&self, test: &str) -> bool {
		let ext = match self.extension() {
			Some(e) => e,
			None => {
				return false;
			}
		};

		ext.eq_ignore_ascii_case(test)
	}

	fn is_empty(&self) -> bool {
		self == *EMPTY_PATH
	}

	fn is_root(&self) -> bool {
		self == *ROOT_PATH
	}
}

impl PathEx for PathBuf {
	fn extension_is(&self, test: &str) -> bool {
		let ext = match self.extension() {
			Some(e) => e,
			None => {
				return false;
			}
		};

		ext.eq_ignore_ascii_case(test)
	}

	fn is_empty(&self) -> bool {
		self == *EMPTY_PATH
	}

	fn is_root(&self) -> bool {
		self == *ROOT_PATH
	}
}

/// Check if an archive is a .wad, .pwad, or .iwad.
pub fn has_wad_extension(path: impl AsRef<Path>) -> bool {
	let p = path.as_ref();
	let ext = p.extension().unwrap_or_default();
	let extstr = ext.to_str().unwrap_or_default();

	lazy_static! {
		static ref RGX_WADEXT: Regex = Regex::new(r"^(?i)[pi]*wad$")
			.expect("Failed to evaluate `has_wad_extension::RGX_WADEXT`.");
	};

	RGX_WADEXT.is_match(extstr)
}

pub fn has_zip_extension(path: impl AsRef<Path>) -> bool {
	let p = path.as_ref();
	let ext = p.extension().unwrap_or_default();
	let extstr = ext.to_str().unwrap_or_default();

	lazy_static! {
		static ref RGX_ZIPEXT: Regex =
			Regex::new(r"^(?i)zip$").expect("Failed to evaluate `has_zip_extension::RGX_ZIPEXT`.");
	};

	RGX_ZIPEXT.is_match(extstr)
}

/// Check if an archive is a .pk3, .pk7, .ipk3, or .ipk7.
pub fn has_gzdoom_extension(path: impl AsRef<Path>) -> bool {
	let p = path.as_ref();
	let ext = p.extension().unwrap_or_default();
	let extstr = ext.to_str().unwrap_or_default();

	lazy_static! {
		static ref RGX_GZDEXT: Regex = Regex::new(r"^(?i)i*pk[37]$")
			.expect("Failed to evaluate `has_zip_extension::RGX_GZDEXT`.");
	};

	RGX_GZDEXT.is_match(extstr)
}

/// Check if an archive is a .pk3 or .pke.
pub fn has_eternity_extension(path: impl AsRef<Path>) -> bool {
	let p = path.as_ref();
	let ext = p.extension().unwrap_or_default();
	let extstr = ext.to_str().unwrap_or_default();

	lazy_static! {
		static ref RGX_ETERNEXT: Regex = Regex::new(r"^(?i)pk[e3]$")
			.expect("Failed to evaluate `has_zip_extension::RGX_ETERNEXT`.");
	};

	RGX_ETERNEXT.is_match(extstr)
}

pub fn str_iter_from_path(path: &Path) -> impl Iterator<Item = &str> {
	path.iter().filter_map(|c| match c.to_str() {
		Some(c) => Some(c),
		None => {
			warn!("`str_iter_from_path()` failed to convert a path component to UTF-8.");
			None
		}
	})
}

/// Checks for the 4-byte magic number, directory info, and that the
/// file size is as expected given the number of entries.
pub fn is_valid_wad(path: impl AsRef<Path>) -> io::Result<bool> {
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

/// Returns `None` if this platform is unsupported
/// or the home directory path is malformed.
pub fn get_user_dir() -> Option<PathBuf> {
	let mut ret = match home::home_dir() {
		Some(hdir) => hdir,
		None => {
			return None;
		}
	};

	match env::consts::OS {
		"linux" => {
			ret.push(".config");
			ret.push("impure");
		}
		"windows" => {
			ret.push("impure");
		}
		_ => {
			return None;
		}
	}

	Some(ret)
}

/// Checks for a 4-byte magic number at the very beginning of the file.
pub fn is_zip(path: impl AsRef<Path>) -> io::Result<bool> {
	let p = path.as_ref();

	if !p.exists() {
		return Err(io::ErrorKind::NotFound.into());
	}

	let mut buffer: [u8; 4] = [0; 4];
	let mut file = File::open(path)?;

	let bytes_read = file.read(&mut buffer)?;

	if bytes_read < buffer.len() {
		return Ok(false);
	}

	match &buffer[0..4] {
		&[0x50, 0x4b, 0x03, 0x04] => Ok(true),
		_ => Ok(false),
	}
}

/// Check if the given file is a zip or WAD.
pub fn is_supported_archive(path: impl AsRef<Path>) -> io::Result<bool> {
	match is_zip(path.as_ref()) {
		Ok(b) => {
			if b {
				return Ok(b);
			}
		}
		Err(err) => {
			return Err(err);
		}
	}

	match is_valid_wad(path.as_ref()) {
		Ok(b) => {
			if b {
				return Ok(b);
			}
		}
		Err(err) => {
			return Err(err);
		}
	}

	Ok(false)
}

/// Check if the given file is not ASCII-encoded, UTF-8 encoded, or UTF-16 encoded.
pub fn is_binary(path: impl AsRef<Path>) -> io::Result<bool> {
	let p = path.as_ref();

	if !p.exists() {
		return Err(io::ErrorKind::NotFound.into());
	}

	const BUF_SIZE: usize = 1024;

	let mut buffer: [u8; BUF_SIZE] = [0; BUF_SIZE];
	let mut file = File::open(path)?;

	let bytes_read = file.read(&mut buffer)?;

	if bytes_read == 0 {
		return Ok(false);
	}

	if std::str::from_utf8(&buffer).is_ok() || buffer.is_ascii() {
		return Ok(false);
	}

	// (Safety: unverified)
	unsafe {
		let buffer16: [u16; BUF_SIZE / 2] = std::mem::transmute(buffer);
		let iter = std::char::decode_utf16(buffer16);

		for cpoint in iter {
			if cpoint.is_err() {
				return Ok(true);
			}
		}
	}

	Ok(false)
}

/// Expands `~` on Unix and performs environment variable substitution.
/// Deliberately designed to mimic `NicePath` in
/// <https://github.com/ZDoom-Official/gzdoom/blob/master/src/common/utility/cmdlib.cpp>.
pub fn nice_path(path: impl AsRef<Path>) -> io::Result<PathBuf> {
	let p = path.as_ref();

	if p.is_empty() {
		return Ok(PathBuf::from("."));
	}

	#[cfg(not(target_os = "windows"))]
	if p.is_root() {
		return Ok(PathBuf::from("/"));
	}

	let mut string = p.to_string_lossy().to_string();

	#[cfg(not(target_os = "windows"))]
	{
		let home = home::home_dir().unwrap_or_default();
		let home = home.to_string_lossy();
		string = string.replace('~', &home);
	}

	lazy_static! {
		static ref RGX_ENVVAR: Regex =
			Regex::new(r"\$[[:word:]]+").expect("Failed to evaluate `nice_path::RGX_ENVVAR`.");
	};

	let matches = RGX_ENVVAR.find_iter(&string);
	let mut ret = string.clone();

	for m in matches {
		match env::var(m.as_str()) {
			Ok(v) => {
				ret.replace_range(m.range(), &v);
			}
			Err(_) => {
				ret.replace_range(m.range(), "");
			}
		}
	}

	Ok(PathBuf::from(string))
}

lazy_static! {
	static ref RGX_VERSION: Regex = Regex::new(
		r"[ \-_][VvRr]*[\._\-]*\d{1,}([\._\-]\d{1,})*([\._\-]\d{1,})*[A-Za-z]*[\._\-]*[A-Za-z0-9]*$"
	)
	.expect("Failed to evaluate `utils::RGX_VERSION`.");
}

/// Locates a version string at the end of a file stem, using a search pattern
/// based off the most common versioning conventions used in ZDoom modding.
/// If the returned option is `None`, the given string is unmodified.
pub fn version_from_filestem(string: &mut String) -> Option<String> {
	match RGX_VERSION.find(string) {
		Some(m) => {
			const TO_TRIM: [char; 3] = [' ', '_', '-'];
			let ret = m.as_str().trim_matches(&TO_TRIM[..]).to_string();
			string.replace_range(m.range(), "");
			Some(ret)
		}
		None => None,
	}
}

/// Returns the number of entries under a directory.
pub fn dir_count(path: impl AsRef<Path>) -> usize {
	match fs::read_dir(path) {
		Ok(read_dir) => read_dir.count(),
		Err(_) => 0,
	}
}

pub fn create_default_user_dir() -> io::Result<()> {
	let user_path = match get_user_dir() {
		Some(up) => up,
		None => {
			return Err(io::Error::new(
				io::ErrorKind::Other,
				"Failed to retrieve user info path. \
				Home directory path is malformed, \
				or this platform is currently unsupported.",
			));
		}
	};

	if !user_path.exists() {
		return Err(io::Error::new(
			io::ErrorKind::Other,
			"Attempted to create a default user directory, \
			but user info directory is missing.",
		));
	}

	let profiles_path = user_path.join("profiles");

	if !profiles_path.exists() {
		return Err(io::Error::new(
			io::ErrorKind::Other,
			"Attempted to create a default user directory, \
			but user profiles directory is missing.",
		));
	}

	let defuser_path = profiles_path.join("Player");

	match fs::create_dir(&defuser_path) {
		Ok(()) => {}
		Err(err) => {
			return Err(io::Error::new(
				err.kind(),
				format!(
					"Failed to create a default user directory: {}\
				Error: {}",
					defuser_path.display(),
					err
				),
			));
		}
	};

	let defuser_saves_path = defuser_path.join("saves");

	match fs::create_dir(&defuser_saves_path) {
		Ok(()) => {}
		Err(err) => {
			return Err(io::Error::new(
				err.kind(),
				format!(
					"Failed to create default user saves directory: {}\
				Error: {}",
					defuser_saves_path.display(),
					err
				),
			));
		}
	};

	let defuser_prefs_path = defuser_path.join("prefs");

	match fs::create_dir(&defuser_prefs_path) {
		Ok(()) => {}
		Err(err) => {
			return Err(io::Error::new(
				err.kind(),
				format!(
					"Failed to create default user preferences directory: {}\
				Error: {}",
					defuser_prefs_path.display(),
					err
				),
			));
		}
	};

	let defuser_storage_path = defuser_path.join("storage");

	match fs::create_dir(&defuser_storage_path) {
		Ok(()) => {}
		Err(err) => {
			return Err(io::Error::new(
				err.kind(),
				format!(
					"Failed to create default user storage directory: {}\
				Error: {}",
					defuser_storage_path.display(),
					err
				),
			));
		}
	};

	Ok(())
}
