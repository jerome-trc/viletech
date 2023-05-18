use doomfront::{rowan::ast::AstNode, util::testing::prettyprint};

use crate::vzs::{
	ast::{self},
	SyntaxNode,
};

use super::*;

#[test]
fn smoke_annotations() {
	const SOURCE: &str = r##"
#![vzs::legal]
#[vile::illegal]

#![legal_w_args(())]
#[::vile::illegal_w_args(::some_resolver, arg1: 123)]
"##;

	let ptree = parse_file::<GreenCacheNoop>(SOURCE, "/test.vzs", None).unwrap();
	assert_no_errors(&ptree);
	let cursor = SyntaxNode::new_root(ptree.root.clone());
	let mut ast = ast_iter(cursor);

	let ast::FileRoot::Annotation(anno0) = ast.next().unwrap();

	assert_eq!(anno0.resolver().syntax().text(), "vzs::legal");
	assert!(anno0.is_inner());

	ast.next().unwrap();
	ast.next().unwrap();

	let ast::FileRoot::Annotation(anno4) = ast.next().unwrap();

	assert_eq!(anno4.resolver().syntax().text(), "::vile::illegal_w_args");
	assert!(!anno4.is_inner());

	let args = anno4.args().unwrap();
	let mut args = args.iter();

	assert!(args.next().unwrap().label().is_none());
	assert_eq!(args.next().unwrap().label().unwrap().text(), "arg1");
}

#[test]
fn smoke_arglist() {
	const SOURCE: &str = "(barons)";
	let parser = common::arg_list();
	let mut state = ParseState::<GreenCacheNoop>::new(None);
	let res = parser.parse_with_state(SOURCE, &mut state);
	let _ = res.into_output().unwrap();
	let root = state.gtb.finish();
	let cursor = SyntaxNode::new_root(root.clone());
	prettyprint(cursor);
}

#[test]
fn smoke_exprs() {
	const SOURCE: &str = "BANQUET";
	let parser = expr::expr();
	let mut state = ParseState::<GreenCacheNoop>::new(None);
	let res = parser.parse_with_state(SOURCE, &mut state);
	let _ = res.into_output().unwrap();
	let root = state.gtb.finish();
	let cursor = SyntaxNode::new_root(root.clone());
	prettyprint(cursor);
}

#[test]
fn smoke_literals() {
	const SOURCES: &[&str] = &[
		"()",
		"true",
		"false",
		"12345_67890",
		"0b0001000__",
		"0xd_E_f_A_c_e",
		"0o77777",
		"1.",
		"1234567890.",
		"1__234__5.6__789__0",
		"123456789e0",
		"12345.6789e00",
		"1234.56789e+000",
		"123456.789e-0000",
		"\"knee deep in the code\"",
		"\"\"",
	];

	const EXPECTED: &[fn(ast::LitToken) -> bool] = &[
		|token| token.is_void(),
		|token| token.bool().filter(|b| *b).is_some(),
		|token| token.bool().filter(|b| !*b).is_some(),
		|token| token.int().unwrap().unwrap() == 12345_67890,
		|token| token.int().unwrap().unwrap() == 0b0001000__,
		|token| token.int().unwrap().unwrap() == 0xd_E_f_A_c_e,
		|token| token.int().unwrap().unwrap() == 0o77777,
		|token| token.float().unwrap() == 1.,
		|token| token.float().unwrap() == 1234567890.,
		|token| token.float().unwrap() == 1__234__5.6__789__0,
		|token| token.float().unwrap() == 123456789e0,
		|token| token.float().unwrap() == 12345.6789e00,
		|token| token.float().unwrap() == 1234.56789e+000,
		|token| token.float().unwrap() == 123456.789e-0000,
		|token| token.string().unwrap() == "knee deep in the code",
		|token| token.string().unwrap() == "",
	];

	for (i, &source) in SOURCES.iter().enumerate() {
		let parser = lit::literal();
		let mut state = ParseState::<GreenCacheNoop>::new(None);
		let res = parser.parse_with_state(source, &mut state);
		let _ = res.into_output().unwrap_or_else(|| {
			panic!("Failed to parse a literal from test case: `{source}`");
		});
		let root = state.gtb.finish();
		let cursor = SyntaxNode::new_root(root.clone());
		assert!(
			EXPECTED[i](ast::Literal::cast(cursor).unwrap().token()),
			"Literal parse test case failed: `{source}`"
		);
	}
}

#[test]
fn smoke_resolvers() {
	const SOURCE: &str = "::march::of::the::Demons";
	let parser = common::resolver();
	let mut state = ParseState::<GreenCacheNoop>::new(None);
	let res = parser.parse_with_state(SOURCE, &mut state);
	let _ = res.into_output().unwrap();
}

// Helpers /////////////////////////////////////////////////////////////////////

#[must_use]
fn ast_iter(cursor: SyntaxNode) -> impl Iterator<Item = ast::FileRoot> {
	cursor.children().map(|c| ast::FileRoot::cast(c).unwrap())
}

fn assert_no_errors(pt: &FileParseTree) {
	assert!(pt.errors.is_empty(), "Encountered errors: {}", {
		let mut output = String::default();

		for err in &pt.errors {
			output.push_str(&format!("{err:#?}\r\n"));
		}

		output
	});
}
