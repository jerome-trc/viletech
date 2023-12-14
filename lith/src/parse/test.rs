use doomfront::{
	rowan::ast::AstNode,
	testing::{assert_no_errors, prettyprint_maybe},
};

use crate::{ast, LexContext, ParseTree};

/// Yes, seriously.
#[test]
fn empty() {
	let ptree: ParseTree = doomfront::parse("", super::chunk, LexContext::default());
	assert_no_errors(&ptree);

	if prettyprint_maybe(ptree.cursor()) {
		eprintln!();
	}
}

#[test]
fn name_smoke() {
	const SAMPLES: &[&str] = &["'lorem_ipsum'", "lorem_ipsum"];

	for sample in SAMPLES {
		let ptree: ParseTree = doomfront::parse(
			sample,
			|p| {
				super::expr(p);
			},
			LexContext::default(),
		);

		assert_no_errors(&ptree);

		if prettyprint_maybe(ptree.cursor()) {
			eprintln!();
		}

		let token0 = ptree.cursor().first_token().unwrap();
		let name = ast::Name(token0);
		assert_eq!(name.text(), "lorem_ipsum");
	}
}

#[test]
#[ignore]
fn with_sample_data() {
	let (_, sample) = match doomfront::testing::read_sample_data("LITHICA_PARSE_SAMPLE") {
		Ok(s) => s,
		Err(err) => {
			eprintln!("Skipping sample data-based unit test. Reason: {err}");
			return;
		}
	};

	let ptree: ParseTree = doomfront::parse(&sample, super::chunk, LexContext::default());
	assert_no_errors(&ptree);
	prettyprint_maybe(ptree.cursor());
}

// Expressions /////////////////////////////////////////////////////////////////

#[test]
fn expr_lit_float_smoke() {
	const SAMPLES: &[&str] = &["0.", "0.1", "0_.", "0_.1", "0_.1_", "0_.1_f32"];

	for sample in SAMPLES {
		let ptree: ParseTree = doomfront::parse(
			sample,
			|p| {
				super::expr(p);
			},
			LexContext::default(),
		);

		assert_no_errors(&ptree);

		if prettyprint_maybe(ptree.cursor()) {
			eprintln!();
		}

		let ast = ast::ExprLit::cast(ptree.cursor()).unwrap();
		let lit = ast.token();

		match lit.float() {
			Some(Ok(_)) => {}
			Some(Err(err)) => {
				panic!("failed to parse float literal sample `{sample}`: {err}");
			}
			_ => panic!("failed to lex float literal"),
		};
	}
}

#[test]
fn expr_lit_decimal_smoke() {
	const SAMPLES: &[&str] = &["0", "0_", "0_u8", "0_i128"];

	for sample in SAMPLES {
		let ptree: ParseTree = doomfront::parse(
			sample,
			|p| {
				super::expr(p);
			},
			LexContext::default(),
		);

		assert_no_errors(&ptree);

		if prettyprint_maybe(ptree.cursor()) {
			eprintln!();
		}

		let ast = ast::ExprLit::cast(ptree.cursor()).unwrap();
		let lit = ast.token();

		match lit.int() {
			Some(Ok(_)) => {}
			Some(Err(err)) => {
				panic!("failed to parse decimal literal sample `{sample}`: {err}");
			}
			_ => panic!("failed to lex decimal literal"),
		};
	}
}

#[test]
fn expr_lit_suffixed_string_smoke() {
	const SAMPLE: &str = "\"lorem ipsum\"_dolor_sit_amet";

	let ptree: ParseTree = doomfront::parse(
		SAMPLE,
		|p| {
			super::expr(p);
		},
		LexContext::default(),
	);

	assert_no_errors(&ptree);

	if prettyprint_maybe(ptree.cursor()) {
		eprintln!();
	}

	let ast = ast::ExprLit::cast(ptree.cursor()).unwrap();
	assert_eq!(ast.token().string().unwrap(), "lorem ipsum");
	assert_eq!(ast.string_suffix().unwrap().text(), "_dolor_sit_amet");
}

#[test]
fn expr_bin_userop_smoke() {
	const SAMPLE: &str = "a @dot b";

	let ptree: ParseTree = doomfront::parse(
		SAMPLE,
		|p| {
			super::expr(p);
		},
		LexContext::default(),
	);

	assert_no_errors(&ptree);

	if prettyprint_maybe(ptree.cursor()) {
		eprintln!();
	}

	let ast = ast::ExprBin::cast(ptree.cursor()).unwrap();

	let ast::BinOp::User { ident, .. } = ast.operator().unwrap() else {
		panic!()
	};

	assert_eq!(ident.text(), "dot");
}
