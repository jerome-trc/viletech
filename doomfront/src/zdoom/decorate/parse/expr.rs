use chumsky::{primitive, recursive::Recursive, IterParser, Parser};
use rowan::GreenNode;

use crate::{
	comb, parser_t,
	parsing::*,
	zdoom::{decorate::Syn, lex::Token},
	GreenElement,
};

use super::common::*;

/// The returned parser emits a node tagged with one of the following:
/// - [`Syn::ArrayExpr`]
/// - [`Syn::BinExpr`]
/// - [`Syn::CallExpr`]
/// - [`Syn::GroupExpr`]
/// - [`Syn::IdentExpr`]
/// - [`Syn::IndexExpr`]
/// - [`Syn::PostfixExpr`]
/// - [`Syn::PrefixExpr`]
/// - [`Syn::SuperExpr`]
pub fn expr<'i>() -> parser_t!(GreenNode) {
	chumsky::recursive::recursive(
		|expr: Recursive<dyn chumsky::Parser<'_, _, GreenNode, _>>| {
			let ident =
				ident_chain().map(|gnode| GreenNode::new(Syn::IdentExpr.into(), [gnode.into()]));

			let literal = primitive::choice((
				comb::just_ts(Token::StringLit, Syn::StringLit),
				comb::just_ts(Token::IntLit, Syn::IntLit),
				comb::just_ts(Token::FloatLit, Syn::FloatLit),
				comb::just_ts(Token::NameLit, Syn::NameLit),
				comb::just_ts(Token::KwTrue, Syn::KwTrue),
				comb::just_ts(Token::KwFalse, Syn::KwFalse),
			))
			.map_with_state(|gtok, _, _| GreenNode::new(Syn::Literal.into(), [gtok.into()]));

			let grouped = primitive::group((
				comb::just_ts(Token::ParenL, Syn::ParenL),
				trivia_0plus(),
				expr.clone(),
				trivia_0plus(),
				comb::just_ts(Token::ParenR, Syn::ParenR),
			))
			.map(|group| coalesce_node(group, Syn::GroupExpr));

			let atom = primitive::choice((grouped, literal, ident.clone()));

			let subscript = primitive::group((
				comb::just_ts(Token::BracketL, Syn::BracketL),
				trivia_0plus(),
				expr.clone(),
				trivia_0plus(),
				comb::just_ts(Token::BracketR, Syn::BracketR),
			))
			.map(|group| coalesce_node(group, Syn::Subscript));

			let index = atom.foldl(subscript.repeated(), |lhs, subscript| {
				GreenNode::new(Syn::IndexExpr.into(), [lhs.into(), subscript.into()])
			});

			let arg_list = primitive::group((
				comb::just_ts(Token::ParenL, Syn::ParenL),
				expr_list(expr.clone()).or_not(),
				comb::just_ts(Token::ParenR, Syn::ParenR),
			))
			.map(|group| coalesce_node(group, Syn::ArgList));

			let call = index.clone().foldl(arg_list.repeated(), |lhs, arg_list| {
				GreenNode::new(Syn::CallExpr.into(), [lhs.into(), arg_list.into()])
			});

			// TODO: Unary

			let op12 = primitive::group((
				trivia_0plus(),
				comb::just_ts(Token::Dot, Syn::Dot),
				trivia_0plus(),
			))
			.map(coalesce_vec);

			let bin12 = call
				.clone()
				.foldl(op12.then(ident).repeated(), |lhs, (op, rhs)| {
					let mut elems = op;
					elems.insert(0, lhs.into());
					elems.push(rhs.into());
					GreenNode::new(Syn::BinExpr.into(), elems)
				})
				.boxed();

			let op11 = primitive::group((
				trivia_0plus(),
				primitive::choice((
					comb::just_ts(Token::Asterisk, Syn::Asterisk),
					comb::just_ts(Token::Slash, Syn::Slash),
					comb::just_ts(Token::Percent, Syn::Percent),
				)),
				trivia_0plus(),
			))
			.map(coalesce_vec);

			let bin11 = bin12
				.clone()
				.foldl(op11.then(bin12).repeated(), |lhs, (op, rhs)| {
					let mut elems = op;
					elems.insert(0, lhs.into());
					elems.push(rhs.into());
					GreenNode::new(Syn::BinExpr.into(), elems)
				})
				.boxed();

			let op10 = primitive::group((
				trivia_0plus(),
				primitive::choice((
					comb::just_ts(Token::Plus, Syn::Plus),
					comb::just_ts(Token::Minus, Syn::Minus),
				)),
				trivia_0plus(),
			))
			.map(coalesce_vec);

			let bin10 = bin11
				.clone()
				.foldl(op10.then(bin11).repeated(), |lhs, (op, rhs)| {
					let mut elems = op;
					elems.insert(0, lhs.into());
					elems.push(rhs.into());
					GreenNode::new(Syn::BinExpr.into(), elems)
				});

			let op9 = primitive::group((
				trivia_0plus(),
				primitive::choice((
					comb::just_ts(Token::AngleL2, Syn::AngleL2),
					comb::just_ts(Token::AngleR2, Syn::AngleR2),
					comb::just_ts(Token::AngleR3, Syn::AngleR3),
				)),
				trivia_0plus(),
			))
			.map(coalesce_vec);

			let bin9 = bin10
				.clone()
				.foldl(op9.then(bin10).repeated(), |lhs, (op, rhs)| {
					let mut elems = op;
					elems.insert(0, lhs.into());
					elems.push(rhs.into());
					GreenNode::new(Syn::BinExpr.into(), elems)
				})
				.boxed();

			let op8 = primitive::group((
				trivia_0plus(),
				comb::just_ts(Token::Ampersand, Syn::Ampersand),
				trivia_0plus(),
			))
			.map(coalesce_vec);

			let bin8 = bin9
				.clone()
				.foldl(op8.then(bin9).repeated(), |lhs, (op, rhs)| {
					let mut elems = op;
					elems.insert(0, lhs.into());
					elems.push(rhs.into());
					GreenNode::new(Syn::BinExpr.into(), elems)
				});

			let op7 = primitive::group((
				trivia_0plus(),
				comb::just_ts(Token::Caret, Syn::Caret),
				trivia_0plus(),
			))
			.map(coalesce_vec);

			let bin7 = bin8
				.clone()
				.foldl(op7.then(bin8).repeated(), |lhs, (op, rhs)| {
					let mut elems = op;
					elems.insert(0, lhs.into());
					elems.push(rhs.into());
					GreenNode::new(Syn::BinExpr.into(), elems)
				})
				.boxed();

			let op6 = primitive::group((
				trivia_0plus(),
				comb::just_ts(Token::Pipe, Syn::Pipe),
				trivia_0plus(),
			))
			.map(coalesce_vec);

			let bin6 = bin7
				.clone()
				.foldl(op6.then(bin7).repeated(), |lhs, (op, rhs)| {
					let mut elems = op;
					elems.insert(0, lhs.into());
					elems.push(rhs.into());
					GreenNode::new(Syn::BinExpr.into(), elems)
				});

			let op5 = primitive::group((
				trivia_0plus(),
				primitive::choice((
					comb::just_ts(Token::AngleL, Syn::AngleL),
					comb::just_ts(Token::AngleR, Syn::AngleR),
					comb::just_ts(Token::AngleLEq, Syn::AngleLEq),
					comb::just_ts(Token::AngleREq, Syn::AngleREq),
				)),
				trivia_0plus(),
			))
			.map(coalesce_vec);

			let bin5 = bin6
				.clone()
				.foldl(op5.then(bin6).repeated(), |lhs, (op, rhs)| {
					let mut elems = op;
					elems.insert(0, lhs.into());
					elems.push(rhs.into());
					GreenNode::new(Syn::BinExpr.into(), elems)
				});

			let op4 = primitive::group((
				trivia_0plus(),
				primitive::choice((
					comb::just_ts(Token::Eq2, Syn::Eq2),
					comb::just_ts(Token::BangEq, Syn::BangEq),
				)),
				trivia_0plus(),
			))
			.map(coalesce_vec);

			let bin4 = bin5
				.clone()
				.foldl(op4.then(bin5).repeated(), |lhs, (op, rhs)| {
					let mut elems = op;
					elems.insert(0, lhs.into());
					elems.push(rhs.into());
					GreenNode::new(Syn::BinExpr.into(), elems)
				})
				.boxed();

			let op3 = primitive::group((
				trivia_0plus(),
				comb::just_ts(Token::Ampersand2, Syn::Ampersand2),
				trivia_0plus(),
			))
			.map(coalesce_vec);

			let bin3 = bin4
				.clone()
				.foldl(op3.then(bin4).repeated(), |lhs, (op, rhs)| {
					let mut elems = op;
					elems.insert(0, lhs.into());
					elems.push(rhs.into());
					GreenNode::new(Syn::BinExpr.into(), elems)
				});

			let op2 = primitive::group((
				trivia_0plus(),
				comb::just_ts(Token::Pipe2, Syn::Pipe2),
				trivia_0plus(),
			))
			.map(coalesce_vec);

			let bin2 = bin3
				.clone()
				.foldl(op2.then(bin3).repeated(), |lhs, (op, rhs)| {
					let mut elems = op;
					elems.insert(0, lhs.into());
					elems.push(rhs.into());
					GreenNode::new(Syn::BinExpr.into(), elems)
				})
				.boxed();

			// TODO: Ternary, assignment, casting

			bin2
		},
	)
}

