//! Microbenchmarking the [`hybrid_rc`] crate.

use hybrid_rc::HybridRc;

criterion::criterion_group!(benches, hybrid_rc);
criterion::criterion_main!(benches);

fn hybrid_rc(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("hybrid_rc");

	let arc = hybrid_rc::Arc::new("???".to_string());
	let rc = HybridRc::to_local(&arc).unwrap();

	grp.bench_function("Clone RC", |bencher| {
		bencher.iter_with_large_drop(|| {
			let _ = std::hint::black_box(HybridRc::clone(&rc));
		});
	});

	grp.bench_function("Clone ARC", |bencher| {
		bencher.iter_with_large_drop(|| {
			let _ = std::hint::black_box(HybridRc::clone(&arc));
		});
	});

	grp.finish();
}
