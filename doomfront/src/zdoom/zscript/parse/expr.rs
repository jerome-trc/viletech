use chumsky::{primitive, recursive::Recursive, IterParser, Parser};
use rowan::GreenNode;

use crate::{
	comb,
	parser::CloseMark,
	parser_t,
	parsing::*,
	zdoom::{zscript::Syn, Token},
	GreenElement,
};

use super::ParserBuilder;

use super::common::*;

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
				let ident = self
					.ident()
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

	/// The returned parser emits 0 or more [`Syn::Argument`] nodes
	/// (comma-separated), each with a possible preceding identifier and colon.
	/// Note that this does not include enclosing parentheses.
	/// The return value of [`Self::expr`] must be passed in to prevent infinite recursion.
	pub(super) fn arg_list<'i>(&self, expr: parser_t!(GreenNode)) -> parser_t!(Vec<GreenElement>) {
		let named_expr = primitive::group((
			primitive::group((
				self.ident(),
				self.trivia_0plus(),
				comb::just_ts(Token::Colon, Syn::Colon),
				self.trivia_0plus(),
			))
			.or_not(),
			expr,
		))
		.map(|group| coalesce_node(group, Syn::Argument));

		let rep = primitive::group((
			self.trivia_0plus(),
			comb::just_ts(Token::Comma, Syn::Comma),
			self.trivia_0plus(),
			named_expr.clone(),
		));

		primitive::group((named_expr.or_not(), rep.repeated().collect::<Vec<_>>()))
			.map(coalesce_vec)
	}
}

pub fn expr(p: &mut crate::parser::Parser<Syn>) {
	recur(p, Token::Eof);
}

fn recur(p: &mut crate::parser::Parser<Syn>, left: Token) {
	let mut lhs = primary_expr(p);

	loop {
		trivia_0plus(p);

		let right = p.nth(0);

		match right {
			Token::Minus2 => {
				let m = p.open_before(lhs);
				p.advance(Syn::Minus2);
				lhs = p.close(m, Syn::PostfixExpr);
				continue;
			}
			Token::Plus2 => {
				let m = p.open_before(lhs);
				p.advance(Syn::Plus2);
				lhs = p.close(m, Syn::PostfixExpr);
				continue;
			}
			Token::ParenL => {
				let m = p.open_before(lhs);
				trivia_0plus(p);
				arg_list(p);
				trivia_0plus(p);
				lhs = p.close(m, Syn::CallExpr);
				continue;
			}
			Token::BracketL => {
				let m = p.open_before(lhs);
				p.expect(Token::BracketL, Syn::BracketL, &["`[`"]);
				trivia_0plus(p);
				expr(p);
				trivia_0plus(p);
				p.expect(Token::BracketR, Syn::BracketR, &["`]`"]);
				lhs = p.close(m, Syn::IndexExpr);
				continue;
			}
			_ => {}
		}

		if infix_right_stronger(left, right) {
			if right == Token::Question {
				let m = p.open_before(lhs);
				p.advance(Syn::Question);
				trivia_0plus(p);
				expr(p);
				trivia_0plus(p);
				p.expect(Token::Colon, Syn::Colon, &["`:`"]);
				trivia_0plus(p);
				expr(p);
				lhs = p.close(m, Syn::TernaryExpr);
			} else {
				let m = p.open_before(lhs);
				p.advance(Syn::from(right));
				trivia_0plus(p);
				recur(p, right);
				lhs = p.close(m, Syn::BinExpr);
			}
		} else {
			break;
		}
	}
}

