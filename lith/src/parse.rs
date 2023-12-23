//! Functions for parsing different elements of the Lithica syntax.

mod common;
mod expr;
mod pat;

#[cfg(test)]
mod test;

use doomfront::parser::Parser;

use crate::Syntax;

use self::common::*;

pub use self::expr::*;

pub type Error = doomfront::ParseError<Syntax>;

pub fn chunk(p: &mut Parser<Syntax>) {
	let root = p.open();

	while !p.eof() {
		if trivia(p) {
			continue;
		}

		// TODO: statements and inner annotations.
	}

	p.close(root, Syntax::Chunk);
}
