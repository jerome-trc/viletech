use criterion::Criterion;

fn sim(_: &mut Criterion) {
	// Dummy function that will probably get used later,
	// so that `criterion_group` macro invocation doesn't fail.
}

criterion::criterion_group!(benches, sim);
criterion::criterion_main!(benches);
