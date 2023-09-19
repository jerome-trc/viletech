use std::{any::Any, sync::Arc};

use criterion::Criterion;
use slotmap::SlotMap;

criterion::criterion_group!(benches, dereference, rt_poly);
criterion::criterion_main!(benches);

fn dereference(crit: &mut Criterion) {
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

	grp.bench_function("SlotMap Index", |bencher| {
		let mut sm = SlotMap::new();
		let _ = sm.insert("");
		let _ = sm.insert("");
		let k2 = sm.insert("");

		bencher.iter(|| {
			let _ = std::hint::black_box(sm[k2].len());
		})
	});

	grp.bench_function("SlotMap Index Unchecked", |bencher| {
		let mut sm = SlotMap::new();
		let _ = sm.insert("");
		let _ = sm.insert("");
		let k2 = sm.insert("");

		bencher.iter(|| unsafe {
			let _ = std::hint::black_box(sm.get_unchecked(k2).len());
		})
	});

	grp.finish();
}

fn rt_poly(crit: &mut Criterion) {
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
