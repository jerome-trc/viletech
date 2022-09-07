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

use std::path::PathBuf;

use super::VirtualFs;

fn sample_paths() -> [(PathBuf, &'static str); 2] {
	let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("sample");
	let ret = [
		(base.join("freedoom1.wad"), "freedoom1"),
		(base.join("freedoom2.wad"), "freedoom2"),
	];

	for tuple in &ret {
		if !tuple.0.exists() {
			panic!(
				"VFS unit testing depends on \
				sample/freedoom1.wad and sample/freedoom2.wad."
			);
		}
	}

	ret
}

#[test]
fn mount() {
	let mut vfs = VirtualFs::default();
	let results = vfs.mount(&sample_paths());
	assert!(results.len() == 2);
	assert!(results[0].is_ok() && results[1].is_ok());
	assert!(vfs.mount_count() == 2);
}

#[test]
fn lookup() {
	let mut vfs = VirtualFs::default();
	vfs.mount(&sample_paths());
	assert!(vfs.lookup("freedoom2/FCGRATE2").is_some());
	assert!(vfs.lookup("/freedoom2/FCGRATE2").is_some());
}

#[test]
fn glob() {
	let mut vfs = VirtualFs::default();
	vfs.mount(&sample_paths());
	let glob = globset::Glob::new("/freedoom2/FCGRATE*").unwrap();
	let opt = vfs.glob(glob);
	assert!(opt.is_some());
	assert!(opt.unwrap().count() == 2);

	let glob = globset::Glob::new("/freedoom1/E*M*").unwrap();
	let opt = vfs.glob(glob);
	let count = opt.unwrap().count();
	assert!(count == 37, "Expected 37 maps, found: {}", count);

	let glob = globset::Glob::new("/freedoom2/MAP*").unwrap();
	let opt = vfs.glob(glob);
	let count = opt.unwrap().count();
	assert!(count == 32, "Expected 32 maps, found: {}", count);
}
