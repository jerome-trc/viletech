//! Microbenchmarking the [`append_only_vec`] crate.

use append_only_vec::AppendOnlyVec;

criterion::criterion_group!(benches, append_only_vec);
criterion::criterion_main!(benches);

fn append_only_vec(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("AppendOnlyVec");

	grp.bench_function("Push", |bencher| {
		bencher.iter_batched(
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
