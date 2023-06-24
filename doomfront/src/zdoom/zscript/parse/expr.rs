use crate::{
	parser::{CloseMark, Parser},
	zdoom::{zscript::Syn, Token},
};

use super::common::*;

/// Builds a node tagged with one of the following:
/// - [`Syn::BinExpr`]
/// - [`Syn::CallExpr`]
/// - [`Syn::ClassCastExpr`]
/// - [`Syn::GroupExpr`]
/// - [`Syn::IdentExpr`]
/// - [`Syn::IndexExpr`]
/// - [`Syn::PostfixExpr`]
/// - [`Syn::PrefixExpr`]
/// - [`Syn::SuperExpr`]
/// - [`Syn::TernaryExpr`]
/// - [`Syn::VectorExpr`]
pub fn expr(p: &mut Parser<Syn>) {
	recur(p, Token::Eof);
}

fn recur(p: &mut Parser<Syn>, left: Token) {
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

fn primary_expr(p: &mut Parser<Syn>) -> CloseMark {
	let ex = p.open();

	if eat_ident_lax(p) {
		return p.close(ex, Syn::IdentExpr);
	}

	match p.nth(0) {
		Token::KwSuper => {
			p.advance(Syn::KwSuper);
			p.close(ex, Syn::SuperExpr)
		}
		Token::KwDefault => {
			p.advance(Syn::Ident);
			p.close(ex, Syn::IdentExpr)
		}
		Token::IntLit => {
			p.advance(Syn::IntLit);
			p.close(ex, Syn::Literal)
		}
		Token::FloatLit => {
			p.advance(Syn::FloatLit);
			p.close(ex, Syn::Literal)
		}
		Token::KwTrue => {
			p.advance(Syn::KwTrue);
			p.close(ex, Syn::Literal)
		}
		Token::KwFalse => {
			p.advance(Syn::KwFalse);
			p.close(ex, Syn::Literal)
		}
		Token::StringLit => {
			p.advance(Syn::StringLit);

			while p.find(0, |token| !token.is_trivia()) == Token::StringLit {
				trivia_0plus(p);
				p.advance(Syn::StringLit);
			}

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
				"a name literal",
				"`true`",
				"`false`",
				"`null`",
				"`super` or `default`",
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

/// Builds a [`Syn::ArgList`] node. Includes delimiting parentheses.
pub fn arg_list(p: &mut Parser<Syn>) {
	p.debug_assert_at(Token::ParenL);
	let arglist = p.open();
	p.expect(Token::ParenL, Syn::ParenL, &["`(`"]);
	trivia_0plus(p);

	while !p.at(Token::ParenR) && !p.eof() {
		let arg = p.open();

		if p.at_if(is_ident_lax) {
			let peeked = p.find(0, |token| !token.is_trivia() && !is_ident_lax(token));

			if peeked == Token::Colon {
				p.advance(Syn::Ident);
				trivia_0plus(p);
				p.advance(Syn::Colon);
				trivia_0plus(p);
			}
		}

		expr(p);

		p.close(arg, Syn::Argument);

		if p.find(0, |token| !token.is_trivia()) == Token::Comma {
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
pub fn expr_list(p: &mut Parser<Syn>) {
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
	fn smoke_1() {
		const SOURCE: &str = "(a[1]() + --b.c) * ++d && (e << f) ~== ((((g /= h ? i : j))))";

		let ptree: ParseTree = crate::parse(SOURCE, expr, zdoom::lex::Context::ZSCRIPT_LATEST);
		assert_no_errors(&ptree);
		prettyprint_maybe(ptree.cursor());
	}

	#[test]
	fn smoke_2() {
		const SOURCE: &str = r#"!multiplayer && (GetPlayerInput(INPUT_BUTTONS))"#;

		let ptree: ParseTree = crate::parse(SOURCE, expr, zdoom::lex::Context::ZSCRIPT_LATEST);
		assert_no_errors(&ptree);
		prettyprint_maybe(ptree.cursor());
	}

	#[test]
	fn smoke_vector_bin() {
		const SOURCE: &str = "(1.0, 2.0, 3.0) + (4.0, 5.0) - (6.0, 7.0, 8.0)";

		let ptree: ParseTree = crate::parse(SOURCE, expr, zdoom::lex::Context::ZSCRIPT_LATEST);
		assert_no_errors(&ptree);
		prettyprint_maybe(ptree.cursor());
	}

	#[test]
	fn smoke_string_lit_concat() {
		const SOURCE: &str = r#"n + "interstellar" "domine""nuclear waste processing facility""#;

		let ptree: ParseTree = crate::parse(SOURCE, expr, zdoom::lex::Context::ZSCRIPT_LATEST);
		assert_no_errors(&ptree);
		prettyprint_maybe(ptree.cursor());
	}
}
