//! Functions for inspecting the host platform/system/environment/etc.

use std::{env, error::Error, io};

use bevy::prelude::error;

pub fn os_info() -> Result<String, Box<dyn Error>> {
	type Command = std::process::Command;

	match env::consts::OS {
		"linux" => {
			let uname = Command::new("uname").args(["-s", "-r", "-v"]).output();

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
				.args(["/C:\"OS\""])
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
		_ => Err(Box::<io::Error>::new(io::ErrorKind::Unsupported.into())),
	}
}
