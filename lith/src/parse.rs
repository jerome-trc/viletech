//! Functions for parsing different elements of the Lithica syntax.

mod common;
mod expr;
mod item;
mod pat;
mod stmt;

#[cfg(test)]
mod test;

use doomfront::parser::{OpenMark, Parser};

use crate::Syn;

pub use self::expr::*;

use self::{common::*, item::*, pat::*, stmt::*};

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

/// An inner annotation, item, or statement. If `ROOT`, statements are forbidden.
fn core_element<const ROOT: bool>(p: &mut Parser<Syn>) {
	if at_inner_annotation(p) {
		annotation(p, true);
		return;
	}

	let mark = p.open();

	let mut documented = false;

	while p.eat(Syn::DocComment, Syn::DocComment) && !p.eof() {
		trivia_0plus(p);
		documented = true;
	}

	while at_annotation(p) {
		annotation(p, false);
		trivia_0plus(p);
	}

	if at_function_decl(p) {
		function_decl(p, mark);
		return;
	}

	if at_symbolic_constant(p) {
		symbolic_constant(p, mark);
		return;
	}

	// Doc comments cannot precede anything below.
	if documented {
		p.advance_err_and_close(mark, p.nth(0), Syn::Error, &[&["TODO"]]);
		return;
	}

	if !ROOT {
		statement(p, mark);
		return;
	}

	p.advance_err_and_close(mark, p.nth(0), Syn::Error, &[&["TODO"]]);
}
