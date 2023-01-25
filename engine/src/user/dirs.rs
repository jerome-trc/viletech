//! Functions for inspecting and managing user directories.

use std::path::{Path, PathBuf};

use crate::{user::GLOBALCFG_FILENAME, utils::path::exe_dir};

/// Check both given directories for existence. If both exist, the one which
/// appears to be more complete will take precedence; if both seem like completely
/// valid user directories, portable takes precedence.
///
/// This will return `None` if no user information could be found. This almost
/// certainly means the engine is a fresh installation, and the user will need
/// to be made to decide if they want user info to be stored in their application's
/// install directory ("portable") or their operating system's "home" directory.
#[must_use]
pub fn select_user_dir(portable: &Path, home: &Option<PathBuf>) -> Option<PathBuf> {
	if home.is_none() {
		if portable.exists() {
			return Some(portable.to_path_buf());
		} else {
			return None;
		}
	}

	let home = home.as_ref().unwrap();

	if !home.exists() {
		if portable.exists() {
			return Some(portable.to_path_buf());
		} else {
			return None;
		}
	} else if !portable.exists() {
		return Some(home.to_path_buf());
	}

	const EXPECTED_DIRS: &[&str] = &["profiles", "prefs"];

	let mut portable_completeness = 0;
	let mut home_completeness = 0;

	for &expected in EXPECTED_DIRS {
		let p_ex = portable.join(expected);
		let h_ex = home.join(expected);

		if p_ex.exists() && p_ex.is_dir() {
			portable_completeness += 1;
		}

		if h_ex.exists() && h_ex.is_dir() {
			home_completeness += 1;
		}
	}

	let p_ex = portable.join(GLOBALCFG_FILENAME);
	let h_ex = home.join(GLOBALCFG_FILENAME);

	if p_ex.exists() && !p_ex.is_dir() {
		portable_completeness += 1;
	}

	if h_ex.exists() && !h_ex.is_dir() {
		home_completeness += 1;
	}

	if home_completeness > portable_completeness {
		Some(home.to_path_buf())
	} else {
		Some(portable.to_path_buf())
	}
}

/// Returns `<executable directory>/user`.
/// Whether it exists on the file system or even is a directory is not considered.
#[must_use]
pub fn user_dir_portable() -> PathBuf {
	[exe_dir(), PathBuf::from("user")].iter().collect()
}

/// Returns `None` if the user's "home" directory can not be resolved.
/// Whether it exists on the file system or even is a directory is not considered.
/// See [`home::home_dir`] for details. Panics if not on Linux or Windows.
#[must_use]
pub fn user_dir_home() -> Option<PathBuf> {
	let mut home = match home::home_dir() {
		Some(h) => h,
		None => return None,
	};

	match std::env::consts::OS {
		"linux" => {
			home.push(".config");
			home.push("viletech");
		}
		"windows" => {
			home.push("viletech");
		}
		_ => {
			unimplemented!("This platform is currently unsupported.");
		}
	};

	Some(home)
}
