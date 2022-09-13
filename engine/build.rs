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

use std::{error::Error, process::Command};

/// Injects the current Git hash and date and time of compilation
/// into the environment before building.
fn main() -> Result<(), Box<dyn Error>> {
	let hash = match Command::new("git").args(&["rev-parse", "HEAD"]).output() {
		Ok(h) => h,
		Err(err) => {
			eprintln!("Failed to execute `git rev-parse HEAD`: {}", err);
			return Err(Box::new(err));
		}
	};

	let hash_str = match String::from_utf8(hash.stdout) {
		Ok(s) => s,
		Err(err) => {
			eprintln!(
				"Failed to convert output of `git rev-parse HEAD` to UTF-8: {}",
				err
			);
			return Err(Box::new(err));
		}
	};

	println!("cargo:rustc-env=GIT_HASH={}", hash_str);
	println!(
		"cargo:rustc-env=COMPILE_DATETIME={} UTC",
		chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
	);

	Ok(())
}
