//! Microbenchmarking [`crossbeam::epoch`], [`haphazard`], and [`seize`].

use std::sync::atomic;

use crossbeam::epoch;
use haphazard::{AtomicPtr, HazardPointer};

criterion::criterion_group!(benches, epoch, haphazard, seize);
criterion::criterion_main!(benches);

fn epoch(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("crossbeam::epoch");

	grp.bench_function("Pin", |bencher| {
		bencher.iter(|| {
			let _ = std::hint::black_box(epoch::pin());
		});
	});

	grp.bench_function("Atomic Load, Relaxed", |bencher| {
		let ptr = epoch::Atomic::new("???".to_string());
		let guard = epoch::pin();

		bencher.iter(|| {
			let _ = std::hint::black_box(ptr.load(atomic::Ordering::Relaxed, &guard));
		});
	});

	grp.bench_function("Atomic Load, Acquire", |bencher| {
		let ptr = epoch::Atomic::new("???".to_string());
		let guard = epoch::pin();

		bencher.iter(|| {
			let _ = std::hint::black_box(ptr.load(atomic::Ordering::Acquire, &guard));
		});
	});

	grp.bench_function("Atomic Load, SeqCst", |bencher| {
		let ptr = epoch::Atomic::new("???".to_string());
		let guard = epoch::pin();

		bencher.iter(|| {
			let _ = std::hint::black_box(ptr.load(atomic::Ordering::SeqCst, &guard));
		});
	});

	grp.finish();
}

fn haphazard(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("haphazard");

	let aptr = AtomicPtr::from(Box::new("???".to_string()));

	grp.bench_function("Hazard Pointer Creation", |bencher| {
		bencher.iter(|| {
			let _ = std::hint::black_box(HazardPointer::new());
		});
	});

	grp.bench_function("Hazard Load", |bencher| {
		let mut haz = HazardPointer::new();

		bencher.iter(|| {
			let _ = std::hint::black_box(aptr.safe_load(&mut haz).unwrap());
		});
	});

	grp.finish();
}

fn seize(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("seize");

	let collector = seize::Collector::new();

	let linked = collector.link_boxed("???".to_string());
	let aptr = seize::AtomicPtr::new(linked);

	grp.bench_function("Collector::enter", |bencher| {
		bencher.iter(|| {
			let _ = std::hint::black_box(collector.enter());
		});
	});

	grp.bench_function("Guard::protect, Relaxed", |bencher| {
		let guard = collector.enter();

		bencher.iter(|| {
			let _ = std::hint::black_box(guard.protect(&aptr, atomic::Ordering::Relaxed));
		});
	});

	grp.bench_function("Guard::protect, Acquire", |bencher| {
		let guard = collector.enter();

		bencher.iter(|| {
			let _ = std::hint::black_box(guard.protect(&aptr, atomic::Ordering::Acquire));
		});
	});

	grp.bench_function("Guard::protect, SeqCst", |bencher| {
		let guard = collector.enter();

		bencher.iter(|| {
			let _ = std::hint::black_box(guard.protect(&aptr, atomic::Ordering::SeqCst));
		});
	});

	grp.finish();
}
