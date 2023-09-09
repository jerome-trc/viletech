//! Functions for manipulating and inspecting filesystem paths.

use std::{
	env, fs,
	path::{Path, PathBuf},
};

use once_cell::sync::Lazy;

use crate::lazy_regex;

static ROOT_PATH: Lazy<&'static Path> = Lazy::new(|| Path::new("/"));

/// Extension trait for anything fulfilling `impl AsRef<std::path::Path>`.
pub trait PathExt: AsRef<Path> {
	#[must_use]
	fn dir_count(&self) -> usize;
	#[must_use]
	fn dir_empty(&self) -> bool {
		self.dir_count() < 1
	}
	/// Note that `test` is compared ASCII case-insensitively.
	#[must_use]
	fn extension_is(&self, test: &str) -> bool;
	/// Check if this path has no components at all.
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
	/// Check if the file name starts with a `.`.
	#[must_use]
	fn is_hidden(&self) -> bool;

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
}

impl<T: AsRef<Path>> PathExt for T {
	fn dir_count(&self) -> usize {
		match fs::read_dir(self.as_ref()) {
			Ok(read_dir) => read_dir.count(),
			Err(_) => 0,
		}
	}

	fn extension_is(&self, test: &str) -> bool {
		self.as_ref()
			.extension()
			.is_some_and(|ext| ext.eq_ignore_ascii_case(test))
	}

	fn is_empty(&self) -> bool {
		self.comp_len() == 0
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

	#[must_use]
	fn is_hidden(&self) -> bool {
		self.as_ref()
			.file_name()
			.map(|fname| fname.to_str().map(|s| s.starts_with('.')).unwrap_or(false))
			.unwrap_or(true)
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
}

/// Gets [`std::env::current_exe`] and pops the file component off.
#[must_use]
pub fn exe_dir() -> PathBuf {
	let mut ret = env::current_exe().expect("failed to get executable's directory");
	ret.pop();
	ret
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
