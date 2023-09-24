//! Microbenchmarking the [`haphazard`] crate.

use haphazard::{AtomicPtr, HazardPointer};

criterion::criterion_group!(benches, haphazard);
criterion::criterion_main!(benches);

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
