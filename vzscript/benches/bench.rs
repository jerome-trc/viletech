use criterion::Criterion;
use util::rstring::RString;

fn interning(crit: &mut Criterion) {
	let mut grp = crit.benchmark_group("Interning");

	grp.bench_function("DashSet", |bencher| {
		bencher.iter_batched(
			|| {
				let set = dashmap::DashSet::new();
				set.insert(RString::new("warming it up first"));
				set
			},
			|set| {
				set.insert(RString::new("Hello world"));
			},
			criterion::BatchSize::LargeInput,
		);
	});

	grp.bench_function("Sharded Slab + DashMap", |bencher| {
		bencher.iter_batched(
			|| {
				let slab = sharded_slab::Slab::default();
				let map = dashmap::DashMap::new();
				let string = RString::new("warming it up first");
				let ix = slab.insert(string.clone()).unwrap();
				map.insert(string, ix);
				(slab, map)
			},
			|(slab, map)| {
				let string = RString::new("hello world");
				let ix = slab.insert(string.clone()).unwrap();
				map.insert(string, ix);
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.finish();
}

criterion::criterion_group!(benches, interning);
criterion::criterion_main!(benches);
