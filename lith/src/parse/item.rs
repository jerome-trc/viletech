//! Function declarations and symbolic constants.

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
		type_spec(p, false);
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

	if p.eat(Syn::Dot3, Syn::Dot3) {
		trivia_0plus(p);
		p.expect(Syn::ParenR, Syn::ParenR, &[&["`)`"]]);
		p.close(mark, Syn::ParamList);
		return;
	}

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

	if p.eat(Syn::Ampersand, Syn::Ampersand) {
		trivia_0plus(p);

		if p.eat(Syn::KwVar, Syn::KwVar) {
			trivia_0plus(p);
		}
	}

	p.expect(Syn::Ident, Syn::Ident, &[&["an identifier"], &["TODO"]]);

	trivia_0plus(p);
	type_spec(p, true);

	if p.find(0, |token| !token.is_trivia()) == Syn::Eq {
		trivia_0plus(p);
		p.advance(Syn::Eq);
		trivia_0plus(p);
		let _ = super::expr(p, true);
	}

	p.close(mark, Syn::Parameter);
}

#[must_use]
pub(super) fn at_symbolic_constant(p: &Parser<Syn>) -> bool {
	p.at(Syn::KwConst)
}

pub(super) fn symbolic_constant(p: &mut Parser<Syn>, mark: OpenMark) {
	p.expect(Syn::KwConst, Syn::KwConst, &[&["TODO"]]);
	trivia_0plus(p);
	p.expect(Syn::Ident, Syn::Ident, &[&["an identifier"]]);
	trivia_0plus(p);
	type_spec(p, false);
	trivia_0plus(p);
	p.expect(Syn::Eq, Syn::Eq, &[&["TODO"]]);
	trivia_0plus(p);
	super::expr(p, true);
	trivia_0plus(p);
	p.expect(Syn::Semicolon, Syn::Semicolon, &[&["TODO"]]);
	p.close(mark, Syn::SymConst);
}
