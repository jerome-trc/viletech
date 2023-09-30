//! Microbenchmarking implementations of concurrent data structures.
//!
//! ...when under ideal, minimal-contention conditions.

use append_only_vec::AppendOnlyVec;

criterion::criterion_group!(benches, append_only_vec, boxcar);
criterion::criterion_main!(benches);

fn append_only_vec(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("AppendOnlyVec");

	grp.bench_function("Push", |bencher| {
		bencher.iter_batched_ref(
			|| AppendOnlyVec::new(),
			|aov| {
				let i = aov.push([0_usize, 0_usize, 0_usize]);
				let _ = std::hint::black_box(i);
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.bench_function("Access", |bencher| {
		let aov = AppendOnlyVec::new();
		let i = aov.push([0_usize, 0_usize, 0_usize]);

		bencher.iter(|| {
			let _ = std::hint::black_box(aov[i]);
		});
	});

	grp.finish();
}

fn boxcar(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("boxcar::Vec");

	grp.bench_function("Push", |bencher| {
		bencher.iter_batched_ref(
			|| boxcar::Vec::new(),
			|aov| {
				let i = aov.push([0_usize, 0_usize, 0_usize]);
				let _ = std::hint::black_box(i);
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.bench_function("Access", |bencher| {
		let aov = boxcar::Vec::new();
		let i = aov.push([0_usize, 0_usize, 0_usize]);

		bencher.iter(|| {
			let _ = std::hint::black_box(aov[i]);
		});
	});

	grp.finish();
}
