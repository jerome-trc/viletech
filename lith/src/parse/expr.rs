//! Expression parsers.

use doomfront::parser::{CloseMark, OpenMark, Parser};

use crate::Syn;

use super::common::*;

pub const EXPR_FIRST_SET: &[Syn] = &[
	Syn::Bang,
	Syn::Ident,
	Syn::LitFalse,
	Syn::LitFloat,
	Syn::LitInt,
	Syn::LitName,
	Syn::LitString,
	Syn::LitTrue,
	Syn::ParenL,
	Syn::Tilde,
];

/// Returns `true` if the expression that was parsed ends with a block.
/// `eq_op` dictates whether [`Syn::Eq`] is a valid infix operator in this position.
pub fn expr(p: &mut Parser<Syn>, eq_op: bool) -> bool {
	let t0 = p.nth(0);

	if matches!(t0, Syn::BracketL) {
		let block_end;
		let mark = p.open();

		loop {
			match p.nth(0) {
				t @ Syn::BracketL => {
					let pfx = p.open();
					p.advance(t);
					trivia_0plus(p);
					let _ = recur(p, eq_op, Syn::Eof);
					trivia_0plus(p);
					p.expect(Syn::BracketR, Syn::BracketR, &[&["`]`"]]);
					p.close(pfx, Syn::ArrayPrefix);
				}
				_ => {
					block_end = recur(p, eq_op, Syn::Eof);
					break;
				}
			}
		}

		p.close(mark, Syn::ExprType);
		block_end
	} else {
		recur(p, eq_op, Syn::Eof)
	}
}

/// Returns `true` if the expression that was parsed ends with a block.
fn recur(p: &mut Parser<Syn>, eq_op: bool, left: Syn) -> bool {
	let (mut lhs, mut block_end) = primary(p, eq_op);

	loop {
		trivia_0plus(p);

		let right = p.nth(0);

		match right {
			Syn::ParenL => {
				let m = p.open_before(lhs);
				trivia_0plus(p);
				arg_list(p);
				trivia_0plus(p);
				lhs = p.close(m, Syn::ExprCall);
				continue;
			}
			Syn::BracketL => {
				let m = p.open_before(lhs);
				p.advance(Syn::BracketL);
				trivia_0plus(p);
				expr(p, eq_op);
				trivia_0plus(p);
				p.expect(Syn::BracketR, Syn::BracketR, &[&["`]`"]]);
				lhs = p.close(m, Syn::ExprIndex);
				continue;
			}
			Syn::BraceL => {
				let m = p.open_before(lhs);
				p.advance(Syn::BraceL);
				lhs = init_list(p, m, Syn::ExprConstruct);
				continue;
			}
			_ => {}
		}

		if doomfront::parser::pratt::<Syn>(left, right, PRATT_PRECEDENCE) {
			match right {
				Syn::Dot => {
					let m = p.open_before(lhs);
					p.advance(Syn::Dot);
					trivia_0plus(p);

					p.expect_any(
						&[(Syn::Ident, Syn::Ident), (Syn::LitName, Syn::LitName)],
						&[&["an identifier", "a name literal"]],
					);

					lhs = p.close(m, Syn::ExprField);
				}
				Syn::At => {
					let m = p.open_before(lhs);
					p.advance(Syn::At);
					p.expect(Syn::Ident, Syn::Ident, &[&["an identifier"]]);
					trivia_0plus(p);
					block_end = recur(p, eq_op, right);
					lhs = p.close(m, Syn::ExprBin);
				}
				t @ Syn::Eq => {
					if eq_op {
						let m = p.open_before(lhs);
						p.advance(t);
						trivia_0plus(p);
						block_end = recur(p, eq_op, right);
						lhs = p.close(m, Syn::ExprBin);
					} else {
						break;
					}
				}
				other => {
					let m = p.open_before(lhs);
					p.advance(other);
					trivia_0plus(p);
					block_end = recur(p, eq_op, right);
					lhs = p.close(m, Syn::ExprBin);
				}
			}
		} else {
			break;
		}
	}

	block_end
}

