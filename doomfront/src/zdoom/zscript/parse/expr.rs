use crate::{
	parser::{CloseMark, Parser},
	zdoom::{zscript::Syntax, Token},
};

use super::common::*;

/// Builds a node tagged with one of the following:
/// - [`Syntax::BinExpr`]
/// - [`Syntax::CallExpr`]
/// - [`Syntax::ClassCastExpr`]
/// - [`Syntax::GroupExpr`]
/// - [`Syntax::IdentExpr`]
/// - [`Syntax::IndexExpr`]
/// - [`Syntax::PostfixExpr`]
/// - [`Syntax::PrefixExpr`]
/// - [`Syntax::SuperExpr`]
/// - [`Syntax::TernaryExpr`]
/// - [`Syntax::VectorExpr`]
pub fn expr(p: &mut Parser<Syntax>) {
	recur(p, Token::Eof);
}

fn recur(p: &mut Parser<Syntax>, left: Token) {
	let mut lhs = primary_expr(p);

	loop {
		trivia_0plus(p);

		let right = p.nth(0);

		match right {
			t @ (Token::Minus2 | Token::Plus2) => {
				let m = p.open_before(lhs);
				p.advance(Syntax::from(t));
				lhs = p.close(m, Syntax::PostfixExpr);
				continue;
			}
			Token::ParenL => {
				let m = p.open_before(lhs);
				trivia_0plus(p);
				arg_list(p);
				trivia_0plus(p);
				lhs = p.close(m, Syntax::CallExpr);
				continue;
			}
			Token::BracketL => {
				let m = p.open_before(lhs);
				p.expect(Token::BracketL, Syntax::BracketL, &[&["`[`"]]);
				trivia_0plus(p);
				expr(p);
				trivia_0plus(p);
				p.expect(Token::BracketR, Syntax::BracketR, &[&["`]`"]]);
				lhs = p.close(m, Syntax::IndexExpr);
				continue;
			}
			_ => {}
		}

		if crate::parser::pratt::<Syntax>(left, right, PRATT_PRECEDENCE) {
			match right {
				Token::Dot => {
					let m = p.open_before(lhs);
					p.advance(Syntax::Dot);
					trivia_0plus(p);
					ident::<{ ID_SFKW | ID_SQKW | ID_TYPES | ID_DEFAULT }>(p);
					lhs = p.close(m, Syntax::MemberExpr);
				}
				Token::Question => {
					let m = p.open_before(lhs);
					p.advance(Syntax::Question);
					trivia_0plus(p);
					expr(p);
					trivia_0plus(p);
					p.expect(Token::Colon, Syntax::Colon, &[&["`:`"]]);
					trivia_0plus(p);
					expr(p);
					lhs = p.close(m, Syntax::TernaryExpr);
				}
				_ => {
					let m = p.open_before(lhs);
					p.advance(Syntax::from(right));
					trivia_0plus(p);
					recur(p, right);
					lhs = p.close(m, Syntax::BinExpr);
				}
			}
		} else {
			break;
		}
	}
}

