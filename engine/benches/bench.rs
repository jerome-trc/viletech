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
use impure::vfs::{ImpureVfs, VirtualFs};

pub fn vfs(crit: &mut Criterion) {
	let mut group = crit.benchmark_group("Mount");

	group.sample_size(10);

	group.bench_function("Gamedata", |b| {
		b.iter(|| {
			let pwd_gd: PathBuf = [
				env::current_dir().expect("Failed to retrieve PWD."),
				PathBuf::from("data"),
			]
			.iter()
			.collect();

			let mut vfs = VirtualFs::default();
			vfs.mount_gamedata(vec![&pwd_gd]);
		});
	});

	group.finish();
}

criterion_group!(benches, vfs);
criterion_main!(benches);
