//! Functions for manipulating and inspecting filesystem paths.

use std::{
	env,
	fs::{self, File},
	io::{self, Read, Seek, SeekFrom},
	path::{Path, PathBuf},
};

use log::warn;
use once_cell::sync::Lazy;

use crate::lazy_regex;

static EMPTY_PATH: Lazy<&'static Path> = Lazy::new(|| Path::new(""));
static ROOT_PATH: Lazy<&'static Path> = Lazy::new(|| Path::new("/"));

pub trait PathExt {
	#[must_use]
	fn dir_count(&self) -> usize;
	#[must_use]
	fn dir_empty(&self) -> bool {
		self.dir_count() < 1
	}
	#[must_use]
	fn extension_is(&self, test: &str) -> bool;
	#[must_use]
	fn is_empty(&self) -> bool;
	#[must_use]
	fn is_root(&self) -> bool;
	/// Results are only valid for absolute paths; will always return `false` if
	/// `self` or `other` is relative. A path can not be a child of itself; giving
	/// two equal paths will also return `false`.
	#[must_use]
	fn is_child_of(&self, other: impl AsRef<Path>) -> bool;
	/// Returns the number of components in the path.
	#[must_use]
	fn comp_len(&self) -> usize;

	#[must_use]
	fn has_zip_extension(&self) -> bool;
	/// Check if an archive is a .wad, .pwad, or .iwad.
	#[must_use]
	fn has_wad_extension(&self) -> bool;
	/// Check if an archive is a .pk3, .pk7, .ipk3, or .ipk7.
	#[must_use]
	fn has_gzdoom_extension(&self) -> bool;
	/// Check if an archive is a .pk3 or .pke.
	#[must_use]
	fn has_eternity_extension(&self) -> bool;

	/// See [`super::io::is_binary`].
	fn is_binary(&self) -> io::Result<bool>;
	/// See [`super::io::is_zip`].
	fn is_zip(&self) -> io::Result<bool>;
	/// See [`super::io::is_lzma`].
	fn is_lzma(&self) -> io::Result<bool>;
	/// See [`super::io::is_xz`].
	fn is_xz(&self) -> io::Result<bool>;
	/// See [`super::io::is_valid_wad`].
	fn is_valid_wad(&self) -> io::Result<bool>;
	/// Check if this file is a zip or WAD.
	fn is_supported_archive(&self) -> io::Result<bool>;
}

impl<T: AsRef<Path>> PathExt for T {
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

	fn is_child_of(&self, other: impl AsRef<Path>) -> bool {
		let this = self.as_ref();
		let other = other.as_ref();

		if this.is_relative() | other.is_relative() {
			return false;
		}

		if this == other {
			return false;
		}

		let mut self_comps = this.components();

		for comp in other.components() {
			if let Some(self_comp) = self_comps.next() {
				if self_comp == comp {
					continue;
				} else {
					return false;
				}
			} else {
				return false;
			}
		}

		true
	}

	fn comp_len(&self) -> usize {
		self.as_ref().components().count()
	}

	fn has_zip_extension(&self) -> bool {
		self.as_ref()
			.extension()
			.unwrap_or_default()
			.to_str()
			.unwrap_or_default()
			.eq_ignore_ascii_case("zip")
	}

	fn has_wad_extension(&self) -> bool {
		lazy_regex!(r"^(?i)[pi]?wad$").is_match(
			self.as_ref()
				.extension()
				.unwrap_or_default()
				.to_str()
				.unwrap_or_default(),
		)
	}

	fn has_gzdoom_extension(&self) -> bool {
		lazy_regex!(r"^(?i)i?pk[37]$").is_match(
			self.as_ref()
				.extension()
				.unwrap_or_default()
				.to_str()
				.unwrap_or_default(),
		)
	}

	fn has_eternity_extension(&self) -> bool {
		let s = self
			.as_ref()
			.extension()
			.unwrap_or_default()
			.to_str()
			.unwrap_or_default();

		s.eq_ignore_ascii_case("pk3") || s.eq_ignore_ascii_case("pke")
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

		let mut buffer = [0u8; 4];
		let mut file = File::open(self)?;

		let bytes_read = file.read(&mut buffer)?;

		if bytes_read < buffer.len() {
			return Ok(false);
		}

		Ok(super::io::is_zip(&buffer[..]))
	}

	fn is_lzma(&self) -> io::Result<bool> {
		if !self.as_ref().exists() {
			return Err(io::ErrorKind::NotFound.into());
		}

		let mut buffer = [0u8; 13];
		let mut file = File::open(self)?;

		let bytes_read = file.read(&mut buffer)?;

		if bytes_read < buffer.len() {
			return Ok(false);
		}

		Ok(super::io::is_lzma(&buffer[..]))
	}

	fn is_xz(&self) -> io::Result<bool> {
		if !self.as_ref().exists() {
			return Err(io::ErrorKind::NotFound.into());
		}

		let mut header = [0u8; 6];
		let mut footer = [0u8; 2];
		let mut file = File::open(self)?;
		let len = file.metadata()?.len();

		let header_read = file.read(&mut header)?;

		if header_read < header.len() {
			return Ok(false);
		}

		file.seek(SeekFrom::End(-2))?;

		let footer_read = file.read(&mut footer)?;

		if footer_read < footer.len() {
			return Ok(false);
		}

		Ok(super::io::is_xz(&header[..], &footer[..], len))
	}

	fn is_valid_wad(&self) -> io::Result<bool> {
		if !self.as_ref().exists() {
			return Err(io::ErrorKind::NotFound.into());
		}

		let mut buffer = [0u8; 12];
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

/// Gets [`std::env::current_exe`] and pops the file component off.
#[must_use]
pub fn exe_dir() -> PathBuf {
	let mut ret = env::current_exe().expect("Failed to get executable's directory.");
	ret.pop();
	ret
}

/// Returns `None` if this platform is unsupported or the home directory path is
/// malformed. See [`home::home_dir`].
#[must_use]
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
			ret.push("viletech");
		}
		"windows" => {
			ret.push("viletech");
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
#[must_use]
pub fn nice_path(path: impl AsRef<Path>) -> PathBuf {
	let p = path.as_ref();

	if p.is_empty() {
		return PathBuf::from(".");
	}

	#[cfg(not(target_os = "windows"))]
	if p.is_root() {
		return PathBuf::from("/");
	}

	let mut string = p.to_string_lossy().to_string();

	#[cfg(not(target_os = "windows"))]
	{
		let home = home::home_dir().unwrap_or_default();
		let home = home.to_string_lossy();
		string = string.replace('~', &home);
	}

	let matches = lazy_regex!(r"\$[[:word:]]+").find_iter(&string);
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

	PathBuf::from(string)
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
