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

use std::{path::{Path, PathBuf}, env};

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
			None => { return false; }
		};

		return ext.eq_ignore_ascii_case(path.as_os_str());
	}
}
