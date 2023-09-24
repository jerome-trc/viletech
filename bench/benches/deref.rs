//! Microbenchmarking time to dereference a pointer vs. time to index a [`Vec`].

use std::sync::Arc;

criterion::criterion_group!(benches, dereference);
criterion::criterion_main!(benches);

fn dereference(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("Dereference");

	grp.bench_function("Arc Deref.", |bencher| {
		let string: Arc<str> = Arc::from("???".to_string());

		bencher.iter(|| {
			let _ = std::hint::black_box(string.len());
		});
	});

	grp.bench_function("Vec Index", |bencher| {
		let v = vec!["", "", ""];

		bencher.iter(|| {
			let _ = std::hint::black_box(v[2].len());
		});
	});

	grp.bench_function("Vec Index Unchecked", |bencher| {
		let v = vec!["", "", ""];

		bencher.iter(|| unsafe {
			let _ = std::hint::black_box(v.get_unchecked(2).len());
		})
	});

	grp.finish();
}
