//! Microbenchmarking implementations of concurrent data structures.
//!
//! ...when under ideal, minimal-contention conditions.

use append_only_vec::AppendOnlyVec;
use crossbeam::queue::SegQueue;
use sharded_slab::Slab;

criterion::criterion_group!(
	benches,
	append_only_vec,
	boxcar,
	crossbeam_queues,
	sharded_slab,
	sharded_slab_pool
);
criterion::criterion_main!(benches);

fn append_only_vec(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("AppendOnlyVec");

	grp.bench_function("Push", |bencher| {
		bencher.iter_batched_ref(
			|| AppendOnlyVec::new(),
			|aov| {
				let i = aov.push([0_usize, 0_usize, 0_usize]);
				let _ = std::hint::black_box(i);
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.bench_function("Access", |bencher| {
		let aov = AppendOnlyVec::new();
		let i = aov.push([0_usize, 0_usize, 0_usize]);

		bencher.iter(|| {
			let _ = std::hint::black_box(aov[i]);
		});
	});

	grp.finish();
}

fn boxcar(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("boxcar::Vec");

	grp.bench_function("Push", |bencher| {
		bencher.iter_batched_ref(
			|| boxcar::Vec::new(),
			|aov| {
				let i = aov.push([0_usize, 0_usize, 0_usize]);
				let _ = std::hint::black_box(i);
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.bench_function("Access", |bencher| {
		let aov = boxcar::Vec::new();
		let i = aov.push([0_usize, 0_usize, 0_usize]);

		bencher.iter(|| {
			let _ = std::hint::black_box(aov[i]);
		});
	});

	grp.finish();
}

fn crossbeam_queues(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("crossbeam::SegQueue");

	grp.bench_function("Push", |bencher| {
		bencher.iter_batched_ref(
			|| SegQueue::new(),
			|q| {
				let _ = std::hint::black_box(q.push([0_usize, 0_usize, 0_usize]));
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.bench_function("Pop", |bencher| {
		bencher.iter_batched_ref(
			|| {
				let q = SegQueue::new();
				q.push([0_usize, 0_usize, 0_usize]);
				q
			},
			|q| {
				let _ = std::hint::black_box(q.pop().unwrap());
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.finish();
}

fn sharded_slab(crit: &mut criterion::Criterion) {
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

fn sharded_slab_pool(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("sharded_slab::Pool");

	grp.bench_function("Insert", |bencher| {
		bencher.iter_batched_ref(
			|| {
				let pool = sharded_slab::Pool::<String>::new();
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
				let pool = sharded_slab::Pool::<String>::new();
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
