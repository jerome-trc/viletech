//! Microbenchmarking Rust's dynamic polymorphism features.

use std::any::Any;

criterion::criterion_group!(benches, rt_poly);
criterion::criterion_main!(benches);

fn rt_poly(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("RT Polymorphism");

	grp.bench_function("Downcast", |bencher| {
		let b: Box<dyn Any> = Box::new("???");

		bencher.iter(|| {
			let d = b.downcast_ref::<&str>().unwrap();
			let _ = std::hint::black_box(d.len());
		});
	});

	grp.bench_function("Downcast Unchecked", |bencher| {
		let b: Box<dyn Any> = Box::new("???");

		bencher.iter(|| unsafe {
			let d = b.downcast_ref::<&str>().unwrap_unchecked();
			let _ = std::hint::black_box(d.len());
		});
	});

	grp.finish();
}
