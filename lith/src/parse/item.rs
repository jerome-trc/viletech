//! Function declarations.

use doomfront::parser::{OpenMark, Parser};

use crate::Syn;

use super::common::*;

#[must_use]
pub(super) fn at_function_decl(p: &Parser<Syn>) -> bool {
	p.at(Syn::KwFunction)
}

pub(super) fn function_decl(p: &mut Parser<Syn>, mark: OpenMark) {
	debug_assert!(at_function_decl(p));

	p.expect(Syn::KwFunction, Syn::KwFunction, &[&["TODO"]]);
	trivia_0plus(p);
	p.expect(Syn::Ident, Syn::Ident, &[&["TODO"]]);
	trivia_0plus(p);
	param_list(p);
	trivia_0plus(p);

	if at_type_spec(p) {
		type_spec(p);
		trivia_0plus(p);
	}

	if p.eat(Syn::Semicolon, Syn::Semicolon) {
		p.close(mark, Syn::FunctionDecl);
		return;
	}

	let body = p.open();
	block(p, body, Syn::FunctionBody, false);
	p.close(mark, Syn::FunctionDecl);
}

fn param_list(p: &mut Parser<Syn>) {
	let mark = p.open();
	p.expect(Syn::ParenL, Syn::ParenR, &[&["`(`"]]);
	trivia_0plus(p);

	while !p.at(Syn::ParenR) && !p.eof() {
		parameter(p);
		trivia_0plus(p);

		match p.nth(0) {
			t @ Syn::Comma => {
				p.advance(t);
				trivia_0plus(p);
			}
			Syn::ParenR => break,
			other => {
				p.advance_with_error(other, &[&["`,`", "`)`"]]);
			}
		}
	}

	p.expect(Syn::ParenR, Syn::ParenR, &[&["`)`"]]);
	p.close(mark, Syn::ParamList);
}

fn parameter(p: &mut Parser<Syn>) {
	let mark = p.open();

	if p.eat(Syn::KwConst, Syn::KwConst) {
		trivia_0plus(p);
	}

	p.expect(Syn::Ident, Syn::Ident, &[&["an identifier", "`const`"]]);
	trivia_0plus(p);
	type_spec(p);

	if p.find(0, |token| !token.is_trivia()) == Syn::ColonEq {
		trivia_0plus(p);
		p.advance(Syn::ColonEq);
		trivia_0plus(p);
		let _ = super::expr(p);
	}

	p.close(mark, Syn::Parameter);
}
