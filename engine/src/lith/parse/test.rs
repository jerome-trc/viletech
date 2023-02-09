use doomfront::rowan::ast::AstNode;

use crate::lith::ast;

use super::*;

#[test]
fn smoke() {
	const SOURCE: &str = r#"

void native_fn();
private ceval lith::void, ::lith::int builtin_fn();
abstract vile::Pref my_fn() {}

"#;

	let pt = parse(SOURCE, false, false).unwrap();
	assert!(!pt.any_errors(), "Encountered errors: {}", {
		let mut output = String::default();

		for err in pt.errors() {
			output.push_str(&format!("{err:#?}"));
			output.push('\r');
			output.push('\n');
		}

		output
	});

	let mut ast = pt.ast();

	let ast::Root::Item(ast::Item::FunctionDecl(fn0)) = ast.next().unwrap();
	assert_eq!(fn0.name().text(), "native_fn");
	assert!(fn0.body().is_none());

	let ast::Root::Item(ast::Item::FunctionDecl(fn1)) = ast.next().unwrap();
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

	let ast::Root::Item(ast::Item::FunctionDecl(fn2)) = ast.next().unwrap();
	assert_eq!(fn2.name().text(), "my_fn");
	assert!(fn2.body().is_some());
}
