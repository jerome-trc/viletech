//! Pattern parsers.

use doomfront::parser::Parser;

use crate::Syntax;

use super::common::*;

pub(super) fn pattern(p: &mut Parser<Syntax>) {
	let mark = p.open();

	match p.nth(0) {
		t @ Syntax::Ident => {
			p.advance(t);
			p.close(mark, Syntax::PatIdent);
		}
		t @ (Syntax::LitTrue
		| Syntax::LitFalse
		| Syntax::LitInt
		| Syntax::LitName
		| Syntax::LitString
		| Syntax::LitFloat) => {
			p.advance(t);
			p.close(mark, Syntax::PatLit);
		}
		t @ Syntax::Minus => {
			p.advance(t);
			trivia_0plus(p);

			p.expect_any(
				&[
					(Syntax::LitInt, Syntax::LitInt),
					(Syntax::LitFloat, Syntax::LitFloat),
				],
				&[&["an integer", "a floating-point number"]],
			);

			p.close(mark, Syntax::PatLit);
		}
		t @ Syntax::BracketL => {
			p.advance(t);
			trivia_0plus(p);

			while !p.eof() {
				if p.at(Syntax::BracketR) {
					break;
				}

				pattern(p);
				trivia_0plus(p);

				if !p.eat(Syntax::Comma, Syntax::Comma) {
					trivia_0plus(p);
					break;
				} else {
					trivia_0plus(p);
				}
			}

			trivia_0plus(p);
			p.expect(Syntax::BracketR, Syntax::BracketR, &[&["TODO"]]);
			p.close(mark, Syntax::PatSlice);
		}
		t @ Syntax::ParenL => {
			p.advance(t);
			trivia_0plus(p);
			pattern(p);
			trivia_0plus(p);
			p.expect(Syntax::ParenR, Syntax::ParenR, &[&["TODO"]]);
			p.close(mark, Syntax::PatGrouped);
		}
		t @ Syntax::Underscore => {
			p.advance(t);
			p.close(mark, Syntax::PatWildcard);
		}
		other => {
			p.advance_err_and_close(mark, other, Syntax::Error, &[&["TODO"]]);
		}
	}
}
