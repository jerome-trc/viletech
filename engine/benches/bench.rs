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

// TODO:
// - VFS
// 		- Mount
//		- Unmount
//		- Lookup
// - Catalog
// 		- Load
//		- Unload
//		- Datum lookup
// - VZS
// 		- Parse
//		- Precompile
//		- Compile

fn data(crit: &mut Criterion) {
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
			load_order: mount_paths,
			tracker: None,
			dev_mode: false,
		}
	}

	fn load_unload(crit: &mut Criterion) {
		let mut catalog = Catalog::new([]);

		let mut grp = crit.benchmark_group("Data: Load and Unload");
		grp.sample_size(20);

		grp.bench_function("FreeDoom, FreeDoom 2", |bencher| {
			bencher.iter_batched(
				|| request(),
				|req_l| {
					let _ = catalog.load(req_l);
					catalog.clear();
				},
				criterion::BatchSize::SmallInput,
			);
		});

		grp.finish();
	}

	fn lookup(crit: &mut Criterion) {
		let mut catalog = Catalog::new([]);
		let _ = catalog.load(request());

		let mut grp = crit.benchmark_group("Data: VFS Lookup");
		grp.sample_size(10_000);

		grp.bench_function("First Loaded", |bencher| {
			bencher.iter(|| {
				let _ = catalog.vfs().get("/freedoom1/E1M1").unwrap();
			});
		});

		grp.bench_function("Last Loaded", |bencher| {
			bencher.iter(|| {
				let _ = catalog.vfs().get("freedoom2/FCGRATE2").unwrap();
			});
		});

		grp.finish();
	}

	load_unload(crit);
	lookup(crit);
}

criterion_group!(benches, misc, data);
criterion_main!(benches);
