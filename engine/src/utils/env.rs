//! Functions for inspecting the host platform/system/environment/etc.

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

use std::{env, io, fs, error::Error};

use log::error;

use super::path::*;

pub fn os_info() -> Result<String, Box<dyn Error>> {
	type Command = std::process::Command;

	match env::consts::OS {
		"linux" => {
			let uname = Command::new("uname").args(&["-s", "-r", "-v"]).output();

			let output = match uname {
				Ok(o) => o,
				Err(err) => {
					error!("Failed to execute `uname -s -r -v`: {}", err);
					return Err(Box::new(err));
				}
			};

			let osinfo = match String::from_utf8(output.stdout) {
				Ok(s) => s.replace('\n', ""),
				Err(err) => {
					error!(
						"Failed to convert `uname -s -r -v` output to UTF-8: {}",
						err
					);
					return Err(Box::new(err));
				}
			};

			Ok(osinfo)
		}
		"windows" => {
			let systeminfo = Command::new("systeminfo | findstr")
				.args(&["/C:\"OS\""])
				.output();

			let output = match systeminfo {
				Ok(o) => o,
				Err(err) => {
					error!(
						"Failed to execute `systeminfo | findstr /C:\"OS\"`: {}",
						err
					);
					return Err(Box::new(err));
				}
			};

			let osinfo = match String::from_utf8(output.stdout) {
				Ok(s) => s,
				Err(err) => {
					error!(
						"Failed to convert `systeminfo | findstr /C:\"OS\"` \
						 output to UTF-8: {}",
						err
					);
					return Err(Box::new(err));
				}
			};

			Ok(osinfo)
		}
		_ => {
			Err(Box::<io::Error>::new(io::ErrorKind::Unsupported.into()))
		}
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
