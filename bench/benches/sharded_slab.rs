//! Microbenchmarking the [`sharded_slab`] crate.

use sharded_slab::{Pool, Slab};

criterion::criterion_group!(benches, slab, pool);
criterion::criterion_main!(benches);

fn slab(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("sharded_slab::Slab");

	grp.bench_function("Insert", |bencher| {
		bencher.iter_batched_ref(
			|| {
				let slab = Slab::new();
				let _ = slab.insert("warming it up first".to_string()).unwrap();
				(slab, "???".to_string())
			},
			|(slab, string)| {
				let _ = std::hint::black_box(slab.insert(std::mem::take(string)).unwrap());
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.bench_function("Access", |bencher| {
		bencher.iter_batched_ref(
			|| {
				let slab = Slab::new();
				let ix = slab.insert("???".to_string()).unwrap();
				let _ = slab.insert("!!!".to_string()).unwrap();
				let _ = slab.get(ix).unwrap();
				(slab, ix)
			},
			|(slab, ix)| {
				let _ = std::hint::black_box(slab.get(*ix).unwrap());
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.finish();
}

fn pool(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("sharded_slab::Pool");

	grp.bench_function("Insert", |bencher| {
		bencher.iter_batched_ref(
			|| {
				let pool = Pool::<String>::new();
				let _ = pool.create();
				pool
			},
			|pool| {
				let r = pool.create_with(|_| {});
				let _ = std::hint::black_box(r.unwrap());
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.bench_function("Access", |bencher| {
		bencher.iter_batched_ref(
			|| {
				let pool = Pool::<String>::new();
				let ix = pool.create_with(|s| s.push_str("???"));
				(pool, ix.unwrap())
			},
			|(pool, ix)| {
				let _ = std::hint::black_box(pool.get(*ix).unwrap());
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.finish();
}
