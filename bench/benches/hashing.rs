//! Microbenchmarking various hashing algorithms.

use std::{hash::Hasher, io::Read, path::Path, sync::OnceLock};

criterion::criterion_group!(benches, fx, md5);
criterion::criterion_main!(benches);

static SAMPLE: OnceLock<Vec<u8>> = OnceLock::new();

fn sample_init() -> Vec<u8> {
	let path = Path::new(env!("CARGO_WORKSPACE_DIR")).join(".gitattributes");
	let mut file = std::fs::File::open(path).expect("failed to read workspace .gitattributes");
	let mut buf = vec![];
	let _ = file.read_to_end(&mut buf).unwrap();
	buf
}

fn fx(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("FxHash");

	grp.bench_function("~1KB", |bencher| {
		let sample = SAMPLE.get_or_init(sample_init);

		bencher.iter_batched(
			|| rustc_hash::FxHasher::default(),
			|mut hasher| {
				hasher.write(sample);
				let _ = std::hint::black_box(hasher.finish());
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.finish();
}

fn md5(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("MD5");

	grp.bench_function("~1KB", |bencher| {
		let sample = SAMPLE.get_or_init(sample_init);

		bencher.iter(|| {
			let _ = std::hint::black_box(md5::compute(sample));
		});
	});

	grp.finish();
}