/// The returned parser emits a series of expression nodes (comma-separated).
/// The return value of [`expr`] must be passed in to prevent infinite recursion.
pub fn expr_list<'i>(expr: parser_t!(GreenNode)) -> parser_t!(Vec<GreenElement>) {
	primitive::group((
		expr.clone(),
		primitive::group((
			trivia_0plus(),
			comb::just_ts(Token::Comma, Syn::Comma),
			trivia_0plus(),
			expr,
		))
		.repeated()
		.collect::<Vec<_>>(),
	))
	.map(|group| {
		let mut ret = vec![GreenElement::from(group.0)];

		for (mut triv0, comma, mut triv1, e) in group.1 {
			ret.append(&mut triv0);
			ret.push(comma.into());
			ret.append(&mut triv1);
			ret.push(e.into());
		}

		ret
	})
}

#[cfg(test)]
#[cfg(any())]
mod test {
	use crate::{
		testing::*,
		zdoom::{self, decorate::ParseTree},
	};

	use super::*;

	#[test]
	fn smoke() {
		const SOURCE: &str = "x ^ ((a * b) + (c / d)) | y & z && foo";

		let tbuf = crate::_scan(SOURCE, zdoom::Version::V1_0_0);
		let result = crate::_parse(expr(), SOURCE, &tbuf);
		let ptree: ParseTree = unwrap_parse_tree(result);

		assert_no_errors(&ptree);
	}

	#[test]
	fn smoke_call() {
		const SOURCE: &str = "nobody(told + me - about * decorate)";

		let tbuf = crate::_scan(SOURCE, zdoom::Version::V1_0_0);
		let result = crate::_parse(expr(), SOURCE, &tbuf);
		let ptree: ParseTree = unwrap_parse_tree(result);

		assert_no_errors(&ptree);
	}

	#[test]
	fn smoke_call_with_rng() {
		const SOURCE: &str = "set_random_seed[rngtbl](1234567890)";

		let tbuf = crate::_scan(SOURCE, zdoom::Version::V1_0_0);
		let result = crate::_parse(expr(), SOURCE, &tbuf);
		let ptree: ParseTree = unwrap_parse_tree(result);

		assert_no_errors(&ptree);
	}
}
