use crate::{
	parser::{CloseMark, Parser},
	zdoom::{
		decorate::{parse::common::*, Syntax},
		Token,
	},
};

pub(super) fn expr(p: &mut Parser<Syntax>) {
	recur(p, Token::Eof);
}

fn recur(p: &mut Parser<Syntax>, left: Token) {
	let mut lhs = primary(p);

	loop {
		trivia_0plus(p);

		let right = p.nth(0);

		match right {
			Token::Minus2 => {
				let m = p.open_before(lhs);
				p.advance(Syntax::Minus2);
				lhs = p.close(m, Syntax::PostfixExpr);
				continue;
			}
			Token::Plus2 => {
				let m = p.open_before(lhs);
				p.advance(Syntax::Plus2);
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

fn primary(p: &mut Parser<Syntax>) -> CloseMark {
	let ex = p.open();

	let token = p.nth(0);

	if is_ident_lax(token) {
		p.advance(Syntax::Ident);
		return p.close(ex, Syntax::IdentExpr);
	}

	match token {
		t @ (Token::IntLit | Token::FloatLit) => {
			if matches!(p.nth(1), Token::Ident | Token::IntLit) {
				let s = p.nth_slice(1);

				if s.chars().all(|c| c.is_ascii_hexdigit()) {
					p.advance_n(Syntax::HexLit, 2);
					return p.close(ex, Syntax::ColorExpr);
				}
			}

			p.advance(Syntax::from(t));
			p.close(ex, Syntax::Literal)
		}
		t @ (Token::KwTrue | Token::KwFalse | Token::StringLit | Token::NameLit) => {
			p.advance(Syntax::from(t));
			p.close(ex, Syntax::Literal)
		}
		Token::KwNone => {
			p.advance(Syntax::KwNone);
			p.close(ex, Syntax::NoneExpr)
		}
		Token::ParenL => {
			p.expect(Token::ParenL, Syntax::ParenL, &[&["`(`"]]);
			trivia_0plus(p);
			expr(p);
			trivia_0plus(p);
			p.expect(Token::ParenR, Syntax::ParenR, &[&["`)`"]]);
			p.close(ex, Syntax::GroupExpr)
		}
		t @ (Token::Bang
		| Token::Minus2
		| Token::Plus2
		| Token::Minus
		| Token::Plus
		| Token::Tilde) => {
			p.advance(Syntax::from(t));
			trivia_0plus(p);
			recur(p, t);
			p.close(ex, Syntax::PrefixExpr)
		}
		other => p.advance_err_and_close(
			ex,
			Syntax::from(other),
			Syntax::Error,
			&[&[
				"an integer",
				"a floating-point number",
				"a string",
				"a name literal",
				"`true` or `false`",
				"`(`",
				"`!`",
				"`--` or `++`",
				"`-` or `+`",
				"`~`",
			]],
		),
	}
}

/// Builds a [`Syntax::ArgList`] node. Includes delimiting parentheses.
pub(super) fn arg_list(p: &mut Parser<Syntax>) {
	p.debug_assert_at(Token::ParenL);
	let arglist = p.open();
	p.advance(Syntax::ParenL);
	trivia_0plus(p);

	while !p.at(Token::ParenR) && !p.eof() {
		expr(p);

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
	&[Token::Eq2, Token::BangEq],
	&[
		Token::AngleL,
		Token::AngleR,
		Token::AngleLEq,
		Token::AngleREq,
	],
	&[Token::Pipe],
	&[Token::Caret],
	&[Token::Ampersand],
	&[Token::AngleL2, Token::AngleR2, Token::AngleR3],
	&[Token::Plus, Token::Minus],
	&[Token::Asterisk, Token::Slash, Token::Percent],
	&[Token::Asterisk2],
	&[Token::Minus2, Token::Plus2, Token::Bang, Token::Tilde],
];
