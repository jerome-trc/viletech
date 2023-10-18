//! Functions for parsing different elements of the Lithica syntax.

mod common;
mod expr;
mod item;
mod pat;

#[cfg(test)]
mod test;

use doomfront::parser::{OpenMark, Parser};

use crate::Syn;

pub use self::expr::*;

use self::{common::*, item::*, pat::*};

pub type Error = doomfront::ParseError<Syn>;

pub fn file(p: &mut Parser<Syn>) {
	let root = p.open();

	while !p.eof() {
		if trivia(p) {
			continue;
		}

		core_element::<true>(p);
	}

	p.close(root, Syn::FileRoot);
}

/// An inner annotation, import, item, or statement. If `ROOT`, statements are forbidden.
fn core_element<const ROOT: bool>(p: &mut Parser<Syn>) {
	if at_inner_annotation(p) {
		annotation(p, true);
		return;
	}

	let mark = p.open();

	while p.eat(Syn::DocComment, Syn::DocComment) && !p.eof() {
		trivia_0plus(p);
	}

	while at_annotation(p) {
		annotation(p, false);
		trivia_0plus(p);
	}

	if at_function_decl(p) {
		function_decl(p, mark);
	} else if at_symbolic_constant(p) {
		symbolic_constant(p, mark);
	} else if p.at(Syn::KwImport) {
		import(p, mark);
	} else if !ROOT {
		todo!("statement parsing")
	} else {
		p.advance_err_and_close(mark, p.nth(0), Syn::Error, &[&["TODO"]]);
	}
}

pub(super) fn import(p: &mut Parser<Syn>, mark: OpenMark) {
	p.expect(Syn::KwImport, Syn::KwImport, &[&["TODO"]]);
	trivia_0plus(p);
	p.expect(Syn::LitString, Syn::LitString, &[&["TODO"]]);
	trivia_0plus(p);
	p.expect(Syn::Colon, Syn::Colon, &[&["TODO"]]);
	trivia_0plus(p);

	let inner = p.open();

	if p.eat(Syn::Asterisk, Syn::Asterisk) {
		trivia_0plus(p);
		p.expect(Syn::ThickArrow, Syn::ThickArrow, &[&["TODO"]]);
		trivia_0plus(p);
		p.expect(Syn::Ident, Syn::Ident, &[&["TODO"]]);
		p.close(inner, Syn::ImportAll);
		trivia_0plus(p);
	} else {
		while !p.eof() {
			import_entry(p);
			trivia_0plus(p);

			match p.nth(0) {
				t @ Syn::Comma => {
					p.advance(t);
					trivia_0plus(p);
				}
				Syn::Semicolon => break,
				other => {
					p.advance_with_error(other, &[&["TODO"]]);
				}
			}
		}

		p.close(inner, Syn::ImportList);
	}

	p.expect(Syn::Semicolon, Syn::Semicolon, &[&["TODO"]]);
	p.close(mark, Syn::Import);
}

fn import_entry(p: &mut Parser<Syn>) {
	let mark = p.open();

	p.expect_any(
		&[(Syn::Ident, Syn::Ident), (Syn::LitName, Syn::LitName)],
		&[&["TODO"]],
	);

	if p.find(0, |t| !t.is_trivia()) == Syn::ThickArrow {
		trivia_0plus(p);
		p.advance(Syn::ThickArrow);
		trivia_0plus(p);
		p.expect(Syn::Ident, Syn::Ident, &[&["TODO"]]);
	}

	p.close(mark, Syn::ImportEntry);
}