fn primary_expr(p: &mut Parser<Syntax>) -> CloseMark {
	let ex = p.open();

	let token = p.nth(0);

	if is_ident_lax(token) {
		p.advance(Syntax::Ident);
		return p.close(ex, Syntax::IdentExpr);
	}

	match token {
		Token::IntLit => {
			p.advance(Syntax::IntLit);
			p.close(ex, Syntax::Literal)
		}
		Token::FloatLit => {
			p.advance(Syntax::FloatLit);
			p.close(ex, Syntax::Literal)
		}
		Token::KwTrue => {
			p.advance(Syntax::KwTrue);
			p.close(ex, Syntax::Literal)
		}
		Token::KwFalse => {
			p.advance(Syntax::KwFalse);
			p.close(ex, Syntax::Literal)
		}
		Token::StringLit => {
			p.advance(Syntax::StringLit);

			while p.find(0, |token| !token.is_trivia()) == Token::StringLit {
				trivia_0plus(p);
				p.advance(Syntax::StringLit);
			}

			p.close(ex, Syntax::Literal)
		}
		Token::NameLit => {
			p.advance(Syntax::NameLit);
			p.close(ex, Syntax::Literal)
		}
		Token::KwNull => {
			p.advance(Syntax::NullLit);
			p.close(ex, Syntax::Literal)
		}
		Token::KwSuper => {
			p.advance(Syntax::KwSuper);
			p.close(ex, Syntax::SuperExpr)
		}
		Token::KwDefault => {
			p.advance(Syntax::Ident);
			p.close(ex, Syntax::IdentExpr)
		}
		Token::ParenL => {
			p.expect(Token::ParenL, Syntax::ParenL, &[&["`(`"]]);
			trivia_0plus(p);

			if p.eat(Token::KwClass, Syntax::KwClass) {
				// Class cast
				trivia_0plus(p);
				p.expect(Token::AngleL, Syntax::AngleL, &[&["`<`"]]);
				trivia_0plus(p);
				ident::<0>(p);
				trivia_0plus(p);
				p.expect(Token::AngleR, Syntax::AngleR, &[&["`>`"]]);
				trivia_0plus(p);
				p.expect(Token::ParenR, Syntax::ParenR, &[&["`)`"]]);
				trivia_0plus(p);
				arg_list(p);
				return p.close(ex, Syntax::ClassCastExpr);
			}

			expr(p);
			trivia_0plus(p);

			if p.eat(Token::ParenR, Syntax::ParenR) {
				p.close(ex, Syntax::GroupExpr)
			} else if p.eat(Token::Comma, Syntax::Comma) {
				// Vector
				for _ in 0..3 {
					trivia_0plus(p);
					expr(p);
					trivia_0plus(p);

					if !p.eat(Token::Comma, Syntax::Comma) {
						p.expect(Token::ParenR, Syntax::ParenR, &[&["`)`"]]);
						break;
					}
				}

				p.close(ex, Syntax::VectorExpr)
			} else {
				p.advance_err_and_close(
					ex,
					Syntax::from(p.nth(0)),
					Syntax::Error,
					&[&["`)`", "`,`"]],
				)
			}
		}
		t @ (Token::Bang
		| Token::Minus2
		| Token::Plus2
		| Token::Minus
		| Token::Plus
		| Token::Tilde
		| Token::KwSizeOf
		| Token::KwAlignOf) => {
			p.advance(Syntax::from(t));
			trivia_0plus(p);
			recur(p, t);
			p.close(ex, Syntax::PrefixExpr)
		}
		_ => p.advance_err_and_close(
			ex,
			Syntax::Unknown,
			Syntax::Error,
			&[&[
				"an integer",
				"a floating-point number",
				"a string",
				"a name literal",
				"`true` or `false`",
				"`null`",
				"`super` or `default`",
				"`sizeof` or `alignof`",
				"`(`",
				"`!`",
				"`--`",
				"`++`",
				"`-`",
				"`+`",
				"`~`",
			]],
		),
	}
}

/// i.e. can `token` begin a primary expression?
#[must_use]
pub(super) fn in_first_set(token: Token) -> bool {
	if is_ident_lax(token) {
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
			| Token::KwSuper
			| Token::Bang
			| Token::Minus2
			| Token::Plus2
			| Token::Minus
			| Token::Plus
			| Token::Tilde
			| Token::KwAlignOf
			| Token::KwSizeOf
	)
}

/// Builds a [`Syntax::ArgList`] node. Includes delimiting parentheses.
pub(super) fn arg_list(p: &mut Parser<Syntax>) {
	p.debug_assert_at(Token::ParenL);
	let arglist = p.open();
	p.advance(Syntax::ParenL);
	trivia_0plus(p);

	while !p.at(Token::ParenR) && !p.eof() {
		let arg = p.open();

		if p.at_if(is_ident_lax) {
			let peeked = p.find(0, |token| !token.is_trivia() && !is_ident_lax(token));

			if peeked == Token::Colon {
				p.advance(Syntax::Ident);
				trivia_0plus(p);
				p.advance(Syntax::Colon);
				trivia_0plus(p);
			}
		}

		expr(p);

		p.close(arg, Syntax::Argument);

		if p.find(0, |token| !token.is_trivia()) == Token::Comma {
			trivia_0plus(p);
			p.advance(Syntax::Comma);
			trivia_0plus(p);
		} else {
			break;
		}
	}

	trivia_0plus(p);
	p.expect(Token::ParenR, Syntax::ParenR, &[&["`)`"]]);
	p.close(arglist, Syntax::ArgList);
}

/// Expects the current position to be the start of at least one expression.
pub fn expr_list(p: &mut Parser<Syntax>) {
	expr(p);
	trivia_0plus(p);

	while !p.eof() {
		if !p.eat(Token::Comma, Syntax::Comma) {
			break;
		}

		trivia_0plus(p);
		expr(p);
		trivia_0plus(p);
	}
}

const PRATT_PRECEDENCE: &[&[Token]] = &[
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
	&[
		Token::Minus2,
		Token::Plus2,
		Token::Bang,
		Token::Tilde,
		Token::KwSizeOf,
		Token::KwAlignOf,
	],
	&[Token::Dot],
];