fn primary_expr(p: &mut crate::parser::Parser<Syn>) -> CloseMark {
	let ex = p.open();

	if eat_ident(p) {
		return p.close(ex, Syn::IdentExpr);
	}

	match p.nth(0) {
		Token::IntLit => {
			p.advance(Syn::IntLit);
			p.close(ex, Syn::Literal)
		}
		Token::FloatLit => {
			p.advance(Syn::FloatLit);
			p.close(ex, Syn::Literal)
		}
		Token::KwTrue => {
			p.advance(Syn::TrueLit);
			p.close(ex, Syn::Literal)
		}
		Token::KwFalse => {
			p.advance(Syn::FalseLit);
			p.close(ex, Syn::Literal)
		}
		Token::StringLit => {
			p.advance(Syn::StringLit);
			p.close(ex, Syn::Literal)
		}
		Token::NameLit => {
			p.advance(Syn::NameLit);
			p.close(ex, Syn::Literal)
		}
		Token::KwNull => {
			p.advance(Syn::NullLit);
			p.close(ex, Syn::Literal)
		}
		Token::ParenL => {
			p.expect(Token::ParenL, Syn::ParenL, &["`(`"]);
			trivia_0plus(p);

			if p.eat(Token::KwClass, Syn::KwClass) {
				// Class cast
				trivia_0plus(p);
				p.expect(Token::AngleL, Syn::AngleL, &["`<`"]);
				trivia_0plus(p);
				ident(p);
				trivia_0plus(p);
				p.expect(Token::AngleR, Syn::AngleR, &["`>`"]);
				trivia_0plus(p);
				p.expect(Token::ParenR, Syn::ParenR, &["`)`"]);
				trivia_0plus(p);
				arg_list(p);
				return p.close(ex, Syn::ClassCastExpr);
			}

			expr(p);
			trivia_0plus(p);

			if p.eat(Token::ParenR, Syn::ParenR) {
				p.close(ex, Syn::GroupExpr)
			} else if p.eat(Token::Comma, Syn::Comma) {
				// Vector
				for _ in 0..3 {
					trivia_0plus(p);
					expr(p);
					trivia_0plus(p);

					if !p.eat(Token::Comma, Syn::Comma) {
						p.expect(Token::ParenR, Syn::ParenR, &["`)`"]);
						break;
					}
				}

				p.close(ex, Syn::VectorExpr)
			} else {
				p.advance_err_and_close(ex, Syn::from(p.nth(0)), Syn::Error, &["`)`", "`,`"])
			}
		}
		Token::Bang => {
			p.advance(Syn::Bang);
			recur(p, Token::Bang);
			p.close(ex, Syn::PrefixExpr)
		}
		Token::Minus2 => {
			p.advance(Syn::Minus2);
			recur(p, Token::Minus2);
			p.close(ex, Syn::PrefixExpr)
		}
		Token::Plus2 => {
			p.advance(Syn::Plus2);
			recur(p, Token::Plus2);
			p.close(ex, Syn::PrefixExpr)
		}
		Token::Minus => {
			p.advance(Syn::Minus);
			recur(p, Token::Minus);
			p.close(ex, Syn::PrefixExpr)
		}
		Token::Plus => {
			p.advance(Syn::Plus);
			recur(p, Token::Plus);
			p.close(ex, Syn::PrefixExpr)
		}
		Token::Tilde => {
			p.advance(Syn::Tilde);
			recur(p, Token::Tilde);
			p.close(ex, Syn::PrefixExpr)
		}
		_ => p.advance_err_and_close(
			ex,
			Syn::Unknown,
			Syn::Error,
			&[
				"an integer",
				"a floating-point number",
				"a string",
				"a name",
				"`true`",
				"`false`",
				"`null`",
				"`(`",
				"`!`",
				"`--`",
				"`++`",
				"`-`",
				"`+`",
				"`~`",
			],
		),
	}
}

/// i.e. can `token` begin a primary expression?
#[must_use]
pub(super) fn in_first_set(token: Token) -> bool {
	if is_ident(token) {
		return true;
	}

	matches!(
		token,
		Token::IntLit
			| Token::FloatLit
			| Token::KwTrue
			| Token::KwFalse
			| Token::StringLit
			| Token::NameLit
			| Token::KwNull
			| Token::ParenL
			| Token::Bang
			| Token::Minus2
			| Token::Plus2
			| Token::Minus
			| Token::Plus
			| Token::Tilde,
	)
}

