//! Functions for parsing different elements of the Lithica syntax.

mod common;
mod expr;
mod pat;
mod stmt;

#[cfg(test)]
mod test;

use doomfront::parser::Parser;

use crate::Syntax;

use self::{common::*, pat::*, stmt::*};

pub use self::expr::*;

pub type Error = doomfront::ParseError<Syntax>;

pub fn chunk(p: &mut Parser<Syntax>) {
	let root = p.open();

	while !p.eof() {
		if trivia(p) {
			continue;
		}

		// TODO: inner annotations

		let mark = p.open();

		while p.eat(Syntax::DocComment, Syntax::DocComment) && !p.eof() {
			trivia_0plus(p);
		}

		statement(p, mark);
	}

	p.close(root, Syntax::Chunk);
}
