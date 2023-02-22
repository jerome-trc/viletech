use std::{env, path::PathBuf};

use criterion::{criterion_group, criterion_main, Criterion};

use viletech::data::{Catalog, LoadRequest};

/// Leave this here even if it's empty, so there's a quick scaffold ready
/// for one-off benchmarking experiments.
fn misc(crit: &mut Criterion) {
	#[allow(unused)]
	let mut grp = crit.benchmark_group("Miscellaneous");
	grp.finish();
}

fn vfs(crit: &mut Criterion) {
	fn request() -> LoadRequest<PathBuf, &'static str> {
		let sample = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("..")
			.join("sample");
		let mount_paths = vec![
			(sample.join("freedoom1.wad"), "freedoom1"),
			(sample.join("freedoom2.wad"), "freedoom2"),
		];

		for (real_path, _) in &mount_paths {
			if !real_path.exists() {
				panic!(
					"VFS benchmarking depends on the following files of sample data:\r\n\t\
					- `$CARGO_MANIFEST_DIR/sample/freedoom1.wad`\r\n\t\
					- `$CARGO_MANIFEST_DIR/sample/freedoom2.wad`\r\n\t\
					They can be acquired from https://freedoom.github.io/."
				);
			}
		}

		LoadRequest {
			paths: mount_paths,
			tracker: None,
		}
	}

	fn mount_unmount(crit: &mut Criterion) {
		let mut catalog = Catalog::default();

		let mut grp = crit.benchmark_group("VFS: Mount, Unmount");
		grp.sample_size(20);

		grp.bench_function("FreeDoom, FreeDoom 2", |bencher| {
			bencher.iter(|| {
				let _ = catalog.load(request());
				let _ = catalog.truncate(0);
			});
		});

		grp.finish();
	}

	fn lookup(crit: &mut Criterion) {
		let mut catalog = Catalog::default();
		let _ = catalog.load(request());

		let mut grp = crit.benchmark_group("VFS: Lookup");
		grp.sample_size(10_000);

		grp.bench_function("First Loaded", |bencher| {
			bencher.iter(|| {
				let _ = catalog.get_file("/freedoom1/E1M1").unwrap();
			});
		});

		grp.bench_function("Last Loaded", |bencher| {
			bencher.iter(|| {
				let _ = catalog.get_file("freedoom2/FCGRATE2").unwrap();
			});
		});

		grp.finish();
	}

	mount_unmount(crit);
	lookup(crit);
}

criterion_group!(benches, misc, vfs);
criterion_main!(benches);
