//! Microbenchmarking immutable data structures from [`im`], [`im_rc`] and [`rpds`].

use viletech_bench::util;

criterion::criterion_group!(benches, im, im_rc, rpds);
criterion::criterion_main!(benches);

fn im(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("im::HashMap");

	grp.bench_function("New", |bencher| {
		bencher.iter_with_large_drop(|| {
			std::hint::black_box(im::HashMap::<&'static str, &'static str>::new())
		});
	});

	grp.bench_function("Insert", |bencher| {
		bencher.iter_batched_ref(
			|| {
				let map = im::HashMap::new();
				(map, "???".to_string())
			},
			|(map, value)| {
				let new_map = map.insert("!!!", std::mem::take(value));
				let _ = std::hint::black_box(new_map);
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.finish();
}

fn im_rc(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("im_rc::HashMap");

	grp.bench_function("Insert", |bencher| {
		bencher.iter_batched_ref(
			|| {
				let map = im_rc::HashMap::new();
				(map, "???".to_string())
			},
			|(map, value)| {
				let new_map = map.insert("!!!", std::mem::take(value));
				let _ = std::hint::black_box(new_map);
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.finish();
}

fn rpds(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("rpds::HashTrieMap");

	grp.bench_function("New", |bencher| {
		bencher.iter_with_large_drop(|| {
			std::hint::black_box(rpds::HashTrieMap::<&'static str, &'static str>::new())
		});
	});

	grp.bench_function("Insert", |bencher| {
		bencher.iter_batched_ref(
			|| {
				let map = rpds::HashTrieMap::new();
				(map, "???".to_string())
			},
			|(map, value)| {
				let new_map = map.insert("!!!", std::mem::take(value));
				let _ = std::hint::black_box(new_map);
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.bench_function("Insert, RcK", |bencher| {
		bencher.iter_batched_ref(
			|| {
				let map = rpds::HashTrieMap::<_, _, util::archery::RcK>::default();
				(map, "???".to_string())
			},
			|(map, value)| {
				let new_map = map.insert("!!!", std::mem::take(value));
				let _ = std::hint::black_box(new_map);
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.bench_function("Insert, TArcK", |bencher| {
		bencher.iter_batched_ref(
			|| {
				let map = rpds::HashTrieMap::<_, _, util::arck::TArcK>::default();
				(map, "???".to_string())
			},
			|(map, value)| {
				let new_map = map.insert("!!!", std::mem::take(value));
				let _ = std::hint::black_box(new_map);
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.finish();
}
