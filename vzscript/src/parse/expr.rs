//! Expression parsers.

use doomfront::parser::{CloseMark, Parser};

use crate::Syn;

use super::{common::*, structure::*};

pub(super) struct Expression;

impl Expression {
	pub(super) const FIRST_SET: &[Syn] = &[
		Syn::AtParenL,
		Syn::Bang,
		Syn::BraceL,
		Syn::DotBraceL,
		Syn::FalseLit,
		Syn::FloatLit,
		Syn::Ident,
		Syn::IntLit,
		Syn::KwClass,
		Syn::KwEnum,
		Syn::KwFor,
		Syn::KwStruct,
		Syn::KwSwitch,
		Syn::KwUnion,
		Syn::KwWhile,
		Syn::Minus,
		Syn::NameLit,
		Syn::NullLit,
		Syn::ParenL,
		Syn::StringLit,
		Syn::Tilde,
		Syn::TrueLit,
	];

	/// Returns `true` if the expression that was parsed ends with a block.
	pub(super) fn parse(p: &mut Parser<Syn>) -> bool {
		let t0 = p.nth(0);

		if matches!(t0, Syn::KwAuto | Syn::KwType) {
			let mark = p.open();
			p.advance(t0);
			p.close(mark, Syn::TypeExpr);
			false
		} else if matches!(t0, Syn::BracketL | Syn::Question | Syn::Ampersand) {
			let mut block_end = false;
			let mark = p.open();

			loop {
				match p.nth(0) {
					t @ Syn::BracketL => {
						let pfx = p.open();
						p.advance(t);
						trivia_0plus(p);
						let _ = recur(p, Syn::Eof);
						trivia_0plus(p);
						p.expect(Syn::BracketR, Syn::BracketR, &[&["`]`"]]);
						p.close(pfx, Syn::ArrayPrefix);
					}
					t @ Syn::Ampersand => {
						let pfx = p.open();
						p.advance(t);
						p.close(pfx, Syn::RefPrefix);
					}
					t @ Syn::Question => {
						let pfx = p.open();
						p.advance(t);
						p.close(pfx, Syn::OptionPrefix);
					}
					_ => {
						block_end = recur(p, Syn::Eof);
						break;
					}
				}
			}

			p.close(mark, Syn::TypeExpr);
			block_end
		} else {
			recur(p, Syn::Eof)
		}
	}
}

fn recur(p: &mut Parser<Syn>, left: Syn) -> bool {
	let (mut lhs, mut block_end) = primary(p);

	loop {
		trivia_0plus(p);

		let right = p.nth(0);

		match right {
			Syn::ParenL => {
				let m = p.open_before(lhs);
				trivia_0plus(p);
				ArgList::parse(p);
				trivia_0plus(p);
				lhs = p.close(m, Syn::CallExpr);
				continue;
			}
			Syn::BracketL => {
				let m = p.open_before(lhs);
				p.advance(Syn::BracketL);
				trivia_0plus(p);
				Expression::parse(p);
				trivia_0plus(p);
				p.expect(Syn::BracketR, Syn::BracketR, &[&["`]`"]]);
				lhs = p.close(m, Syn::IndexExpr);
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
					p.expect(Syn::Ident, Syn::Ident, &[&["an identifier"]]);
					lhs = p.close(m, Syn::FieldExpr);
				}
				other => {
					let m = p.open_before(lhs);
					p.advance(other);
					trivia_0plus(p);
					block_end = recur(p, right);
					lhs = p.close(m, Syn::BinExpr);
				}
			}
		} else {
			break;
		}
	}

	block_end
}