/// Builds a [`Syn::ArgList`] node. Includes delimiting parentheses.
///
pub fn arg_list(p: &mut crate::parser::Parser<Syn>) {
	debug_assert!(p.at(Token::ParenL));
	let arglist = p.open();
	p.expect(Token::ParenL, Syn::ParenL, &["`(`"]);
	trivia_0plus(p);

	while !p.at(Token::ParenR) && !p.eof() {
		let arg = p.open();

		if p.at_if(is_ident_lax) {
			let peeked = p.next_filtered(|token| !token.is_trivia() && !is_ident_lax(token));

			if peeked == Token::Colon {
				p.advance(Syn::Ident);
				trivia_0plus(p);
				p.advance(Syn::Colon);
				trivia_0plus(p);
			}
		}

		expr(p);

		p.close(arg, Syn::Argument);

		if p.next_filtered(|token| !token.is_trivia()) == Token::Comma {
			trivia_0plus(p);
			p.advance(Syn::Comma);
			trivia_0plus(p);
		} else {
			break;
		}
	}

	trivia_0plus(p);
	p.expect(Token::ParenR, Syn::ParenR, &["`)`"]);
	p.close(arglist, Syn::ArgList);
}

#[must_use]
fn infix_right_stronger(left: Token, right: Token) -> bool {
	#[must_use]
	fn strength(token: Token) -> Option<usize> {
		const PREC_TABLE: &[&[Token]] = &[
			&[
				Token::Eq,
				Token::AsteriskEq,
				Token::SlashEq,
				Token::PercentEq,
				Token::PlusEq,
				Token::MinusEq,
				Token::AngleL2Eq,
				Token::AngleR2Eq,
				Token::AmpersandEq,
				Token::PipeEq,
				Token::CaretEq,
				Token::AngleR3Eq,
			],
			&[Token::Question],
			&[Token::Pipe2],
			&[Token::Ampersand2],
			&[Token::Eq2, Token::BangEq, Token::TildeEq2],
			&[
				Token::AngleL,
				Token::AngleR,
				Token::AngleLEq,
				Token::AngleREq,
				Token::AngleLAngleREq,
				Token::KwIs,
			],
			&[Token::Dot2],
			&[Token::Pipe],
			&[Token::Caret],
			&[Token::Ampersand],
			&[Token::AngleL2, Token::AngleR2, Token::AngleR3],
			&[Token::Plus, Token::Minus],
			&[
				Token::Asterisk,
				Token::Slash,
				Token::Percent,
				Token::KwCross,
				Token::KwDot,
			],
			&[Token::Asterisk2],
			&[Token::Minus2, Token::Plus2],
			&[Token::Dot],
		];

		PREC_TABLE.iter().position(|level| level.contains(&token))
	}

	let Some(right_s) = strength(right) else {
		return false;
	};

	let Some(left_s) = strength(left) else {
		debug_assert_eq!(left, Token::Eof);
		return true;
	};

	right_s > left_s
}

/// Expects the current position to be the start of at least one expression.
pub fn expr_list(p: &mut crate::parser::Parser<Syn>) {
	expr(p);
	trivia_0plus(p);

	while !p.eof() {
		if !p.eat(Token::Comma, Syn::Comma) {
			break;
		}

		trivia_0plus(p);
		expr(p);
		trivia_0plus(p);
	}
}

#[cfg(test)]
mod test {
	use crate::{
		testing::*,
		zdoom::{self, zscript::ParseTree},
	};

	use super::*;

	#[test]
	fn smoke() {
		const SOURCE: &str = "(a[1]() + --b.c) * ++d && (e << f) ~== ((((g /= h ? i : j))))";

		let ptree: ParseTree = crate::parse(SOURCE, expr, zdoom::Version::default());
		assert_no_errors(&ptree);
	}

	#[test]
	fn smoke_vector_bin() {
		const SOURCE: &str = "(1.0, 2.0, 3.0) + (4.0, 5.0) - (6.0, 7.0, 8.0)";

		let ptree: ParseTree = crate::parse(SOURCE, expr, zdoom::Version::default());
		assert_no_errors(&ptree);
	}
}
