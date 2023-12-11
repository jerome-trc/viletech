//! Function declarations and symbolic constants.

use doomfront::parser::{OpenMark, Parser};

use crate::Syntax;

use super::common::*;

#[must_use]
pub(super) fn at_function_decl(p: &Parser<Syntax>) -> bool {
	p.at(Syntax::KwFunction)
}

pub(super) fn function_decl(p: &mut Parser<Syntax>, mark: OpenMark) {
	debug_assert!(at_function_decl(p));

	p.expect(Syntax::KwFunction, Syntax::KwFunction, &[&["TODO"]]);
	trivia_0plus(p);
	p.expect(Syntax::Ident, Syntax::Ident, &[&["TODO"]]);
	trivia_0plus(p);
	param_list(p);
	trivia_0plus(p);

	if at_type_spec(p) {
		type_spec(p, false);
		trivia_0plus(p);
	}

	if p.eat(Syntax::Semicolon, Syntax::Semicolon) {
		p.close(mark, Syntax::FunctionDecl);
		return;
	}

	let body = p.open();
	block(p, body, Syntax::FunctionBody, false);
	p.close(mark, Syntax::FunctionDecl);
}

fn param_list(p: &mut Parser<Syntax>) {
	let mark = p.open();
	p.expect(Syntax::ParenL, Syntax::ParenR, &[&["`(`"]]);
	trivia_0plus(p);

	if p.eat(Syntax::Dot3, Syntax::Dot3) {
		trivia_0plus(p);
		p.expect(Syntax::ParenR, Syntax::ParenR, &[&["`)`"]]);
		p.close(mark, Syntax::ParamList);
		return;
	}

	while !p.at(Syntax::ParenR) && !p.eof() {
		parameter(p);
		trivia_0plus(p);

		match p.nth(0) {
			t @ Syntax::Comma => {
				p.advance(t);
				trivia_0plus(p);
			}
			Syntax::ParenR => break,
			other => {
				p.advance_with_error(other, &[&["`,`", "`)`"]]);
			}
		}
	}

	p.expect(Syntax::ParenR, Syntax::ParenR, &[&["`)`"]]);
	p.close(mark, Syntax::ParamList);
}

fn parameter(p: &mut Parser<Syntax>) {
	let mark = p.open();

	if p.eat(Syntax::KwConst, Syntax::KwConst) {
		trivia_0plus(p);
	}

	if p.eat(Syntax::Ampersand, Syntax::Ampersand) {
		trivia_0plus(p);

		if p.eat(Syntax::KwVar, Syntax::KwVar) {
			trivia_0plus(p);
		}
	}

	p.expect(
		Syntax::Ident,
		Syntax::Ident,
		&[&["an identifier"], &["TODO"]],
	);

	trivia_0plus(p);
	type_spec(p, true);

	if p.find(0, |token| !token.is_trivia()) == Syntax::Eq {
		trivia_0plus(p);
		p.advance(Syntax::Eq);
		trivia_0plus(p);
		let _ = super::expr(p, true);
	}

	p.close(mark, Syntax::Parameter);
}

#[must_use]
pub(super) fn at_symbolic_constant(p: &Parser<Syntax>) -> bool {
	p.at(Syntax::KwConst)
}

pub(super) fn symbolic_constant(p: &mut Parser<Syntax>, mark: OpenMark) {
	p.expect(Syntax::KwConst, Syntax::KwConst, &[&["TODO"]]);
	trivia_0plus(p);
	p.expect(Syntax::Ident, Syntax::Ident, &[&["an identifier"]]);
	trivia_0plus(p);
	type_spec(p, false);
	trivia_0plus(p);
	p.expect(Syntax::Eq, Syntax::Eq, &[&["TODO"]]);
	trivia_0plus(p);
	super::expr(p, true);
	trivia_0plus(p);
	p.expect(Syntax::Semicolon, Syntax::Semicolon, &[&["TODO"]]);
	p.close(mark, Syntax::SymConst);
}
