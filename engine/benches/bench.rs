use criterion::Criterion;

/// Leave this here even if it's empty, so there's a quick scaffold ready
/// for one-off benchmarking experiments.
fn misc(crit: &mut Criterion) {
	#[allow(unused)]
	let mut grp = crit.benchmark_group("Miscellaneous");
	grp.finish();
}

criterion::criterion_group!(benches, misc);
criterion::criterion_main!(benches);
