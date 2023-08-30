//! Functions for parsing different elements of the VZScript syntax.

mod common;
mod expr;
mod item;
mod stat;
mod structure;
#[cfg(test)]
mod test;

use doomfront::parser::Parser;

use crate::Syn;

use self::{common::*, item::*};

pub type Error = doomfront::ParseError<Syn>;

pub fn file(p: &mut Parser<Syn>) {
	let root = p.open();

	while !p.eof() {
		if trivia(p) {
			continue;
		}

		item(p);
	}

	p.close(root, Syn::FileRoot);
}
