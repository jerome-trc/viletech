use rowan::{ast::AstNode, GreenNode};

use crate::{
	parser::Parser,
	testing,
	zdoom::{
		self,
		zscript::{ast, parse, SyntaxNode},
	},
};

use super::*;

fn harness<E>(
	sample: &str,
	parse_fn: fn(&mut Parser<Syn>),
	conv_fn: fn(GreenNode) -> E,
	format_fn: fn(&mut AutoFormatter, E) -> GreenNode,
	callback: fn(formatted: GreenNode),
) {
	let ptree = crate::parse(sample, parse_fn, zdoom::lex::Context::ZSCRIPT_LATEST);
	testing::assert_no_errors(&ptree);

	let cfg = Config::new(LineEnds::Lf);
	let cache = Cache::default();

	let mut f = AutoFormatter::new(&cfg, &cache);
	let interm = conv_fn(ptree.into_green());
	let formatted = format_fn(&mut f, interm);

	{
		let ftxt = format!("{}", SyntaxNode::new_root(formatted.clone()));
		let reparsed = crate::parse(&ftxt, parse_fn, zdoom::lex::Context::ZSCRIPT_LATEST);
		assert!(!reparsed.any_errors());
	}

	callback(formatted);
}

fn assert_text_eq(expected: &'static str, formatted: GreenNode) {
	let cursor = SyntaxNode::new_root(formatted);
	let fmt_txt = format!("{}", cursor.text());

	if fmt_txt != expected {
		panic!(
			"Expected: {nl2}{expected}{nl2} but formatting produced {nl2}{fmt_txt}{nl2}",
			nl2 = "\r\n\r\n"
		);
	}
}

// Expressions /////////////////////////////////////////////////////////////////

#[test]
fn smoke_expr_bin() {
	const SAMPLE: &str = "2/* */+  2";
	const EXPECTED: &str = "2 /* */ + 2";

	harness(
		SAMPLE.trim(),
		parse::expr,
		|green| ast::BinExpr::cast(SyntaxNode::new_root(green)).unwrap(),
		expr_bin,
		|formatted| {
			assert_text_eq(EXPECTED, formatted);
		},
	);
}
