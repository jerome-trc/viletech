//! Functions for manipulating and inspecting filesystem paths.

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

use lazy_static::lazy_static;
use log::warn;
use regex::Regex;
use std::{
	env,
	fs::{self, File},
	io::{self, Read},
	path::{Path, PathBuf},
};

lazy_static! {
	static ref EMPTY_PATH: &'static Path = Path::new("");
	static ref ROOT_PATH: &'static Path = Path::new("/");
}

pub trait PathEx {
	fn dir_count(&self) -> usize;
	fn dir_empty(&self) -> bool {
		self.dir_count() < 1
	}
	/// Returns the number of components in the path.
	fn extension_is(&self, test: &str) -> bool;
	fn is_empty(&self) -> bool;
	fn is_root(&self) -> bool;
	fn size(&self) -> usize;

	fn has_zip_extension(&self) -> bool;
	/// Check if an archive is a .wad, .pwad, or .iwad.
	fn has_wad_extension(&self) -> bool;
	/// Check if an archive is a .pk3, .pk7, .ipk3, or .ipk7.
	fn has_gzdoom_extension(&self) -> bool;
	/// Check if an archive is a .pk3 or .pke.
	fn has_eternity_extension(&self) -> bool;

	/// See [`super::io::is_binary`].
	fn is_binary(&self) -> io::Result<bool>;
	/// See [`super::io::is_zip`].
	fn is_zip(&self) -> io::Result<bool>;
	/// See [`super::io::is_valid_wad`].
	fn is_valid_wad(&self) -> io::Result<bool>;
	/// Check if this file is a zip or WAD.
	fn is_supported_archive(&self) -> io::Result<bool>;
}

impl<T: AsRef<Path>> PathEx for T {
	fn dir_count(&self) -> usize {
		match fs::read_dir(self.as_ref()) {
			Ok(read_dir) => read_dir.count(),
			Err(_) => 0,
		}
	}

	fn extension_is(&self, test: &str) -> bool {
		let ext = match self.as_ref().extension() {
			Some(e) => e,
			None => {
				return false;
			}
		};

		ext.eq_ignore_ascii_case(test)
	}

	fn is_empty(&self) -> bool {
		self.as_ref() == *EMPTY_PATH
	}

	fn is_root(&self) -> bool {
		self.as_ref() == *ROOT_PATH
	}

	fn size(&self) -> usize {
		self.as_ref().components().count()
	}

	fn has_zip_extension(&self) -> bool {
		let p = self.as_ref();
		let ext = p.extension().unwrap_or_default();
		let extstr = ext.to_str().unwrap_or_default();

		lazy_static! {
			static ref RGX_ZIPEXT: Regex = Regex::new(r"^(?i)zip$")
				.expect("Failed to evaluate `has_zip_extension::RGX_ZIPEXT`.");
		};

		RGX_ZIPEXT.is_match(extstr)
	}

	fn has_wad_extension(&self) -> bool {
		let p = self.as_ref();
		let ext = p.extension().unwrap_or_default();
		let extstr = ext.to_str().unwrap_or_default();

		lazy_static! {
			static ref RGX_WADEXT: Regex = Regex::new(r"^(?i)[pi]*wad$")
				.expect("Failed to evaluate `has_wad_extension::RGX_WADEXT`.");
		};

		RGX_WADEXT.is_match(extstr)
	}

	fn has_gzdoom_extension(&self) -> bool {
		let p = self.as_ref();
		let ext = p.extension().unwrap_or_default();
		let extstr = ext.to_str().unwrap_or_default();

		lazy_static! {
			static ref RGX_GZDEXT: Regex = Regex::new(r"^(?i)i*pk[37]$")
				.expect("Failed to evaluate `has_zip_extension::RGX_GZDEXT`.");
		};

		RGX_GZDEXT.is_match(extstr)
	}

	fn has_eternity_extension(&self) -> bool {
		let p = self.as_ref();
		let ext = p.extension().unwrap_or_default();
		let extstr = ext.to_str().unwrap_or_default();

		lazy_static! {
			static ref RGX_ETERNEXT: Regex = Regex::new(r"^(?i)pk[e3]$")
				.expect("Failed to evaluate `has_zip_extension::RGX_ETERNEXT`.");
		};

		RGX_ETERNEXT.is_match(extstr)
	}

	fn is_binary(&self) -> io::Result<bool> {
		let p = self.as_ref();

		if !p.exists() {
			return Err(io::ErrorKind::NotFound.into());
		}

		let mut buffer = [0u8; 512];
		let mut file = File::open(p)?;

		let bytes_read = file.read(&mut buffer)?;

		if bytes_read == 0 {
			return Ok(false);
		}

		Ok(super::io::is_binary(&buffer))
	}

	fn is_zip(&self) -> io::Result<bool> {
		if !self.as_ref().exists() {
			return Err(io::ErrorKind::NotFound.into());
		}

		let mut buffer: [u8; 4] = [0; 4];
		let mut file = File::open(self)?;

		let bytes_read = file.read(&mut buffer)?;

		if bytes_read < buffer.len() {
			return Ok(false);
		}

		Ok(super::io::is_zip(&buffer[..]))
	}

	fn is_valid_wad(&self) -> io::Result<bool> {
		if !self.as_ref().exists() {
			return Err(io::ErrorKind::NotFound.into());
		}

		let mut buffer: [u8; 12] = [0; 12];
		let mut file = File::open(self)?;
		let metadata = file.metadata()?;

		let bytes_read = file.read(&mut buffer)?;

		if bytes_read < buffer.len() {
			return Ok(false);
		}

		super::io::is_valid_wad(&buffer[..], metadata.len())
	}

	fn is_supported_archive(&self) -> io::Result<bool> {
		let p = self.as_ref();

		match p.is_zip() {
			Ok(b) => {
				if b {
					return Ok(b);
				}
			}
			Err(err) => {
				return Err(err);
			}
		}

		match p.is_valid_wad() {
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
}

pub fn exe_dir() -> PathBuf {
	let mut ret = env::current_exe().expect("Failed to get executable's directory.");
	ret.pop();
	ret
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

/// Expands `~` on Unix and performs environment variable substitution.
/// Deliberately designed to mimic `NicePath` in
/// <https://github.com/ZDoom/gzdoom/blob/master/src/common/utility/cmdlib.cpp>.
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

pub fn str_iter_from_path(path: &Path) -> impl Iterator<Item = &str> {
	path.iter().filter_map(|c| match c.to_str() {
		Some(c) => Some(c),
		None => {
			warn!(
				"`utils::path::str_iter_from_path` \
				failed to convert a path component to UTF-8."
			);
			None
		}
	})
}
