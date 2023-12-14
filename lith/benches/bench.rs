criterion::criterion_group!(benches, cranelift_ops);
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
