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

use std::{env, path::PathBuf};

use criterion::{criterion_group, criterion_main, Criterion};
use impure::vfs::VirtualFs;

fn vfs(crit: &mut Criterion) {
	fn mount(crit: &mut Criterion, mount_paths: &[(PathBuf, &str)]) {
		let mut vfs = VirtualFs::default();

		let mut grp_mount = crit.benchmark_group("Mount");
		grp_mount.sample_size(20);

		grp_mount.bench_function("Gamedata", |bencher| {
			bencher.iter(|| {
				vfs.mount(&mount_paths);
			});
		});

		grp_mount.finish();
	}

	fn lookup(crit: &mut Criterion, mount_paths: &[(PathBuf, &str)]) {
		let mut vfs = VirtualFs::default();
		vfs.mount(&mount_paths);

		let mut grp_lookup = crit.benchmark_group("Lookup");
		grp_lookup.sample_size(10_000);

		grp_lookup.bench_with_input("Worst-Case", &vfs, |bencher, vfs| {
			bencher.iter(|| {
				vfs.lookup("freedoom2/FCGRATE2").unwrap();
			});
		});

		grp_lookup.finish();
	}

	let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("sample");
	let mount_paths = [
		(base.join("freedoom1.wad"), "freedoom1"),
		(base.join("freedoom2.wad"), "freedoom2"),
	];

	for tuple in &mount_paths {
		if !tuple.0.exists() {
			eprintln!(
				"VFS benchmarking depends on \
				sample/freedoom1.wad and sample/freedoom2.wad."
			);
			return;
		}
	}

	mount(crit, &mount_paths);
	lookup(crit, &mount_paths);
}

criterion_group!(benches, vfs);
criterion_main!(benches);
