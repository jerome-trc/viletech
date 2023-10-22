//! Statement parsers.

use doomfront::parser::{OpenMark, Parser};

use crate::Syn;

use super::common::*;

pub(super) fn statement(p: &mut Parser<Syn>, mark: OpenMark) {
	match p.nth(0) {
		t @ Syn::KwReturn => {
			p.advance(t);
			trivia_0plus(p);
			p.expect(Syn::Semicolon, Syn::Semicolon, &[&["`;`"]]);
			p.close(mark, Syn::StmtReturn);
		}
		t @ Syn::KwBreak => {
			p.advance(t);
			trivia_0plus(p);

			if at_block_label(p) {
				block_label(p);
				trivia_0plus(p);
			}

			if !p.at(Syn::Semicolon) {
				super::expr(p, true);
				trivia_0plus(p);
			}

			p.expect(Syn::Semicolon, Syn::Semicolon, &[&["TODO"]]);
			p.close(mark, Syn::StmtBreak);
		}
		t @ (Syn::KwLet | Syn::KwVar) => {
			p.advance(t);
			trivia_0plus(p);

			if p.eat(Syn::KwConst, Syn::KwConst) {
				trivia_0plus(p);
			}

			super::pattern(p);
			trivia_0plus(p);

			if at_type_spec(p) {
				type_spec(p, false);
				trivia_0plus(p);
			}

			if p.eat(Syn::Eq, Syn::Eq) {
				trivia_0plus(p);
				super::expr(p, true);
				trivia_0plus(p);
			}

			p.expect(Syn::Semicolon, Syn::Semicolon, &[&["`;`"]]);
			p.close(mark, Syn::StmtBind);
		}
		t @ Syn::KwContinue => {
			p.advance(t);
			trivia_0plus(p);

			if at_block_label(p) {
				block_label(p);
				trivia_0plus(p);
			}

			p.expect(Syn::Semicolon, Syn::Semicolon, &[&["`;`"]]);
			p.close(mark, Syn::StmtContinue);
		}
		_ => {
			let block_end = super::expr(p, true);

			if !block_end {
				trivia_0plus(p);
				p.expect(Syn::Semicolon, Syn::Semicolon, &[&["`;`"]])
			}

			p.close(mark, Syn::StmtExpr);
		}
	}
}