#[must_use]
fn primary(p: &mut Parser<Syn>) -> (CloseMark, bool) {
	let mark = p.open();

	match p.nth(0) {
		t @ Syn::Ident => {
			p.advance(t);
			(p.close(mark, Syn::IdentExpr), false)
		}
		t @ Syn::Dot => {
			p.advance(t);
			p.expect(Syn::Ident, Syn::Ident, &[&["an identifier"]]);
			(p.close(mark, Syn::IdentExpr), false)
		}
		t @ (Syn::FalseLit
		| Syn::FloatLit
		| Syn::IntLit
		| Syn::NameLit
		| Syn::NullLit
		| Syn::StringLit
		| Syn::TrueLit) => {
			p.advance(t);
			(p.close(mark, Syn::Literal), false)
		}
		t @ (Syn::Bang | Syn::Minus | Syn::Tilde) => {
			p.advance(t);
			trivia_0plus(p);
			recur(p, t);
			(p.close(mark, Syn::PrefixExpr), false)
		}
		t @ Syn::ParenL => {
			p.advance(t);
			trivia_0plus(p);
			Expression::parse(p);
			trivia_0plus(p);
			p.expect(Syn::ParenR, Syn::ParenR, &[&["`)`"]]);
			(p.close(mark, Syn::GroupExpr), false)
		}
		t @ Syn::KwFor => {
			p.advance(t);
			trivia_0plus(p);
			p.expect(Syn::Ident, Syn::Ident, &[&["an identifier"]]);
			// TODO: Patterns?
			trivia_0plus(p);
			p.expect(Syn::Colon, Syn::Colon, &[&["`:`"]]);
			trivia_0plus(p);
			Expression::parse(p);
			trivia_0plus(p);
			let body = p.open();
			block(p, body, Syn::Block, true);
			(p.close(mark, Syn::ForExpr), true)
		}
		t @ Syn::KwWhile => {
			p.advance(t);
			trivia_0plus(p);
			Expression::parse(p);
			trivia_0plus(p);
			let body = p.open();
			block(p, body, Syn::Block, true);
			(p.close(mark, Syn::WhileExpr), true)
		}
		t @ Syn::DotBraceL => {
			p.advance(t);
			trivia_0plus(p);
			todo!();
			p.expect(Syn::BraceR, Syn::BraceR, &[&["`}`"]]);
			(p.close(mark, Syn::ConstructExpr), true)
		}
		t @ Syn::KwClass => {
			p.advance(t);
			trivia_0plus(p);

			if p.at_any(TypeSpec::FIRST_SET) {
				TypeSpec::parse(p);
				trivia_0plus(p);
			}

			p.expect(Syn::BraceL, Syn::BraceL, &[&["`{`"]]);
			trivia_0plus(p);

			while !p.at(Syn::BraceR) && !p.eof() {
				struct_innard(p);
				trivia_0plus(p);
			}

			p.expect(Syn::BraceR, Syn::BraceR, &[&["`}`"]]);
			(p.close(mark, Syn::ClassExpr), true)
		}
		t @ Syn::KwEnum => {
			p.advance(t);
			trivia_0plus(p);
			p.expect(Syn::BraceL, Syn::BraceL, &[&["`{`"]]);
			todo!();
			p.expect(Syn::BraceR, Syn::BraceR, &[&["`}`"]]);
			(p.close(mark, Syn::EnumExpr), true)
		}
		t @ Syn::KwStruct => {
			p.advance(t);
			trivia_0plus(p);
			p.expect(Syn::BraceL, Syn::BraceL, &[&["`{`"]]);
			trivia_0plus(p);

			while !p.at(Syn::BraceR) && !p.eof() {
				struct_innard(p);
				trivia_0plus(p);
			}

			p.expect(Syn::BraceR, Syn::BraceR, &[&["`}`"]]);
			(p.close(mark, Syn::StructExpr), true)
		}
		t @ Syn::KwUnion => {
			p.advance(t);
			trivia_0plus(p);
			p.expect(Syn::BraceL, Syn::BraceL, &[&["`{`"]]);
			todo!();
			p.expect(Syn::BraceR, Syn::BraceR, &[&["`}`"]]);
			(p.close(mark, Syn::UnionExpr), true)
		}
		Syn::BraceL => (block(p, mark, Syn::BlockExpr, false), false),
		other => (
			p.advance_err_and_close(
				mark,
				other,
				Syn::Error,
				&[
					&["This is a placeholder error message!"],
					// TODO
				],
			),
			false,
		),
	}
}

const PRATT_PRECEDENCE: &[&[Syn]] = &[
	&[
		// `Eq` might go here, but it causes trouble for param. defaults.
		Syn::AsteriskEq,
		Syn::SlashEq,
		Syn::PercentEq,
		Syn::PlusEq,
		Syn::MinusEq,
		Syn::AngleL2Eq,
		Syn::AngleR2Eq,
		Syn::AmpersandEq,
		Syn::PipeEq,
		Syn::CaretEq,
	],
	&[Syn::Pipe2],
	&[Syn::Ampersand2],
	&[Syn::Eq2, Syn::BangEq, Syn::TildeEq2],
	&[
		Syn::AngleL,
		Syn::AngleR,
		Syn::AngleLEq,
		Syn::AngleREq,
		Syn::KwIs,
		Syn::KwIsNot,
	],
	&[Syn::Pipe],
	&[Syn::Caret],
	&[Syn::Ampersand],
	&[Syn::AngleL2, Syn::AngleR2],
	&[Syn::Plus, Syn::Minus],
	&[Syn::Asterisk, Syn::Slash, Syn::Percent],
	&[Syn::Asterisk2],
	&[Syn::Bang, Syn::Tilde],
	&[Syn::Dot],
];
