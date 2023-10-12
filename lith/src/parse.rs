//! Functions for parsing different elements of the Lithica syntax.

mod common;
mod expr;
mod item;

#[cfg(test)]
mod test;

use doomfront::parser::Parser;

use crate::Syn;

pub use self::expr::*;

use self::{common::*, item::*};

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
	// TODO: if at an inner annotation, parse it and return here.

	let mark = p.open();
	let mut parsed_docs = false;

	while p.eat(Syn::DocComment, Syn::DocComment) && !p.eof() {
		parsed_docs = true;
		trivia_0plus(p);
	}

	// TODO: parse 0 or more outer annotations here.

	if parsed_docs || at_function_decl(p) {
		function_decl(p, mark);
	} else if !ROOT {
		todo!("statement parsing")
	} else {
		p.advance_err_and_close(mark, p.nth(0), Syn::Error, &[&["TODO"]]);
	}
}
