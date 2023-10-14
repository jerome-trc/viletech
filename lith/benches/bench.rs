use append_only_vec::AppendOnlyVec;
use dashmap::DashMap;
use util::rstring::RString;

criterion::criterion_group!(benches, cranelift_ops, string_interning);
criterion::criterion_main!(benches);

fn cranelift_ops(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("Cranelift Ops");

	grp.bench_function("New CLIF Function", |bencher| {
		bencher.iter(|| {
			let _ = std::hint::black_box(cranelift::codegen::ir::Function::new());
		});
	});

	grp.bench_function("New CLIF Function Context", |bencher| {
		bencher.iter(|| {
			let _ = std::hint::black_box(cranelift::codegen::Context::new());
		});
	});

	grp.finish();
}

fn string_interning(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("String Interning");

	grp.bench_function("Insertion, DashMap + AppendOnlyVec (Entry)", |bencher| {
		bencher.iter_batched_ref(
			|| {
				let map = DashMap::new();
				let aovec = AppendOnlyVec::new();
				let string = RString::new("Lorem ipsum");
				let ix = aovec.push(string.clone());
				map.insert(string, ix as u32);
				(map, aovec)
			},
			|(map, aovec)| {
				let rstring = RString::new("Dolor sit amet");

				let dashmap::mapref::entry::Entry::Vacant(vac) = map.entry(rstring.clone()) else {
					unreachable!()
				};

				let ix = aovec.push(rstring);
				let _ = std::hint::black_box(vac.insert(ix as u32));
			},
			criterion::BatchSize::LargeInput,
		);
	});

	grp.bench_function(
		"Insertion, DashMap + AppendOnlyVec (Check -> Insert)",
		|bencher| {
			bencher.iter_batched_ref(
				|| {
					let map = DashMap::new();
					let aovec = AppendOnlyVec::new();
					let string = RString::new("Lorem ipsum");
					let ix = aovec.push(string.clone());
					map.insert(string, ix as u32);
					(map, aovec)
				},
				|(map, aovec)| {
					const STRING: &str = "Dolor sit amet";
					let _ = std::hint::black_box(map.contains_key(STRING));
					let rstring = RString::new(STRING);
					let ix = aovec.push(rstring.clone());
					let _ = std::hint::black_box(map.insert(rstring, ix as u32));
				},
				criterion::BatchSize::LargeInput,
			);
		},
	);

	grp.finish();
}
