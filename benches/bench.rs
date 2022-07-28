use std::{env, path::PathBuf};

use criterion::{criterion_group, criterion_main, Criterion};
use impure::vfs::{ImpureVfs, VirtualFs};

pub fn vfs(crit: &mut Criterion) {
	let mut group = crit.benchmark_group("Mount");

	group.sample_size(10);

	group
		.bench_function("Gamedata", |b| {
			b.iter(|| {
				let pwd_gd: PathBuf = [
					env::current_dir().expect("Failed to retrieve PWD."),
					PathBuf::from("data"),
				]
				.iter()
				.collect();

				let mut vfs = VirtualFs::default();
				vfs.mount_gamedata(vec!(&pwd_gd));
			});
		});

	group.finish();
}

criterion_group!(benches, vfs);
criterion_main!(benches);
