use std::path::{Path, PathBuf};

use viletech_fs::{VPath, VirtualFs};

criterion::criterion_group!(benches, operations);
criterion::criterion_main!(benches);

fn operations(crit: &mut criterion::Criterion) {
	let Some(freedoom2) = freedoom2_path() else {
		return;
	};

	let mut grp = crit.benchmark_group("Mount");

	grp.bench_function("WAD", |bencher| {
		bencher.iter_batched_ref(
			VirtualFs::default,
			|vfs| {
				vfs.mount(&freedoom2, VPath::new("freedoom2")).unwrap();
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.finish();

	let mut grp = crit.benchmark_group("Lookup");

	grp.bench_function("WAD, Best-Case", |bencher| {
		let mut vfs = VirtualFs::default();
		vfs.mount(&freedoom2, VPath::new("freedoom2")).unwrap();

		bencher.iter(|| {
			let _ = std::hint::black_box(vfs.get(VPath::new("/freedoom2/MAP01")).unwrap());
		});
	});

	grp.bench_function("WAD, Worst-Case", |bencher| {
		let mut vfs = VirtualFs::default();
		vfs.mount(&freedoom2, VPath::new("freedoom2")).unwrap();

		bencher.iter(|| {
			let _ = std::hint::black_box(vfs.get(VPath::new("/freedoom2/FCGRATE2")).unwrap());
		});
	});

	grp.finish();
}

#[must_use]
fn freedoom2_path() -> Option<PathBuf> {
	let Ok(evar) = std::env::var("VILETECHFS_SAMPLE_DIR") else {
		eprintln!("Env. var. `VILETECHFS_SAMPLE_DIR` not set; skipping benchmarks.");
		return None;
	};

	let ret = Path::new(&evar).join("freedoom2.wad");

	if !ret.exists() {
		eprintln!(
			"`{}` not found on the disk; skipping benchmarks.",
			ret.display()
		);

		return None;
	}

	Some(ret)
}
