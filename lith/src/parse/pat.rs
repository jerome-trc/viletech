//! Pattern parsers.

use doomfront::parser::Parser;

use crate::Syn;

use super::common::*;

pub(super) fn pattern(p: &mut Parser<Syn>) {
	let mark = p.open();

	match p.nth(0) {
		t @ Syn::Ident => {
			p.advance(t);
			p.close(mark, Syn::PatIdent);
		}
		t @ (Syn::LitTrue
		| Syn::LitFalse
		| Syn::LitInt
		| Syn::LitName
		| Syn::LitString
		| Syn::LitFloat) => {
			p.advance(t);
			p.close(mark, Syn::PatLit);
		}
		t @ Syn::Minus => {
			p.advance(t);
			trivia_0plus(p);

			p.expect_any(
				&[(Syn::LitInt, Syn::LitInt), (Syn::LitFloat, Syn::LitFloat)],
				&[&["an integer", "a floating-point number"]],
			);

			p.close(mark, Syn::PatLit);
		}
		t @ Syn::BracketL => {
			p.advance(t);
			trivia_0plus(p);

			while !p.eof() {
				pattern(p);
				trivia_0plus(p);

				if p.eat(Syn::Comma, Syn::Comma) {
					trivia_0plus(p);
					continue;
				} else {
					break;
				}
			}

			trivia_0plus(p);
			p.expect(Syn::BracketR, Syn::BracketR, &[&["TODO"]]);
			p.close(mark, Syn::PatSlice);
		}
		t @ Syn::ParenL => {
			p.advance(t);
			trivia_0plus(p);
			pattern(p);
			trivia_0plus(p);
			p.expect(Syn::ParenR, Syn::ParenR, &[&["TODO"]]);
			p.close(mark, Syn::PatGrouped);
		}
		t @ Syn::Underscore => {
			p.advance(t);
			p.close(mark, Syn::PatWildcard);
		}
		other => {
			p.advance_err_and_close(mark, other, Syn::Error, &[&["TODO"]]);
		}
	}
}
