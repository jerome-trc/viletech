criterion::criterion_group!(benches, mir_ops);
criterion::criterion_main!(benches);

fn mir_ops(_: &mut criterion::Criterion) {}
