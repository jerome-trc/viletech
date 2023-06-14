use std::path::PathBuf;

use criterion::Criterion;

fn udmf(crit: &mut Criterion) {
	const ENV_VAR: &str = "SUBTERRA_UDMF_SAMPLE";

	let path = match std::env::var(ENV_VAR) {
		Ok(v) => PathBuf::from(v),
		Err(_) => {
			eprintln!(
				"Environment variable not set: `{ENV_VAR}`. \
				Skipping UDMF parsing benchmarks."
			);
			return;
		}
	};

	let mut grp = crit.benchmark_group("UDMF");

	grp.sample_size(20);

	let bytes = std::fs::read(path)
		.map_err(|err| panic!("file I/O failure: {err}"))
		.unwrap();
	let source = String::from_utf8_lossy(&bytes);

	grp.bench_function("Parse, Hand-written", |bencher| {
		bencher.iter(|| {
			let result = subterra::udmf::parse_textmap(source.as_ref());
			let _ = std::hint::black_box(result.unwrap());
		});
	});

	grp.finish();
}

criterion::criterion_group!(benches, udmf);
criterion::criterion_main!(benches);
