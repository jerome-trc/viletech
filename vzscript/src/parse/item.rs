//! Non-structural items; enums, functions, symbolic constants, type aliases.

use doomfront::parser::{OpenMark, Parser};

use crate::Syn;

use super::common::*;

/// Note that this also covers [annotations](Annotation).
pub(super) fn item(p: &mut Parser<Syn>) {
	if p.at_any(Annotation::FIRST_SET) {
		Annotation::parse(p);
		return;
	}

	let mark = p.open();

	while !p.eof() && p.at_any(Attribute::FIRST_SET) {
		Attribute::parse(p);
	}

	trivia_0plus(p);
	doc_comments(p);
	trivia_0plus(p);
	let token = p.nth(0);

	if FuncDecl::FIRST_SET.contains(&token) {
		FuncDecl::parse(p, mark);
	} else {
		p.advance_err_and_close(mark, token, Syn::Error, &["`func`", "`#`"]);
	}
}

#[must_use]
pub(super) fn at_item(p: &mut Parser<Syn>) -> bool {
	p.at_any(Attribute::FIRST_SET) || p.at_any(FuncDecl::FIRST_SET)
}

pub(super) struct FuncDecl;

impl FuncDecl {
	pub(super) const FIRST_SET: &[Syn] = &[Syn::KwFunction];

	pub(super) fn parse(p: &mut Parser<Syn>, mark: OpenMark) {
		p.expect(Syn::KwFunction, Syn::KwFunction, &["`function`"]);
		trivia_0plus(p);
		p.expect(Syn::Ident, Syn::Ident, &["an identifier"]);
		trivia_0plus(p);
		Self::param_list(p);
		trivia_0plus(p);

		if p.at_any(TypeSpec::FIRST_SET) {
			TypeSpec::parse(p);
			trivia_0plus(p);
		}

		if p.eat(Syn::Semicolon, Syn::Semicolon) {
			p.close(mark, Syn::FuncDecl);
			return;
		}

		let body = p.open();
		block(p, body, Syn::FuncBody, false);
		p.close(mark, Syn::FuncDecl);
	}

	fn param_list(p: &mut Parser<Syn>) {
		let mark = p.open();
		p.expect(Syn::ParenL, Syn::ParenR, &["`(`"]);
		trivia_0plus(p);

		while !p.at(Syn::ParenR) && !p.eof() {
			Self::parameter(p);
			trivia_0plus(p);

			match p.nth(0) {
				t @ Syn::Comma => {
					p.advance(t);
					trivia_0plus(p);
				}
				Syn::ParenR => break,
				other => {
					p.advance_with_error(other, &["`,`", "`)`"]);
				}
			}
		}

		p.expect(Syn::ParenR, Syn::ParenR, &["`)`"]);
		p.close(mark, Syn::ParamList);
	}

	fn parameter(p: &mut Parser<Syn>) {
		let mark = p.open();
		p.expect(Syn::Ident, Syn::Ident, &["`an identifier`"]);
		trivia_0plus(p);
		TypeSpec::parse(p);
		p.close(mark, Syn::Parameter);
	}
}
