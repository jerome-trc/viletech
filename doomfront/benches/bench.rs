use criterion::Criterion;

use doomfront::{testing::read_sample_data, zdoom};

fn decorate(_: &mut Criterion) {}

#[cfg(any())]
fn decorate(crit: &mut Criterion) {
	let mut grp = crit.benchmark_group("DECORATE");

	grp.sample_size(20);

	const SOURCE_EXPR: &str = "x ^ ((a * b) + (c / d)) | y & z && foo";

	grp.bench_function("Parser Build", |bencher| {
		bencher.iter(|| {
			let parser = decorate::parse::file::<GreenCacheNoop>();
			let _ = std::hint::black_box(parser);
		});
	});

	grp.bench_function("Expressions", |bencher| {
		let parser = decorate::parse::expr(false);

		bencher.iter(|| {
			let ptree = doomfront::parse(
				parser.clone(),
				Some(GreenCacheNoop),
				decorate::Syn::Root.into(),
				SOURCE_EXPR,
				zdoom::lex::Token::stream(SOURCE_EXPR),
			);

			let _ = std::hint::black_box(ptree);
		});
	});

	let (root_path, sample) = match read_sample_data("DOOMFRONT_DECORATE_SAMPLE") {
		Ok(s) => s,
		Err(err) => {
			eprintln!("Skipping DECORATE sample data-based benchmarks. Reason: {err}");
			return;
		}
	};

	grp.bench_function("Parse", |bencher| {
		bencher.iter(|| {
			let parser = decorate::parse::file();

			let ptree = doomfront::parse(
				parser,
				Some(GreenCacheNoop),
				decorate::Syn::Root.into(),
				&sample,
				doomfront::zdoom::lex::Token::stream(&sample),
			);

			let _ = std::hint::black_box(ptree);
		});
	});

	let Some(root_parent_path) = root_path.parent() else {
		eprintln!(
			"Skipping DECORATE include tree benchmark. Reason: `{}` has no parent.",
			root_path.display()
		);

		return;
	};

	grp.bench_function("Include Tree, Parallel, No Green Cache", |bencher| {
		bencher.iter(|| {
			let cache = GreenCacheNoop;

			let inctree = decorate::IncludeTree::new_par(
				|path: &Path| -> Option<Cow<str>> {
					let p = root_parent_path.join(path);

					if !p.exists() {
						return None;
					}

					let bytes = std::fs::read(p)
						.map_err(|err| panic!("File I/O failure: {err}"))
						.unwrap();
					let source = String::from_utf8_lossy(&bytes);
					Some(Cow::Owned(source.as_ref().to_owned()))
				},
				&root_path,
				Some(cache),
			)
			.unwrap();

			let _ = std::hint::black_box(inctree);
		});
	});

	grp.bench_function("Include Tree, Parallel, Green Cache", |bencher| {
		bencher.iter(|| {
			let cache = GreenCacheMt::default();

			let inctree = decorate::IncludeTree::new_par(
				|path: &Path| -> Option<Cow<str>> {
					let p = root_parent_path.join(path);

					if !p.exists() {
						return None;
					}

					let bytes = std::fs::read(p)
						.map_err(|err| panic!("File I/O failure: {err}"))
						.unwrap();
					let source = String::from_utf8_lossy(&bytes);
					Some(Cow::Owned(source.as_ref().to_owned()))
				},
				&root_path,
				Some(cache),
			)
			.unwrap();

			let _ = std::hint::black_box(inctree);
		});
	});

	grp.finish();
}

fn language(crit: &mut Criterion) {
	let (_, sample) = match read_sample_data("DOOMFRONT_LANGUAGE_SAMPLE") {
		Ok(s) => s,
		Err(err) => {
			eprintln!("Skipping LANGUAGE sample data-based benchmarks. Reason: {err}");
			return;
		}
	};

	let mut grp = crit.benchmark_group("LANGUAGE");

	grp.bench_function("Parse", |bencher| {
		bencher.iter(|| {
			let parser = zdoom::language::parse::file();

			let tbuf = doomfront::scan(&sample);

			let ptree =
				doomfront::parse::<zdoom::Token, zdoom::language::Syn>(parser, &sample, &tbuf);

			let _ = std::hint::black_box(ptree);
		});
	});

	grp.finish();
}

criterion::criterion_group!(benches, decorate, language);
criterion::criterion_main!(benches);
