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

use self::{common::*, item::*, structure::*};

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

/// Note that this also covers [annotations](Annotation).
fn item(p: &mut Parser<Syn>) {
	if p.at_any(Annotation::FIRST_SET) {
		Annotation::parse(p);
		return;
	}

	let mark = p.open();

	while p.eat(Syn::DocComment, Syn::DocComment) && !p.eof() {
		trivia_0plus(p);
	}

	while p.at_any(Attribute::FIRST_SET) && !p.eof() {
		Attribute::parse(p);
		trivia_0plus(p);
	}

	let token = p.nth(0);

	if FuncDecl::FIRST_SET.contains(&token) {
		FuncDecl::parse(p, mark);
	} else if ClassDef::FIRST_SET.contains(&token) {
		ClassDef::parse(p, mark);
	} else {
		p.advance_err_and_close(
			mark,
			token,
			Syn::Error,
			&["This is a placeholder error message!"],
		);
	}
}

#[must_use]
fn at_item(p: &mut Parser<Syn>) -> bool {
	p.at_any(Attribute::FIRST_SET) || p.at_any(FuncDecl::FIRST_SET)
}
