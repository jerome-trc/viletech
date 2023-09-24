//! Microbenchmarking the [`arc_swap`] crate.

use std::sync::Arc;

use arc_swap::ArcSwap;

criterion::criterion_group!(benches, arc_swap);
criterion::criterion_main!(benches);

fn arc_swap(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("ArcSwap");

	grp.bench_function("Load", |bencher| {
		let arcswap = ArcSwap::from_pointee("???".to_string());

		bencher.iter(|| {
			let g = arcswap.load();
			let _ = std::hint::black_box(g.len());
		});
	});

	grp.bench_function("Store", |bencher| {
		bencher.iter_batched(
			|| {
				(
					ArcSwap::from_pointee("???".to_string()),
					Arc::new("!!!".to_string()),
				)
			},
			|(arcswap, arc)| {
				let _ = std::hint::black_box(arcswap.store(arc));
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.bench_function("RCU", |bencher| {
		bencher.iter_batched(
			|| ArcSwap::from_pointee("???".to_string()),
			|arcswap| {
				arcswap.rcu(|prev| prev.clone());
				let _ = std::hint::black_box(arcswap);
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.finish();
}
