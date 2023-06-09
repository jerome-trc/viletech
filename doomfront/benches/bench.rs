use std::{
	borrow::Cow,
	path::{Path, PathBuf},
};

use criterion::Criterion;

use doomfront::{
	util::builder::{GreenCacheMt, GreenCacheNoop},
	zdoom::{self, decorate},
};

fn decorate(crit: &mut Criterion) {
	const ENV_VAR: &str = "DOOMFRONT_DECORATE_SAMPLE";

	let mut grp = crit.benchmark_group("DECORATE");

	grp.sample_size(20);

	const SOURCE_EXPR: &str = "x ^ ((a * b) + (c / d)) | y & z && foo";

	grp.bench_function("Parser Build", |bencher| {
		bencher.iter(|| {
			let parser = decorate::parse::file::<GreenCacheNoop>();
			let _ = std::hint::black_box(parser);
		});
	});

	grp.bench_function("Expressions, Chumsky", |bencher| {
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

	grp.bench_function("Expressions, rust-peg", |bencher| {
		bencher.iter(|| {
			let root = decorate::parser::expr(SOURCE_EXPR).unwrap();
			let _ = std::hint::black_box(root);
		});
	});

	let root_path = match std::env::var(ENV_VAR) {
		Ok(v) => PathBuf::from(v),
		Err(_) => {
			eprintln!(
				"Environment variable not set: `{ENV_VAR}`. \
				Skipping benchmark `DECORATE/Parse`."
			);
			return;
		}
	};

	let sample = match std::fs::read(&root_path) {
		Ok(s) => s,
		Err(_) => {
			println!("`{ENV_VAR}` not found. Skipping benchmark `DECORATE/Parse`.");

			return;
		}
	};

	let sample = match std::str::from_utf8(&sample) {
		Ok(s) => s,
		Err(_) => {
			println!("`{ENV_VAR}` is invalid UTF-8. Skipping benchmark `DECORATE/Parse`.");

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
				sample,
				doomfront::zdoom::lex::Token::stream(sample),
			);

			let _ = std::hint::black_box(ptree);
		});
	});

	let Some(root_parent_path) = root_path.parent() else {
		eprintln!(
			"Path passed via `{ENV_VAR}` has no parent: {}",
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
	const ENV_VAR: &str = "DOOMFRONT_LANGUAGE_SAMPLE";

	let path = match std::env::var(ENV_VAR) {
		Ok(v) => PathBuf::from(v),
		Err(_) => {
			eprintln!(
				"Environment variable not set: `{ENV_VAR}`. \
				Cancelling `zdoom::language::parse::test::with_sample_data`."
			);
			return;
		}
	};

	if !path.exists() {
		eprintln!(
			"File does not exist: `{p}`. \
			Cancelling ``.",
			p = path.display(),
		);
		return;
	}

	let bytes = std::fs::read(path)
		.map_err(|err| panic!("File I/O failure: {err}"))
		.unwrap();
	let source = String::from_utf8_lossy(&bytes);

	let mut grp = crit.benchmark_group("LANGUAGE");

	grp.bench_function("Parse, Chumsky", |bencher| {
		bencher.iter(|| {
			let parser = zdoom::language::parse::file();

			let ptree = doomfront::parse(
				parser,
				Some(GreenCacheNoop),
				zdoom::language::Syn::Root.into(),
				source.as_ref(),
				zdoom::lex::Token::stream(source.as_ref()),
			);

			let _ = std::hint::black_box(ptree);
		});
	});

	grp.bench_function("Parse, Peg", |bencher| {
		bencher.iter(|| {
			let ptree = doomfront::ParseTree::<zdoom::lex::Token> {
				root: zdoom::language::parser::file(source.as_ref()).unwrap(),
				errors: vec![],
			};

			let _ = std::hint::black_box(ptree);
		});
	});

	grp.finish();
}

criterion::criterion_group!(benches, decorate, language);
criterion::criterion_main!(benches);
