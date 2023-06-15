use chumsky::{primitive, recursive::Recursive, IterParser, Parser};
use rowan::GreenNode;

use crate::{
	comb, parser_t,
	parsing::*,
	zdoom::{zscript::Syn, Token},
	GreenElement,
};

use super::ParserBuilder;

impl ParserBuilder {
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
	pub fn expr<'i>(&self) -> parser_t!(GreenNode) {
		chumsky::recursive::recursive(
			|expr: Recursive<dyn chumsky::Parser<'_, _, GreenNode, _>>| {
				let ident = comb::just_ts(Token::Ident, Syn::Ident)
					.map(|gtok| GreenNode::new(Syn::IdentExpr.into(), [gtok.into()]));

				let literal = primitive::choice((
					comb::just_ts(Token::StringLit, Syn::StringLit),
					comb::just_ts(Token::IntLit, Syn::IntLit),
					comb::just_ts(Token::FloatLit, Syn::FloatLit),
					comb::just_ts(Token::NameLit, Syn::NameLit),
					comb::just_ts(Token::KwTrue, Syn::TrueLit),
					comb::just_ts(Token::KwFalse, Syn::FalseLit),
					comb::just_ts(Token::KwNull, Syn::NullLit),
				))
				.map_with_state(|gtok, _, _| GreenNode::new(Syn::Literal.into(), [gtok.into()]));

				let vector = primitive::group((
					comb::just_ts(Token::ParenL, Syn::ParenL),
					self.trivia_0plus(),
					expr.clone(),
					primitive::group((
						self.trivia_0plus(),
						comb::just_ts(Token::Comma, Syn::Comma),
						self.trivia_0plus(),
						expr.clone(),
					))
					.repeated()
					.at_least(1)
					.at_most(3)
					.collect::<Vec<_>>(),
					self.trivia_0plus(),
					comb::just_ts(Token::ParenR, Syn::ParenR),
				))
				.map(|group| coalesce_node(group, Syn::VectorExpr));

				let grouped = primitive::group((
					comb::just_ts(Token::ParenL, Syn::ParenL),
					self.trivia_0plus(),
					expr.clone(),
					self.trivia_0plus(),
					comb::just_ts(Token::ParenR, Syn::ParenR),
				))
				.map(|group| coalesce_node(group, Syn::GroupExpr));

				let atom = primitive::choice((vector, grouped, literal, ident.clone()));

				let subscript = primitive::group((
					comb::just_ts(Token::BracketL, Syn::BracketL),
					self.trivia_0plus(),
					expr.clone(),
					self.trivia_0plus(),
					comb::just_ts(Token::BracketR, Syn::BracketR),
				))
				.map(|group| coalesce_node(group, Syn::Subscript));

				let index = atom.foldl(subscript.repeated(), |lhs, subscript| {
					GreenNode::new(Syn::IndexExpr.into(), [lhs.into(), subscript.into()])
				});

				let arg_list = primitive::group((
					comb::just_ts(Token::ParenL, Syn::ParenL),
					self.arg_list(expr.clone()),
					comb::just_ts(Token::ParenR, Syn::ParenR),
				))
				.map(|group| coalesce_node(group, Syn::ArgList));

				let call = index.clone().foldl(arg_list.repeated(), |lhs, arg_list| {
					GreenNode::new(Syn::CallExpr.into(), [lhs.into(), arg_list.into()])
				});

				let op14 = primitive::group((
					self.trivia_0plus(),
					comb::just_ts(Token::Dot, Syn::Dot),
					self.trivia_0plus(),
				))
				.map(coalesce_vec);

				// TODO: Unary

				let bin14 = call
					.clone()
					.foldl(op14.then(ident).repeated(), |lhs, (op, rhs)| {
						let mut elems = op;
						elems.insert(0, lhs.into());
						elems.push(rhs.into());
						GreenNode::new(Syn::BinExpr.into(), elems)
					})
					.boxed();

				let op13 = primitive::group((
					self.trivia_0plus(),
					comb::just_ts(Token::Asterisk2, Syn::Asterisk2),
					self.trivia_0plus(),
				))
				.map(coalesce_vec);

				let bin13 = bin14
					.clone()
					.foldl(op13.then(bin14).repeated(), |lhs, (op, rhs)| {
						let mut elems = op;
						elems.insert(0, lhs.into());
						elems.push(rhs.into());
						GreenNode::new(Syn::BinExpr.into(), elems)
					});

				let op12 = primitive::group((
					self.trivia_0plus(),
					primitive::choice((
						comb::just_ts(Token::Asterisk, Syn::Asterisk),
						comb::just_ts(Token::Slash, Syn::Slash),
						comb::just_ts(Token::Percent, Syn::Percent),
						comb::just_ts(Token::KwCross, Syn::KwCross),
						comb::just_ts(Token::KwDot, Syn::KwDot),
					)),
					self.trivia_0plus(),
				))
				.map(coalesce_vec);

				let bin12 = bin13
					.clone()
					.foldl(op12.then(bin13).repeated(), |lhs, (op, rhs)| {
						let mut elems = op;
						elems.insert(0, lhs.into());
						elems.push(rhs.into());
						GreenNode::new(Syn::BinExpr.into(), elems)
					})
					.boxed();

				let op11 = primitive::group((
					self.trivia_0plus(),
					primitive::choice((
						comb::just_ts(Token::Plus, Syn::Plus),
						comb::just_ts(Token::Minus, Syn::Minus),
					)),
					self.trivia_0plus(),
				))
				.map(coalesce_vec);

				let bin11 = bin12
					.clone()
					.foldl(op11.then(bin12).repeated(), |lhs, (op, rhs)| {
						let mut elems = op;
						elems.insert(0, lhs.into());
						elems.push(rhs.into());
						GreenNode::new(Syn::BinExpr.into(), elems)
					});

				let op10 = primitive::group((
					self.trivia_0plus(),
					primitive::choice((
						comb::just_ts(Token::AngleL2, Syn::AngleL2),
						comb::just_ts(Token::AngleR2, Syn::AngleR2),
						comb::just_ts(Token::AngleR3, Syn::AngleR3),
					)),
					self.trivia_0plus(),
				))
				.map(coalesce_vec);

				let bin10 = bin11
					.clone()
					.foldl(op10.then(bin11).repeated(), |lhs, (op, rhs)| {
						let mut elems = op;
						elems.insert(0, lhs.into());
						elems.push(rhs.into());
						GreenNode::new(Syn::BinExpr.into(), elems)
					})
					.boxed();

				let op9 = primitive::group((
					self.trivia_0plus(),
					comb::just_ts(Token::Ampersand, Syn::Ampersand),
					self.trivia_0plus(),
				))
				.map(coalesce_vec);

				let bin9 = bin10
					.clone()
					.foldl(op9.then(bin10).repeated(), |lhs, (op, rhs)| {
						let mut elems = op;
						elems.insert(0, lhs.into());
						elems.push(rhs.into());
						GreenNode::new(Syn::BinExpr.into(), elems)
					});

				let op8 = primitive::group((
					self.trivia_0plus(),
					comb::just_ts(Token::Caret, Syn::Caret),
					self.trivia_0plus(),
				))
				.map(coalesce_vec);

				let bin8 = bin9
					.clone()
					.foldl(op8.then(bin9).repeated(), |lhs, (op, rhs)| {
						let mut elems = op;
						elems.insert(0, lhs.into());
						elems.push(rhs.into());
						GreenNode::new(Syn::BinExpr.into(), elems)
					})
					.boxed();

				let op7 = primitive::group((
					self.trivia_0plus(),
					comb::just_ts(Token::Pipe, Syn::Pipe),
					self.trivia_0plus(),
				))
				.map(coalesce_vec);

				let bin7 = bin8
					.clone()
					.foldl(op7.then(bin8).repeated(), |lhs, (op, rhs)| {
						let mut elems = op;
						elems.insert(0, lhs.into());
						elems.push(rhs.into());
						GreenNode::new(Syn::BinExpr.into(), elems)
					});

				let op6 = primitive::group((
					self.trivia_0plus(),
					comb::just_ts(Token::Dot2, Syn::Dot2),
					self.trivia_0plus(),
				))
				.map(coalesce_vec);

				let bin6 = bin7
					.clone()
					.foldl(op6.then(bin7).repeated(), |lhs, (op, rhs)| {
						let mut elems = op;
						elems.insert(0, lhs.into());
						elems.push(rhs.into());
						GreenNode::new(Syn::BinExpr.into(), elems)
					})
					.boxed();

				let op5 = primitive::group((
					self.trivia_0plus(),
					primitive::choice((
						comb::just_ts(Token::AngleL, Syn::AngleL),
						comb::just_ts(Token::AngleR, Syn::AngleR),
						comb::just_ts(Token::AngleLEq, Syn::AngleLEq),
						comb::just_ts(Token::AngleREq, Syn::AngleREq),
						comb::just_ts(Token::AngleLAngleREq, Syn::AngleLAngleREq),
						comb::just_ts(Token::KwIs, Syn::KwIs),
					)),
					self.trivia_0plus(),
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
					self.trivia_0plus(),
					primitive::choice((
						comb::just_ts(Token::Eq2, Syn::Eq2),
						comb::just_ts(Token::BangEq, Syn::BangEq),
						comb::just_ts(Token::TildeEq2, Syn::TildeEq2),
					)),
					self.trivia_0plus(),
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
					self.trivia_0plus(),
					comb::just_ts(Token::Ampersand2, Syn::Ampersand2),
					self.trivia_0plus(),
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
					self.trivia_0plus(),
					comb::just_ts(Token::Pipe2, Syn::Pipe2),
					self.trivia_0plus(),
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
	/// The return value of [`Self::expr`] must be passed in to prevent infinite recursion.
	pub(super) fn expr_list<'i>(&self, expr: parser_t!(GreenNode)) -> parser_t!(Vec<GreenElement>) {
		primitive::group((
			expr.clone(),
			primitive::group((
				self.trivia_0plus(),
				comb::just_ts(Token::Comma, Syn::Comma),
				self.trivia_0plus(),
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

	/// The returned parser emits 0 or more expression nodes (comma-separated),
	/// each with a possible preceding identifier and colon.
	/// Note that this does not include enclosing parentheses.
	/// The return value of [`Self::expr`] must be passed in to prevent infinite recursion.
	pub(super) fn arg_list<'i>(&self, expr: parser_t!(GreenNode)) -> parser_t!(Vec<GreenElement>) {
		let named_expr = primitive::group((
			self.ident(),
			self.trivia_0plus(),
			comb::just_ts(Token::Colon, Syn::Colon),
			self.trivia_0plus(),
			expr,
		))
		.map(coalesce_vec);

		let rep = primitive::group((
			self.trivia_0plus(),
			comb::just_ts(Token::Comma, Syn::Comma),
			self.trivia_0plus(),
			named_expr.clone(),
		));

		named_expr.foldl(rep.repeated(), |mut lhs, (mut t0, comma, mut t1, mut e)| {
			lhs.append(&mut t0);
			lhs.push(comma.into());
			lhs.append(&mut t1);
			lhs.append(&mut e);
			lhs
		})
	}
}

#[cfg(test)]
mod test {
	use crate::{
		testing::*,
		zdoom::{zscript::ParseTree, Version},
	};

	use super::*;

	#[test]
	fn smoke() {
		const SOURCE: &str = "(a[1]() + b.c) * d && (e << f) ~== ((((g >>> h))))";

		let parser = ParserBuilder::new(Version::default()).expr();
		let tbuf = crate::scan(SOURCE, Version::default());
		let result = crate::parse(parser, SOURCE, &tbuf);
		let ptree: ParseTree = unwrap_parse_tree(result);
		assert_no_errors(&ptree);
		prettyprint(ptree.cursor());
	}
}
