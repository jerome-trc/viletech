use subterra::gfx::PictureReader;

fn graphics(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("Graphics");

	grp.bench_function("PictureReader::new", |bencher| {
		let pic = include_bytes!("../../sample/freedoom/STFST01.lmp");

		bencher.iter(|| {
			let _ = std::hint::black_box(PictureReader::new(pic).unwrap());
		});
	});

	grp.finish();
}

criterion::criterion_group!(benches, graphics);
criterion::criterion_main!(benches);
