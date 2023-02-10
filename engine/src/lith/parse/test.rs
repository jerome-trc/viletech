use doomfront::rowan::ast::AstNode;

use crate::lith::ast;

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
		assert_eq!(fn0.name().text(), "native_fn");
		assert!(fn0.body().is_none());
	} else {
		panic!("Expected a top-level function, but failed to parse it.");
	}

	if let ast::Root::Item(ast::Item::FunctionDecl(fn1)) = ast.next().unwrap() {
		assert_eq!(fn1.name().text(), "builtin_fn");
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
		assert_eq!(fn2.name().text(), "my_fn");
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
#[::vile::illegal_w_args(::lith::int, arg1: vile::Pref)]

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

		assert!(args.next().unwrap().name().is_none());
		assert_eq!(args.next().unwrap().name().unwrap().text(), "arg1");
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
