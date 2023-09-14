//! Non-structural items; enums, functions, symbolic constants.

use doomfront::parser::{OpenMark, Parser};

use crate::Syn;

use super::{common::*, expr::*};

pub(super) struct ConstDef;

impl ConstDef {
	pub(super) const FIRST_SET: &[Syn] = &[Syn::KwConst];

	pub(super) fn parse(p: &mut Parser<Syn>, mark: OpenMark) {
		p.expect(Syn::KwConst, Syn::KwConst, &["`const`"]);
		trivia_0plus(p);
		p.expect(Syn::Ident, Syn::Ident, &["an identifier"]);
		trivia_0plus(p);

		if p.at_any(TypeSpec::FIRST_SET) {
			TypeSpec::parse(p);
			trivia_0plus(p);
		}

		p.expect(Syn::Eq, Syn::Eq, &["`=`"]);
		trivia_0plus(p);
		Expression::parse(p);
		trivia_0plus(p);
		p.expect(Syn::Semicolon, Syn::Semicolon, &["`;`"]);
		p.close(mark, Syn::ConstDef);
	}
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

		if p.eat(Syn::KwConst, Syn::KwConst) {
			trivia_0plus(p);
		}

		p.expect(Syn::Ident, Syn::Ident, &["an identifier", "`const`"]);
		trivia_0plus(p);
		TypeSpec::parse(p);

		if p.find(0, |token| !token.is_trivia()) == Syn::Eq {
			trivia_0plus(p);
			p.advance(Syn::Eq);
			trivia_0plus(p);
			let _ = Expression::parse(p);
		}

		p.close(mark, Syn::Parameter);
	}
}
