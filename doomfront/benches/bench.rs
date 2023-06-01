use std::path::Path;

use criterion::Criterion;

use doomfront::util::builder::GreenCacheNoop;

fn doomfront(crit: &mut Criterion) {
	let mut grp = crit.benchmark_group("DoomFront");

	grp.bench_function("DECORATE Parser Build", |bencher| {
		bencher.iter(|| {
			let _ = doomfront::zdoom::decorate::parse::file::<GreenCacheNoop>();
		});
	});

	let sample = match std::fs::read(
		Path::new(env!("CARGO_MANIFEST_DIR")).join("../sample/decorate-sample.dec"),
	) {
		Ok(s) => s,
		Err(_) => {
			println!(
				"`$CARGO_MANIFEST_DIR/../sample/decorate-sample.dec` not found. \
				Skipping benchmark `DECORATE Parse`."
			);

			return;
		}
	};

	let sample = match std::str::from_utf8(&sample) {
		Ok(s) => s,
		Err(_) => {
			println!(
				"`$CARGO_MANIFEST_DIR/../sample/decorate-sample.dec` is invalid UTF-8. \
				Skipping benchmark `DECORATE Parse`."
			);

			return;
		}
	};

	grp.bench_function("DECORATE Parse", |bencher| {
		bencher.iter(|| {
			let parser = doomfront::zdoom::decorate::parse::file();

			let ptree = doomfront::parse(
				parser,
				Some(GreenCacheNoop),
				doomfront::zdoom::decorate::Syn::Root.into(),
				sample,
				doomfront::zdoom::lex::Token::stream(sample, None),
			);

			let _ = std::hint::black_box(ptree);
		});
	});

	grp.finish();
}

criterion::criterion_group!(benches, doomfront);
criterion::criterion_main!(benches);
