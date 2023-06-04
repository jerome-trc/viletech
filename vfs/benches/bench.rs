use std::path::{Path, PathBuf};

use criterion::Criterion;
use viletech_fs::{MountOutcome, MountRequest, VPathBuf, VirtualFs};

criterion::criterion_group!(benches, mount, lookup);
criterion::criterion_main!(benches);

fn mount(crit: &mut Criterion) {
	if load_order().is_none() {
		return;
	}

	let mut grp = crit.benchmark_group("FreeDoom 1 and 2");

	grp.bench_function("Mount", |bencher| {
		bencher.iter_batched(
			|| {
				(
					VirtualFs::default(),
					MountRequest {
						load_order: load_order().unwrap(),
						basedata: false,
						tracker: None,
					},
				)
			},
			|(mut vfs, req)| {
				if let MountOutcome::Errs(_) = std::hint::black_box(vfs.mount(req)) {
					panic!("VFS mount benchmark encountered errors.");
				}
			},
			criterion::BatchSize::LargeInput,
		);
	});

	grp.bench_function("Unmount", |bencher| {
		bencher.iter_batched(
			|| {
				let mut vfs = VirtualFs::default();

				let req = MountRequest {
					load_order: load_order().unwrap(),
					basedata: false,
					tracker: None,
				};

				if let MountOutcome::Errs(_) = vfs.mount(req) {
					panic!("VFS unmount benchmark encountered errors.");
				}

				vfs
			},
			|mut vfs| {
				let _ = vfs.truncate(0);
			},
			criterion::BatchSize::LargeInput,
		);
	});

	grp.finish();
}

fn lookup(crit: &mut Criterion) {
	if load_order().is_none() {
		return;
	}

	let mut grp = crit.benchmark_group("FreeDoom 1 and 2");

	let mut vfs = VirtualFs::default();

	let req = MountRequest {
		load_order: load_order().unwrap(),
		basedata: false,
		tracker: None,
	};

	if let MountOutcome::Errs(_) = vfs.mount(req) {
		panic!("VFS lookup benchmark encountered errors.");
	}

	grp.bench_function("Lookup", |bencher| {
		bencher.iter(|| {
			let _ = std::hint::black_box(vfs.get("/freedoom1/E1M1").unwrap());
		});
	});

	grp.finish();
}

// Helpers /////////////////////////////////////////////////////////////////////

#[must_use]
fn load_order() -> Option<Vec<(PathBuf, VPathBuf)>> {
	let sample = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sample");

	let ret = vec![
		(sample.join("freedoom1.wad"), PathBuf::from("/freedoom1")),
		(sample.join("freedoom2.wad"), PathBuf::from("/freedoom2")),
	];

	for (real_path, _) in &ret {
		if !real_path.exists() {
			eprintln!(
				"VFS benchmarking depends on the following files of sample data:\r\n\t\
				- `$CARGO_MANIFEST_DIR/sample/freedoom1.wad`\r\n\t\
				- `$CARGO_MANIFEST_DIR/sample/freedoom2.wad`\r\n\t\
				They can be acquired from https://freedoom.github.io/."
			);

			return None;
		}
	}

	Some(ret)
}