/// Returns `true` if the expression that was parsed ends with a block.
fn primary(p: &mut Parser<Syn>, eq_op: bool) -> (CloseMark, bool) {
	let mark = p.open();

	match p.nth(0) {
		t @ Syn::Ident => {
			p.advance(t);
			(p.close(mark, Syn::ExprIdent), false)
		}
		t @ Syn::Dot => {
			p.advance(t);
			p.expect(Syn::Ident, Syn::Ident, &[&["an identifier"]]);
			(p.close(mark, Syn::ExprIdent), false)
		}
		t @ (Syn::LitFalse | Syn::LitFloat | Syn::LitInt | Syn::LitName | Syn::LitTrue) => {
			p.advance(t);
			(p.close(mark, Syn::ExprLit), false)
		}
		t @ Syn::LitString => {
			p.advance(t);
			p.eat(Syn::Ident, Syn::Ident);
			(p.close(mark, Syn::ExprLit), false)
		}
		t @ (Syn::Bang | Syn::Minus | Syn::Tilde) => {
			p.advance(t);
			trivia_0plus(p);
			recur(p, eq_op, t);
			(p.close(mark, Syn::ExprPrefix), false)
		}
		t @ Syn::ParenL => {
			p.advance(t);
			trivia_0plus(p);
			expr(p, eq_op);
			trivia_0plus(p);
			p.expect(Syn::ParenR, Syn::ParenR, &[&["`)`"]]);
			(p.close(mark, Syn::ExprGroup), false)
		}
		t @ Syn::DotBraceL => {
			p.advance(t);
			let ret = init_list(p, mark, Syn::ExprAggregate);
			(ret, true)
		}
		t @ Syn::KwStruct => {
			p.advance(t);
			trivia_0plus(p);
			p.expect(Syn::BraceL, Syn::BraceL, &[&["`{`"]]);
			trivia_0plus(p);

			while !p.at(Syn::BraceR) && !p.eof() {
				if p.at(Syn::Ident) {
					field_decl(p);
				} else {
					super::core_element::<false>(p);
				}

				trivia_0plus(p);
			}

			p.expect(Syn::BraceR, Syn::BraceR, &[&["TODO"]]);
			(p.close(mark, Syn::ExprStruct), true)
		}
		Syn::BraceL => (block(p, mark, Syn::ExprBlock, true), true),
		other => (
			p.advance_err_and_close(mark, other, Syn::Error, &[&["TODO"]]),
			false,
		),
	}
}

#[must_use]
fn init_list(p: &mut Parser<Syn>, mark: OpenMark, kind: Syn) -> CloseMark {
	trivia_0plus(p);

	while !p.eof() {
		if p.at(Syn::BraceR) {
			break;
		}

		aggregate_init(p);
		trivia_0plus(p);

		if !p.eat(Syn::Comma, Syn::Comma) {
			trivia_0plus(p);
			break;
		} else {
			trivia_0plus(p);
		}
	}

	p.expect(Syn::BraceR, Syn::BraceR, &[&["TODO"]]);
	p.close(mark, kind)
}

fn aggregate_init(p: &mut Parser<Syn>) {
	let mark = p.open();

	match p.nth(0) {
		t @ Syn::Dot => {
			p.advance(t);
			trivia_0plus(p);
			p.expect(Syn::Ident, Syn::Ident, &[&["TODO"]]);
			trivia_0plus(p);
			p.expect(Syn::Eq, Syn::Eq, &[&["TODO"]]);
			trivia_0plus(p);
			let _ = expr(p, true);
		}
		t @ Syn::BracketL => {
			p.advance(t);
			trivia_0plus(p);
			let _ = expr(p, true);
			trivia_0plus(p);
			p.expect(Syn::BracketR, Syn::BracketR, &[&["TODO"]]);
			trivia_0plus(p);
			p.expect(Syn::Eq, Syn::Eq, &[&["TODO"]]);
			trivia_0plus(p);
			let _ = expr(p, true);
		}
		_ => {
			let _ = expr(p, true);
		}
	}

	p.close(mark, Syn::AggregateInit);
}

fn field_decl(p: &mut Parser<Syn>) {
	let mark = p.open();
	p.expect(Syn::Ident, Syn::Ident, &[&["TODO"]]);
	trivia_0plus(p);
	type_spec(p, false);
	trivia_0plus(p);
	p.expect(Syn::Semicolon, Syn::Semicolon, &[&["TODO"]]);
	p.close(mark, Syn::FieldDecl);
}

const PRATT_PRECEDENCE: &[&[Syn]] = &[
	&[
		Syn::AmpersandEq,
		Syn::Ampersand2Eq,
		Syn::AngleL2Eq,
		Syn::AngleR2Eq,
		Syn::AsteriskEq,
		Syn::Asterisk2Eq,
		Syn::CaretEq,
		Syn::Eq,
		Syn::MinusEq,
		Syn::PercentEq,
		Syn::PipeEq,
		Syn::Pipe2Eq,
		Syn::PlusEq,
		Syn::Plus2Eq,
		Syn::SlashEq,
	],
	&[Syn::Pipe2],
	&[Syn::Ampersand2],
	&[Syn::BangEq, Syn::Eq2, Syn::TildeEq2],
	&[Syn::AngleL, Syn::AngleR, Syn::AngleLEq, Syn::AngleREq],
	&[Syn::Plus2],
	&[Syn::Pipe],
	&[Syn::Caret],
	&[Syn::Ampersand],
	&[Syn::AngleL2, Syn::AngleR2],
	&[Syn::Plus, Syn::Minus, Syn::At],
	&[Syn::Asterisk, Syn::Slash, Syn::Percent],
	&[Syn::Asterisk2],
	&[Syn::Bang, Syn::Tilde],
	&[Syn::Dot],
];
