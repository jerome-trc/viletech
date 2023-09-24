//! Microbenchmarking the locks provided by [`std`], [`parking_lot`], and [`crossbeam`].

use crossbeam::sync::ShardedLock;

criterion::criterion_group!(benches, std_sync, parking_lot, crossbeam,);
criterion::criterion_main!(benches);

fn std_sync(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("std::sync");

	grp.bench_function("Mutex", |bencher| {
		bencher.iter_batched_ref(
			|| std::sync::Mutex::new("???".to_string()),
			|lock| {
				let g = lock.lock().unwrap();
				let _ = std::hint::black_box(drop(g));
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.finish();
}

fn parking_lot(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("parking_lot");

	grp.bench_function("Mutex", |bencher| {
		bencher.iter_batched_ref(
			|| parking_lot::Mutex::new("???".to_string()),
			|lock| {
				let _ = std::hint::black_box(lock.lock());
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.finish();
}

fn crossbeam(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("crossbeam");

	grp.bench_function("ShardedLock Read", |bencher| {
		bencher.iter_batched_ref(
			|| ShardedLock::new("???".to_string()),
			|lock| {
				let _ = std::hint::black_box(lock.read().unwrap());
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.bench_function("ShardedLock Write", |bencher| {
		bencher.iter_batched_ref(
			|| ShardedLock::new("!!!".to_string()),
			|lock| {
				let _ = std::hint::black_box(lock.write().unwrap());
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.finish();
}
