//! Microbenchmarking the [`slotmap`] crate.

use slotmap::SlotMap;

criterion::criterion_group!(benches, slotmap);
criterion::criterion_main!(benches);

fn slotmap(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("SlotMap");

	grp.bench_function("Access", |bencher| {
		let mut sm = SlotMap::new();
		let _ = sm.insert("?");
		let _ = sm.insert("??");
		let k2 = sm.insert("???");

		bencher.iter(|| {
			let _ = std::hint::black_box(sm[k2].len());
		})
	});

	grp.bench_function("Access Unchecked", |bencher| {
		let mut sm = SlotMap::new();
		let _ = sm.insert("!");
		let _ = sm.insert("!!");
		let k2 = sm.insert("!!!");

		bencher.iter(|| unsafe {
			let _ = std::hint::black_box(sm.get_unchecked(k2).len());
		})
	});

	grp.finish();
}
