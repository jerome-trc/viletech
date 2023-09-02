//! Statement parsers.

use doomfront::parser::Parser;

use crate::Syn;

use super::{common::*, expr::*, item::*};

pub(super) fn statement(p: &mut Parser<Syn>) {
	if p.at_any(Annotation::FIRST_SET) {
		Annotation::parse(p);
		return;
	}

	if super::at_item(p) {
		super::item(p);
		return;
	}

	let mark = p.open();
	let token = p.nth(0);

	if Expression::FIRST_SET.contains(&token) {
		if !Expression::parse(p) {
			trivia_0plus(p);
			p.expect(Syn::Semicolon, Syn::Semicolon, &["`;`"]);
		}

		p.close(mark, Syn::ExprStat);
		return;
	}

	match token {
		t @ Syn::KwBreak => {
			p.advance(t);
			trivia_0plus(p);

			if p.at_any(BlockLabel::FIRST_SET) {
				BlockLabel::parse(p);
				trivia_0plus(p);
			}

			p.expect(Syn::Semicolon, Syn::Semicolon, &["`;`"]);
			p.close(mark, Syn::BreakStat);
		}
		t @ Syn::KwContinue => {
			p.advance(t);
			trivia_0plus(p);

			if p.at_any(BlockLabel::FIRST_SET) {
				BlockLabel::parse(p);
				trivia_0plus(p);
			}

			p.expect(Syn::Semicolon, Syn::Semicolon, &["`;`"]);
			p.close(mark, Syn::ContinueStat);
		}
		t @ (Syn::KwConst | Syn::KwLet) => {
			p.advance(t);
			trivia_0plus(p);
			p.expect(Syn::Ident, Syn::Ident, &["an identifier"]);
			trivia_0plus(p);

			if p.at_any(TypeSpec::FIRST_SET) {
				TypeSpec::parse(p);
				trivia_0plus(p);
			}

			if p.eat(Syn::Eq, Syn::Eq) {
				trivia_0plus(p);
				Expression::parse(p);
			}

			p.close(mark, Syn::BindStat);
		}
		t @ Syn::KwReturn => {
			p.advance(t);
			trivia_0plus(p);
			Expression::parse(p);
			trivia_0plus(p);
			p.expect(Syn::Semicolon, Syn::Semicolon, &["`;`"]);
			p.close(mark, Syn::ReturnStat);
		}
		other => {
			p.advance_err_and_close(
				mark,
				other,
				Syn::Error,
				&[
					"`break`",
					"`continue`",
					"`const` or `let`",
					"`return`",
					"an expression",
				],
			);
		}
	}
}
