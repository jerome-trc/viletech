use append_only_vec::AppendOnlyVec;
use criterion::Criterion;
use dashmap::{DashMap, DashSet};
use util::rstring::RString;

fn interning(crit: &mut Criterion) {
	let mut grp = crit.benchmark_group("Interning Insertion");

	grp.bench_function("DashSet", |bencher| {
		bencher.iter_batched(
			|| {
				let set = DashSet::new();
				set.insert(RString::new("warming it up first"));
				set
			},
			|set| {
				let _ = std::hint::black_box(set.contains("Hello world"));
				let clobbered = set.insert(RString::new("Hello world"));
				let _ = std::hint::black_box(clobbered);
			},
			criterion::BatchSize::LargeInput,
		);
	});

	grp.bench_function("DashMap + AppendOnlyVec", |bencher| {
		bencher.iter_batched(
			|| {
				let map = DashMap::new();
				let aovec = AppendOnlyVec::new();
				let string = RString::new("warming it up first");
				let ix = aovec.push(string.clone());
				map.insert(string, ix);
				(map, aovec)
			},
			|(map, aovec)| {
				let _ = std::hint::black_box(map.contains_key("Hello world"));
				let string = RString::new("Hello world");
				let ix = aovec.push(string.clone());
				map.insert(string, ix);
			},
			criterion::BatchSize::LargeInput,
		);
	});

	grp.finish();

	let mut grp = crit.benchmark_group("Interning Retrieval");

	grp.bench_function("DashSet", |bencher| {
		bencher.iter_batched(
			|| {
				let set = DashSet::new();
				let string = RString::new("warming it up first");
				set.insert(string.clone());
				(set, string)
			},
			|(_, string)| {
				let _ = std::hint::black_box(string.clone());
			},
			criterion::BatchSize::LargeInput,
		);
	});

	grp.bench_function("DashMap + AppendOnlyVec", |bencher| {
		bencher.iter_batched(
			|| {
				let map = DashMap::new();
				let aovec = AppendOnlyVec::new();
				let string = RString::new("warming it up first");
				let ix = aovec.push(string.clone());
				map.insert(string, ix);
				(map, aovec, ix)
			},
			|(_, aovec, ix)| {
				let _ = std::hint::black_box(&aovec[ix]);
			},
			criterion::BatchSize::LargeInput,
		);
	});

	grp.finish();

	let mut grp = crit.benchmark_group("Interned Comparison");

	grp.bench_function("RString (ASCII case ignored)", |bencher| {
		bencher.iter_batched(
			|| {
				let i1 = std::time::Instant::now();
				let i2 = std::time::Instant::now();

				(
					RString::new(format!("{i1:#?}")),
					RString::new(format!("{i2:#?}")),
				)
			},
			|(s1, s2)| s1.eq_ignore_ascii_case(&s2),
			criterion::BatchSize::SmallInput,
		);
	});

	grp.bench_function("u64", |bencher| {
		bencher.iter_batched(
			|| {
				let i1 = std::time::Instant::now();
				let i2 = std::time::Instant::now();
				let dur = i2.duration_since(i1);
				(dur.subsec_micros() as u64, dur.subsec_nanos() as u64)
			},
			|(u1, u2)| {
				let _ = std::hint::black_box(u1 == u2);
			},
			criterion::BatchSize::SmallInput,
		);
	});

	grp.finish();
}

criterion::criterion_group!(benches, interning);
criterion::criterion_main!(benches);
