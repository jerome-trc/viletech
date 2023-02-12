use doomfront::rowan::ast::AstNode;

use crate::lith::ast::{self};

use super::*;

#[test]
fn smoke_fndecls() {
	const SOURCE: &str = r#"

void native_fn();
private ceval lith::void, ::lith::int builtin_fn();
abstract vile::Pref my_fn() {}

"#;

	let pt = parse(SOURCE, false, false).unwrap();

	assert_no_errors(&pt);

	let mut ast = pt.ast();

	if let ast::Root::Item(ast::Item::FunctionDecl(fn0)) = ast.next().unwrap() {
		assert_eq!(fn0.name().token().text(), "native_fn");
		assert!(fn0.body().is_none());
	} else {
		panic!("Expected a top-level function, but failed to parse it.");
	}

	if let ast::Root::Item(ast::Item::FunctionDecl(fn1)) = ast.next().unwrap() {
		assert_eq!(fn1.name().token().text(), "builtin_fn");
		assert!(fn1
			.qualifiers()
			.unwrap()
			.as_flags()
			.contains(ast::DeclQualifierFlags::PRIVATE | ast::DeclQualifierFlags::CEVAL));
		assert!(fn1.body().is_none());

		let mut fn1_rets = fn1.return_types();

		assert_eq!(fn1_rets.next().unwrap().syntax().text(), "lith::void");
		assert_eq!(fn1_rets.next().unwrap().syntax().text(), "::lith::int");
	} else {
		panic!("Expected a top-level function, but failed to parse it.");
	}

	if let ast::Root::Item(ast::Item::FunctionDecl(fn2)) = ast.next().unwrap() {
		assert_eq!(fn2.name().token().text(), "my_fn");
		assert!(fn2.body().is_some());
	} else {
		panic!("Expected a top-level function, but failed to parse it.");
	}
}

#[test]
fn smoke_annotations() {
	const SOURCE: &str = r##"

#![lith::legal]
#[vile::illegal]

#![legal_w_args(void)]
#[::vile::illegal_w_args(@[::lith::int], arg1: 123)]

"##;

	let pt = parse(SOURCE, false, false).unwrap();

	assert_no_errors(&pt);

	let mut ast = pt.ast();

	if let ast::Root::Annotation(anno0) = ast.next().unwrap() {
		assert_eq!(anno0.resolver().syntax().text(), "lith::legal");
		assert!(anno0.is_inner());
	} else {
		panic!("Expected a top-level annotation, but failed to parse it.");
	}

	ast.next().unwrap();
	ast.next().unwrap();

	if let ast::Root::Annotation(anno4) = ast.next().unwrap() {
		assert_eq!(anno4.resolver().syntax().text(), "::vile::illegal_w_args");
		assert!(!anno4.is_inner());

		let args = anno4.args().unwrap();
		let mut args = args.iter();

		assert!(args.next().unwrap().label().is_none());
		assert_eq!(args.next().unwrap().label().unwrap().token().text(), "arg1");
	} else {
		panic!("Expected a top-level annotation, but failed to parse it.");
	}
}

#[test]
fn smoke_literals() {
	const SOURCE: &str = r##"
	#![annotation(
		null,
		true,
		false,
		12345_67890,
		0b0001000__,
		0xd_E_f_A_c_e,
		0o77777,
		12345.67890,
		1234567890.,
		123456789e0,
		12345.6789e00,
		1234.56789e+000,
		123456.789e-0000,
		'\n',
		"knee deep in the code",
		""
	)]
	"##;

	let pt = parse(SOURCE, false, false).unwrap();

	assert_no_errors(&pt);

	let mut ast = pt.ast();

	if let ast::Root::Annotation(anno0) = ast.next().unwrap() {
		let args = anno0.args().unwrap();
		let mut args = args.iter();

		assert!(args
			.next()
			.unwrap()
			.expr()
			.into_literal()
			.unwrap()
			.token()
			.is_null());
		assert_eq!(
			args.next()
				.unwrap()
				.expr()
				.into_literal()
				.unwrap()
				.token()
				.bool()
				.unwrap(),
			true,
		);
		assert_eq!(
			args.next()
				.unwrap()
				.expr()
				.into_literal()
				.unwrap()
				.token()
				.bool()
				.unwrap(),
			false,
		);
		assert_eq!(
			args.next()
				.unwrap()
				.expr()
				.into_literal()
				.unwrap()
				.token()
				.int()
				.unwrap()
				.unwrap(),
			12345_67890,
		);
		assert_eq!(
			args.next()
				.unwrap()
				.expr()
				.into_literal()
				.unwrap()
				.token()
				.int()
				.unwrap()
				.unwrap(),
			0b0001000,
		);
		assert_eq!(
			args.next()
				.unwrap()
				.expr()
				.into_literal()
				.unwrap()
				.token()
				.int()
				.unwrap()
				.unwrap(),
			0xdeface,
		);
		assert_eq!(
			args.next()
				.unwrap()
				.expr()
				.into_literal()
				.unwrap()
				.token()
				.int()
				.unwrap()
				.unwrap(),
			0o77777,
		);
		assert_eq!(
			args.next()
				.unwrap()
				.expr()
				.into_literal()
				.unwrap()
				.token()
				.float()
				.unwrap(),
			12345.67890_f64,
		);
		assert_eq!(
			args.next()
				.unwrap()
				.expr()
				.into_literal()
				.unwrap()
				.token()
				.float()
				.unwrap(),
			1234567890_f64,
		);
		assert_eq!(
			args.next()
				.unwrap()
				.expr()
				.into_literal()
				.unwrap()
				.token()
				.float()
				.unwrap(),
			123456789e0_f64,
		);
		assert_eq!(
			args.next()
				.unwrap()
				.expr()
				.into_literal()
				.unwrap()
				.token()
				.float()
				.unwrap(),
			12345.6789e00,
		);
		assert_eq!(
			args.next()
				.unwrap()
				.expr()
				.into_literal()
				.unwrap()
				.token()
				.float()
				.unwrap(),
			1234.56789e+000,
		);
		assert_eq!(
			args.next()
				.unwrap()
				.expr()
				.into_literal()
				.unwrap()
				.token()
				.float()
				.unwrap(),
			123456.789e-0000,
		);
		assert_eq!(
			args.next()
				.unwrap()
				.expr()
				.into_literal()
				.unwrap()
				.token()
				.char()
				.unwrap(),
			'\n'
		);
		assert_eq!(
			args.next()
				.unwrap()
				.expr()
				.into_literal()
				.unwrap()
				.token()
				.string()
				.unwrap(),
			"knee deep in the code"
		);
		assert_eq!(
			args.next()
				.unwrap()
				.expr()
				.into_literal()
				.unwrap()
				.token()
				.string()
				.unwrap(),
			""
		);
	} else {
		panic!("Expected a top-level annotation, but failed to parse it.");
	}
}

fn assert_no_errors(pt: &ParseTree<Syn>) {
	assert!(!pt.any_errors(), "Encountered errors: {}", {
		let mut output = String::default();

		for err in pt.errors() {
			output.push_str(&format!("{err:#?}"));
			output.push('\r');
			output.push('\n');
		}

		output
	});
}
