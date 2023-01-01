use std::{env, path::PathBuf};

use criterion::{criterion_group, criterion_main, Criterion};

use viletech::vfs::VirtualFs;

/// Leave this here even if it's empty, so there's a quick scaffold ready
/// for one-off benchmarking experiments.
fn misc(crit: &mut Criterion) {
	#[allow(unused)]
	let mut grp = crit.benchmark_group("Miscellaneous");
	grp.finish();
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

criterion_group!(benches, misc, vfs);
criterion_main!(benches);
