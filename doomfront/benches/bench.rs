use std::{borrow::Cow, path::Path};

use criterion::Criterion;

use doomfront::{
	testing::read_sample_data,
	zdoom::{self, decorate, zscript},
};

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
			let tbuf = doomfront::scan(SOURCE_EXPR, zdoom::Version::V1_0_0);
			let parser = decorate::parse::expr();

			let ptree: decorate::ParseTree =
				doomfront::parse(parser.clone(), SOURCE_EXPR, &tbuf).unwrap();

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
			let tbuf = doomfront::scan(&sample, zdoom::Version::V1_0_0);
			let ptree: decorate::ParseTree = doomfront::parse(parser, &sample, &tbuf).unwrap();
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

	grp.bench_function("Parse, Chumsky", |bencher| {
		bencher.iter(|| {
			let parser = zdoom::language::parse::file();

			let tbuf = doomfront::scan(&sample, zdoom::Version::default());

			let ptree =
				doomfront::parse::<zdoom::Token, zdoom::language::Syn>(parser, &sample, &tbuf);

			let _ = std::hint::black_box(ptree);
		});
	});

	grp.bench_function("Parse, Handwritten", |bencher| {
		bencher.iter(|| {
			let mut parser = doomfront::parser::Parser::new(&sample, zdoom::Version::default());
			let ptree = zdoom::language::parse::hand::_file(&mut parser);
			let _ = std::hint::black_box(ptree);
		});
	});

	grp.finish();
}

fn zscript(crit: &mut Criterion) {
	let mut grp = crit.benchmark_group("ZScript");

	grp.bench_function("Expression", |bencher| {
		bencher.iter(|| {
			const SOURCE: &str = "(a[1]() + b.c) * d && (e << f) ~== ((((g >>> h))))";
			let builder = zscript::parse::ParserBuilder::new(zdoom::Version::default());
			let tbuf = doomfront::scan(SOURCE, zdoom::Version::default());
			let parser = builder.expr();
			let ptree: zscript::ParseTree = doomfront::parse(parser, SOURCE, &tbuf).unwrap();
			let _ = std::hint::black_box(ptree);
		});
	});

	grp.finish();
}

criterion::criterion_group!(benches, decorate, language, zscript);
criterion::criterion_main!(benches);
