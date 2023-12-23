use doomfront::parser::{CloseMark, Parser};

use crate::Syntax;

use super::common::*;

/// Returns `true` if the expression that was parsed ends with a block.
pub fn expr(p: &mut Parser<Syntax>) -> bool {
	recur(p, Syntax::Eof)
}

/// Returns `true` if the expression that was parsed ends with a block.
pub fn type_expr(p: &mut Parser<Syntax>) -> bool {
	let mark = p.open();
	let block_end;

	loop {
		trivia_0plus(p);

		match p.nth(0) {
			t @ Syntax::BracketL => {
				let pfx = p.open();
				p.advance(t);
				trivia_0plus(p);
				let _ = primary(p);
				trivia_0plus(p);
				p.expect(Syntax::BracketR, Syntax::BracketR, &[&["`]`"]]);
				p.close(pfx, Syntax::ArrayPrefix);
			}
			t @ Syntax::Asterisk => {
				let pfx = p.open();
				p.advance(t);
				p.close(pfx, Syntax::PointerPrefix);
			}
			_ => {
				block_end = primary(p).1;
				break;
			}
		}
	}

	p.close(mark, Syntax::ExprType);
	block_end
}

/// Returns `true` if the expression that was parsed ends with a block.
fn recur(p: &mut Parser<Syntax>, left: Syntax) -> bool {
	let (mut lhs, mut block_end) = primary(p);

	loop {
		trivia_0plus(p);

		let right = p.nth(0);

		// TODO: call, index, and construction expressions go here.

		if doomfront::parser::pratt::<Syntax>(left, right, PRATT_PRECEDENCE) {
			match right {
				Syntax::Dot => {
					let m = p.open_before(lhs);
					p.advance(Syntax::Dot);
					trivia_0plus(p);

					p.expect_any(
						&[
							(Syntax::Ident, Syntax::Ident),
							(Syntax::LitName, Syntax::LitName),
						],
						&[&["an identifier", "a name literal"]],
					);

					lhs = p.close(m, Syntax::ExprField);
				}
				Syntax::At => {
					let m = p.open_before(lhs);
					p.advance(Syntax::At);
					p.expect(Syntax::Ident, Syntax::Ident, &[&["an identifier"]]);
					trivia_0plus(p);
					block_end = recur(p, right);
					lhs = p.close(m, Syntax::ExprBin);
				}
				other => {
					let m = p.open_before(lhs);
					p.advance(other);
					trivia_0plus(p);
					block_end = recur(p, right);
					lhs = p.close(m, Syntax::ExprBin);
				}
			}
		} else {
			break;
		}
	}

	block_end
}

/// Returns `true` if the expression that was parsed ends with a block.
fn primary(p: &mut Parser<Syntax>) -> (CloseMark, bool) {
	let mark = p.open();

	match p.nth(0) {
		t @ Syntax::Ident => {
			p.advance(t);
			(p.close(mark, Syntax::ExprIdent), false)
		}
		t @ (Syntax::LitFalse
		| Syntax::LitFloat
		| Syntax::LitInt
		| Syntax::LitName
		| Syntax::LitNull
		| Syntax::LitTrue) => {
			p.advance(t);
			(p.close(mark, Syntax::ExprLit), false)
		}
		t @ Syntax::LitString => {
			p.advance(t);
			p.eat(Syntax::Ident, Syntax::Ident);
			(p.close(mark, Syntax::ExprLit), false)
		}
		t @ (Syntax::Bang | Syntax::Minus | Syntax::Tilde) => {
			p.advance(t);
			trivia_0plus(p);
			recur(p, t);
			(p.close(mark, Syntax::ExprPrefix), false)
		}
		t @ Syntax::ParenL => {
			p.advance(t);
			trivia_0plus(p);
			expr(p);
			trivia_0plus(p);
			p.expect(Syntax::ParenR, Syntax::ParenR, &[&["`)`"]]);
			(p.close(mark, Syntax::ExprGroup), false)
		}
		Syntax::BraceL => (block(p, mark, Syntax::ExprBlock, true), true),
		other => (
			p.advance_err_and_close(mark, other, Syntax::Error, &[&["TODO"]]),
			false,
		),
	}
}

const PRATT_PRECEDENCE: &[&[Syntax]] = &[
	&[
		Syntax::AmpersandEq,
		Syntax::Ampersand2Eq,
		Syntax::AngleL2Eq,
		Syntax::AngleR2Eq,
		Syntax::AsteriskEq,
		Syntax::Asterisk2Eq,
		Syntax::CaretEq,
		Syntax::Eq,
		Syntax::MinusEq,
		Syntax::PercentEq,
		Syntax::PipeEq,
		Syntax::Pipe2Eq,
		Syntax::PlusEq,
		Syntax::Plus2Eq,
		Syntax::SlashEq,
	],
	&[Syntax::Pipe2],
	&[Syntax::Ampersand2],
	&[Syntax::BangEq, Syntax::Eq2, Syntax::TildeEq2],
	&[
		Syntax::AngleL,
		Syntax::AngleR,
		Syntax::AngleLEq,
		Syntax::AngleREq,
	],
	&[Syntax::Plus2],
	&[Syntax::Pipe],
	&[Syntax::Caret],
	&[Syntax::Ampersand],
	&[Syntax::AngleL2, Syntax::AngleR2],
	&[Syntax::Plus, Syntax::Minus, Syntax::At],
	&[Syntax::Asterisk, Syntax::Slash, Syntax::Percent],
	&[Syntax::Asterisk2],
	&[Syntax::Bang, Syntax::Tilde],
	&[Syntax::Dot],
];
