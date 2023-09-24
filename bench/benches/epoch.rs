//! Microbenchmarking [`crossbeam::epoch`].

use std::sync::atomic;

use crossbeam::epoch;

criterion::criterion_group!(benches, epoch);
criterion::criterion_main!(benches);

fn epoch(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("Crossbeam Epoch");

	grp.bench_function("Pin", |bencher| {
		bencher.iter(|| {
			let _ = std::hint::black_box(epoch::pin());
		});
	});

	grp.bench_function("Atomic Load", |bencher| {
		let ptr = epoch::Atomic::new("???".to_string());
		let guard = epoch::pin();

		bencher.iter(|| {
			let _ = std::hint::black_box(ptr.load(atomic::Ordering::Acquire, &guard));
		});
	});

	grp.finish();
}
