criterion::criterion_group!(benches, threading);
criterion::criterion_main!(benches);

fn threading(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("Threading");

	grp.bench_function("Spawn", |bencher| {
		bencher.iter(|| {
			let j = std::thread::spawn(|| {});
			let _ = std::hint::black_box(j.join().unwrap());
		});
	});

	grp.bench_function("Thread ID", |bencher| {
		bencher.iter(|| {
			let _ = std::hint::black_box(std::thread::current().id());
		});
	});

	grp.bench_function("Available Parallelism", |bencher| {
		bencher.iter(|| {
			let _ = std::hint::black_box(std::thread::available_parallelism());
		});
	});

	grp.finish();
}
