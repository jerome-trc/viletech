//! Statement parsers.

use doomfront::parser::{OpenMark, Parser};

use crate::Syntax;

use super::common::*;

pub(super) fn statement(p: &mut Parser<Syntax>, mark: OpenMark) {
	match p.nth(0) {
		t @ Syntax::KwReturn => {
			p.advance(t);
			trivia_0plus(p);

			if !p.at(Syntax::Semicolon) {
				super::expr(p, true);
				trivia_0plus(p);
			}

			p.expect(Syntax::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
			p.close(mark, Syntax::StmtReturn);
		}
		t @ Syntax::KwBreak => {
			p.advance(t);
			trivia_0plus(p);

			if at_block_label(p) {
				block_label(p);
				trivia_0plus(p);
			}

			if !p.at(Syntax::Semicolon) {
				super::expr(p, true);
				trivia_0plus(p);
			}

			p.expect(Syntax::Semicolon, Syntax::Semicolon, &[&["TODO"]]);
			p.close(mark, Syntax::StmtBreak);
		}
		t @ (Syntax::KwLet | Syntax::KwVar) => {
			p.advance(t);
			trivia_0plus(p);

			if p.eat(Syntax::KwConst, Syntax::KwConst) {
				trivia_0plus(p);
			}

			super::pattern(p);
			trivia_0plus(p);

			if at_type_spec(p) {
				type_spec(p, false);
				trivia_0plus(p);
			}

			if p.eat(Syntax::Eq, Syntax::Eq) {
				trivia_0plus(p);
				super::expr(p, true);
				trivia_0plus(p);
			}

			p.expect(Syntax::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
			p.close(mark, Syntax::StmtBind);
		}
		t @ Syntax::KwContinue => {
			p.advance(t);
			trivia_0plus(p);

			if at_block_label(p) {
				block_label(p);
				trivia_0plus(p);
			}

			p.expect(Syntax::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
			p.close(mark, Syntax::StmtContinue);
		}
		_ => {
			let block_end = super::expr(p, true);

			if !block_end {
				trivia_0plus(p);
				p.expect(Syntax::Semicolon, Syntax::Semicolon, &[&["`;`"]])
			}

			p.close(mark, Syntax::StmtExpr);
		}
	}
}
