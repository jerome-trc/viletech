//! Expression parsers.

use doomfront::parser::{CloseMark, OpenMark, Parser};

use crate::Syntax;

use super::common::*;

pub const EXPR_FIRST_SET: &[Syntax] = &[
	Syntax::Bang,
	Syntax::Ident,
	Syntax::LitFalse,
	Syntax::LitFloat,
	Syntax::LitInt,
	Syntax::LitName,
	Syntax::LitString,
	Syntax::LitTrue,
	Syntax::ParenL,
	Syntax::Tilde,
];

/// Returns `true` if the expression that was parsed ends with a block.
/// `eq_op` dictates whether [`Syntax::Eq`] is a valid infix operator in this position.
pub fn expr(p: &mut Parser<Syntax>, eq_op: bool) -> bool {
	let t0 = p.nth(0);

	if matches!(t0, Syntax::BracketL) {
		let block_end;
		let mark = p.open();

		loop {
			match p.nth(0) {
				t @ Syntax::BracketL => {
					let pfx = p.open();
					p.advance(t);
					trivia_0plus(p);
					let _ = recur(p, eq_op, Syntax::Eof);
					trivia_0plus(p);
					p.expect(Syntax::BracketR, Syntax::BracketR, &[&["`]`"]]);
					p.close(pfx, Syntax::ArrayPrefix);
				}
				_ => {
					block_end = recur(p, eq_op, Syntax::Eof);
					break;
				}
			}
		}

		p.close(mark, Syntax::ExprType);
		block_end
	} else {
		recur(p, eq_op, Syntax::Eof)
	}
}

/// Returns `true` if the expression that was parsed ends with a block.
fn recur(p: &mut Parser<Syntax>, eq_op: bool, left: Syntax) -> bool {
	let (mut lhs, mut block_end) = primary(p, eq_op);

	loop {
		trivia_0plus(p);

		let right = p.nth(0);

		match right {
			Syntax::ParenL => {
				let m = p.open_before(lhs);
				trivia_0plus(p);
				arg_list(p);
				trivia_0plus(p);
				lhs = p.close(m, Syntax::ExprCall);
				continue;
			}
			Syntax::BracketL => {
				let m = p.open_before(lhs);
				p.advance(Syntax::BracketL);
				trivia_0plus(p);
				expr(p, eq_op);
				trivia_0plus(p);
				p.expect(Syntax::BracketR, Syntax::BracketR, &[&["`]`"]]);
				lhs = p.close(m, Syntax::ExprIndex);
				continue;
			}
			Syntax::BraceL => {
				let m = p.open_before(lhs);
				p.advance(Syntax::BraceL);
				lhs = init_list(p, m, Syntax::ExprConstruct);
				continue;
			}
			_ => {}
		}

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
					block_end = recur(p, eq_op, right);
					lhs = p.close(m, Syntax::ExprBin);
				}
				t @ Syntax::Eq => {
					if eq_op {
						let m = p.open_before(lhs);
						p.advance(t);
						trivia_0plus(p);
						block_end = recur(p, eq_op, right);
						lhs = p.close(m, Syntax::ExprBin);
					} else {
						break;
					}
				}
				other => {
					let m = p.open_before(lhs);
					p.advance(other);
					trivia_0plus(p);
					block_end = recur(p, eq_op, right);
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
fn primary(p: &mut Parser<Syntax>, eq_op: bool) -> (CloseMark, bool) {
	let mark = p.open();

	match p.nth(0) {
		t @ Syntax::Ident => {
			p.advance(t);
			(p.close(mark, Syntax::ExprIdent), false)
		}
		t @ Syntax::Dot => {
			p.advance(t);
			p.expect(Syntax::Ident, Syntax::Ident, &[&["an identifier"]]);
			(p.close(mark, Syntax::ExprIdent), false)
		}
		t @ (Syntax::LitFalse
		| Syntax::LitFloat
		| Syntax::LitInt
		| Syntax::LitName
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
			recur(p, eq_op, t);
			(p.close(mark, Syntax::ExprPrefix), false)
		}
		t @ Syntax::ParenL => {
			p.advance(t);
			trivia_0plus(p);
			expr(p, eq_op);
			trivia_0plus(p);
			p.expect(Syntax::ParenR, Syntax::ParenR, &[&["`)`"]]);
			(p.close(mark, Syntax::ExprGroup), false)
		}
		t @ Syntax::DotBraceL => {
			p.advance(t);
			let ret = init_list(p, mark, Syntax::ExprAggregate);
			(ret, true)
		}
		t @ Syntax::KwStruct => {
			p.advance(t);
			trivia_0plus(p);
			p.expect(Syntax::BraceL, Syntax::BraceL, &[&["`{`"]]);
			trivia_0plus(p);

			while !p.at(Syntax::BraceR) && !p.eof() {
				if p.at(Syntax::Ident) {
					field_decl(p);
				} else {
					super::core_element::<false>(p);
				}

				trivia_0plus(p);
			}

			p.expect(Syntax::BraceR, Syntax::BraceR, &[&["TODO"]]);
			(p.close(mark, Syntax::ExprStruct), true)
		}
		Syntax::BraceL => (block(p, mark, Syntax::ExprBlock, true), true),
		other => (
			p.advance_err_and_close(mark, other, Syntax::Error, &[&["TODO"]]),
			false,
		),
	}
}

#[must_use]
fn init_list(p: &mut Parser<Syntax>, mark: OpenMark, kind: Syntax) -> CloseMark {
	trivia_0plus(p);

	while !p.eof() {
		if p.at(Syntax::BraceR) {
			break;
		}

		aggregate_init(p);
		trivia_0plus(p);

		if !p.eat(Syntax::Comma, Syntax::Comma) {
			trivia_0plus(p);
			break;
		} else {
			trivia_0plus(p);
		}
	}

	p.expect(Syntax::BraceR, Syntax::BraceR, &[&["TODO"]]);
	p.close(mark, kind)
}

fn aggregate_init(p: &mut Parser<Syntax>) {
	let mark = p.open();

	match p.nth(0) {
		t @ Syntax::Dot => {
			p.advance(t);
			trivia_0plus(p);
			p.expect(Syntax::Ident, Syntax::Ident, &[&["TODO"]]);
			trivia_0plus(p);
			p.expect(Syntax::Eq, Syntax::Eq, &[&["TODO"]]);
			trivia_0plus(p);
			let _ = expr(p, true);
		}
		t @ Syntax::BracketL => {
			p.advance(t);
			trivia_0plus(p);
			let _ = expr(p, true);
			trivia_0plus(p);
			p.expect(Syntax::BracketR, Syntax::BracketR, &[&["TODO"]]);
			trivia_0plus(p);
			p.expect(Syntax::Eq, Syntax::Eq, &[&["TODO"]]);
			trivia_0plus(p);
			let _ = expr(p, true);
		}
		_ => {
			let _ = expr(p, true);
		}
	}

	p.close(mark, Syntax::AggregateInit);
}

fn field_decl(p: &mut Parser<Syntax>) {
	let mark = p.open();
	p.expect(Syntax::Ident, Syntax::Ident, &[&["TODO"]]);
	trivia_0plus(p);
	type_spec(p, false);
	trivia_0plus(p);
	p.expect(Syntax::Semicolon, Syntax::Semicolon, &[&["TODO"]]);
	p.close(mark, Syntax::FieldDecl);
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
