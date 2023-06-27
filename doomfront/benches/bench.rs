use criterion::Criterion;

use doomfront::{
	testing::read_sample_data,
	zdoom::{self, zscript},
};

#[cfg(any())]
fn decorate(crit: &mut Criterion) {
	let mut grp = crit.benchmark_group("DECORATE");

	grp.sample_size(20);

	const SOURCE_EXPR: &str = "x ^ ((a * b) + (c / d)) | y & z && foo";

	grp.bench_function("Parser Build", |bencher| {
		bencher.iter(|| {
			let parser = decorate::parse::file();
			let _ = std::hint::black_box(parser);
		});
	});

	grp.bench_function("Expressions", |bencher| {
		bencher.iter(|| {
			let tbuf = doomfront::_scan(SOURCE_EXPR, zdoom::Version::V1_0_0);
			let parser = decorate::parse::expr();

			let ptree: decorate::ParseTree =
				doomfront::_parse(parser.clone(), SOURCE_EXPR, &tbuf).unwrap();

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
			let tbuf = doomfront::_scan(&sample, zdoom::Version::V1_0_0);
			let ptree: decorate::ParseTree = doomfront::_parse(parser, &sample, &tbuf).unwrap();
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
			let inctree = decorate::IncludeTree::new_par(
				|path: &Path| -> Option<Cow<str>> {
					let p = root_parent_path.join(path);

					if !p.exists() {
						return None;
					}

					let bytes = std::fs::read(p)
						.map_err(|err| panic!("file I/O failure: {err}"))
						.unwrap();
					let source = String::from_utf8_lossy(&bytes);
					Some(Cow::Owned(source.as_ref().to_owned()))
				},
				&root_path,
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

	grp.bench_function("Parse, Sample Data", |bencher| {
		bencher.iter(|| {
			let _ = std::hint::black_box(doomfront::parse(
				&sample,
				zdoom::language::parse::file,
				zdoom::lex::Context::NON_ZSCRIPT,
			));
		});
	});

	grp.finish();
}

fn zscript(crit: &mut Criterion) {
	let mut grp = crit.benchmark_group("ZScript");

	{
		const SOURCE: &str = "(a[1]() + b.c) * d && (e << f) ~== ((((g >>> h))))";

		grp.bench_function("Expressions", |bencher| {
			bencher.iter(|| {
				let _ = std::hint::black_box(doomfront::parse(
					SOURCE,
					zscript::parse::expr,
					zdoom::lex::Context::ZSCRIPT_LATEST,
				));
			});
		});
	}

	{
		let (_, sample) = match read_sample_data("DOOMFRONT_ZSCRIPT_SAMPLE") {
			Ok(s) => s,
			Err(err) => {
				eprintln!("Skipping ZScript sample data-based benchmarks. Reason: {err}");
				return;
			}
		};

		grp.bench_function("Parse, Sample Data", |bencher| {
			bencher.iter(|| {
				let ptree: zscript::ParseTree = std::hint::black_box(doomfront::parse(
					&sample,
					zscript::parse::file,
					zdoom::lex::Context::ZSCRIPT_LATEST,
				));

				assert!(!ptree.any_errors());
			});
		});

		grp.bench_function("Parse, Sample Data, Gutawer's", |bencher| {
			use zscript_parser::filesystem as zspfs;

			let mut files = zspfs::Files::default();
			let fndx = files.add(zspfs::File::new(
				"test.zs".to_string(),
				sample.as_bytes().to_vec(),
			));

			bencher.iter(|| {
				let result = zscript_parser::parser::Parser::new(fndx, &sample).parse();
				assert!(result.errs.is_empty());
			});
		});
	}

	grp.finish();
}

criterion::criterion_group!(benches, language, zscript);
criterion::criterion_main!(benches);
