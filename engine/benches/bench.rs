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

use std::{env, path::PathBuf, sync::Arc};

use criterion::{criterion_group, criterion_main, Criterion};
use impure::{lua::ImpureLua, newtype, rng::ImpureRng, sim::PlaySim, vfs::VirtualFs};
use mlua::prelude::*;
use parking_lot::{Mutex, RwLock};

newtype!(struct PlaySimBenchWrapper(PlaySim));

impl LuaUserData for PlaySimBenchWrapper {}

fn lua(crit: &mut Criterion) {
	let lua = Lua::new_ex(true, true).unwrap();
	lua.set_app_data(PlaySim::default());
	let amps = Arc::new(Mutex::new(PlaySim::default()));
	let arwps = Arc::new(RwLock::new(PlaySim::default()));
	let regkey = lua
		.create_registry_value(PlaySimBenchWrapper(PlaySim::default()))
		.unwrap();

	let mut grp_nativebind = crit.benchmark_group("Lua");

	grp_nativebind.bench_function("Native Bind, App Data", |bencher| {
		bencher.iter(|| {
			let mut r = lua.app_data_mut::<PlaySim>().unwrap();
			let _ = r.rng.get_anon().range_i32(0, 1);
		});
	});

	grp_nativebind.bench_function("Native Bind, Registry Userdata", |bencher| {
		bencher.iter(|| {
			let r = lua.registry_value::<LuaAnyUserData>(&regkey).unwrap();
			let mut ps = r.borrow_mut::<PlaySimBenchWrapper>().unwrap();
			let _ = ps.rng.get_anon().range_i32(0, 1);
		});
	});

	grp_nativebind.bench_function("Native Bind, Arc/Mutex", |bencher| {
		bencher.iter(|| {
			let _ = amps.lock().rng.get_anon().range_i32(0, 1);
		});
	});

	grp_nativebind.bench_function("Native Bind, Arc/RwLock", |bencher| {
		bencher.iter(|| {
			let _ = arwps.write().rng.get_anon().range_i32(0, 1);
		});
	});

	grp_nativebind.finish();
}

fn vfs(crit: &mut Criterion) {
	fn mount(crit: &mut Criterion, mount_paths: &[(PathBuf, &str)]) {
		let mut vfs = VirtualFs::default();

		let mut grp_mount = crit.benchmark_group("VFS: Mount");
		grp_mount.sample_size(20);

		grp_mount.bench_function("Gamedata", |bencher| {
			bencher.iter(|| {
				let _ = vfs.mount(&mount_paths);
			});
		});

		grp_mount.finish();
	}

	fn lookup(crit: &mut Criterion, mount_paths: &[(PathBuf, &str)]) {
		let mut vfs = VirtualFs::default();
		let _ = vfs.mount(&mount_paths);

		let mut grp_lookup = crit.benchmark_group("VFS: Lookup");
		grp_lookup.sample_size(10_000);

		grp_lookup.bench_with_input("Worst-Case", &vfs, |bencher, vfs| {
			bencher.iter(|| {
				vfs.lookup("freedoom2/FCGRATE2").unwrap();
			});
		});

		grp_lookup.finish();
	}

	let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.join("..")
		.join("sample");
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

criterion_group!(benches, lua, vfs);
criterion_main!(benches);
