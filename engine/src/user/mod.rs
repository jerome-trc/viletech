//! User information (preferences, storage) structures and functions.

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

use std::{collections::HashMap, fs, io};

use bitflags::bitflags;

use crate::{
	gfx::Rgb32,
	utils::{
		env::create_default_user_dir,
		path::{get_user_dir, PathEx},
	},
};

bitflags! {
	pub struct PrefFlags: u8 {
		const NONE = 0;
		/// If unset, this pref only applies client-side.
		const SIM = 1 << 0;
	}
}

/// The second value holds the default.
pub enum PrefKind {
	Bool(bool, bool),
	Int(i32, i32),
	Float(f32, f32),
	Color(Rgb32, Rgb32),
	String(String, String),
}

pub enum UserGender {
	Female = 0,
	Male = 1,
	Neutral = 2,
	Object = 3,
}

pub struct UserPref {
	kind: PrefKind,
	flags: PrefFlags,
}

pub struct UserProfile {
	name: String,
	prefs: HashMap<String, UserPref>,
}

pub fn build_user_dirs() -> io::Result<()> {
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
		match fs::create_dir_all(&user_path) {
			Ok(()) => {}
			Err(err) => {
				return Err(io::Error::new(
					err.kind(),
					format!("Failed to create a part of the user info path: {}", err),
				));
			}
		};
	}

	let profiles_path = user_path.join("profiles");

	// End execution with an error if this directory has anything else in it,
	// so as not to clobber any other software's config files

	if !profiles_path.exists() {
		if !user_path.dir_empty() {
			return Err(io::Error::new(
				io::ErrorKind::Other,
				format!(
					"User info folder has unexpected contents; \
					is another program named \"Impure\" using it?
					({})",
					user_path.display()
				),
			));
		} else {
			match fs::create_dir(&profiles_path) {
				Ok(()) => {}
				Err(err) => {
					return Err(io::Error::new(
						io::ErrorKind::Other,
						format!(
							"Failed to create directory: {} \
							Error: {}",
							profiles_path.display(),
							err
						),
					))
				}
			};
		}
	}

	if profiles_path.dir_empty() {
		create_default_user_dir()?;
	}

	Ok(())
}
