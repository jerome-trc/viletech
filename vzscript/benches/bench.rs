use std::path::Path;

use append_only_vec::AppendOnlyVec;
use criterion::Criterion;
use dashmap::{DashMap, DashSet};
use util::rstring::RString;
use vzscript::{
	compile::{Compiler, LibSource},
	IncludeTree,
};

fn frontend(crit: &mut Criterion) {
	let mut grp = crit.benchmark_group("Frontend");

	let Ok(zs_sample) = std::env::var("VZSCRIPT_SAMPLE_ZSCRIPT") else {
		eprintln!("Env. var. `VZSCRIPT_SAMPLE_ZSCRIPT` not defined.");
		eprintln!("Skipping frontend benchmarks.");
		return;
	};

	grp.bench_function("Declaration", |bencher| {
		bencher.iter_batched(
			|| {
				let _ = LibSource {
					name: "vzscript".to_string(),
					version: vzscript::Version::new(0, 0, 0),
					native: true,
					inctree: IncludeTree::from_fs(
						&Path::new(env!("CARGO_WORKSPACE_DIR")).join("assets/viletech"),
						Path::new("vzscript/main.vzs"),
						Some(vzscript::Version::new(0, 0, 0)),
					),
					decorate: None,
				};

				let userlib = LibSource {
					name: "userlib".to_string(),
					version: vzscript::Version::new(0, 0, 0),
					native: false,
					inctree: IncludeTree::from_fs(
						Path::new(&zs_sample),
						Path::new("ZSCRIPT.zs"),
						Some(vzscript::Version::new(0, 0, 0)),
					),
					decorate: None,
				};

				Compiler::new([/* TODO: add corelib */ userlib])
			},
			|mut compiler| {
				vzscript::front::declare_symbols(&mut compiler);
			},
			criterion::BatchSize::LargeInput,
		)
	});

	grp.finish();
}

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

criterion::criterion_group!(benches, frontend, interning);
criterion::criterion_main!(benches);
