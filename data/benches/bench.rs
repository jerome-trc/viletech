use criterion::Criterion;

fn udmf(_: &mut Criterion) {
	// TODO
}

criterion::criterion_group!(benches, udmf);
criterion::criterion_main!(benches);
