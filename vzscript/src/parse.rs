//! Functions for parsing different elements of the VZScript syntax.

mod common;
mod expr;
mod item;
mod stat;
mod structure;
#[cfg(test)]
mod test;

use doomfront::parser::{OpenMark, Parser};

use crate::Syn;

use self::{common::*, item::*, stat::*, structure::*};

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

/// An annotation, item, or statement. If `ROOT`, statements are forbidden.
fn core_element<const ROOT: bool>(p: &mut Parser<Syn>) {
	if p.at_any(Annotation::FIRST_SET) {
		Annotation::parse(p);
		return;
	}

	let mark = p.open();
	let mut parsed_docs = false;

	while p.eat(Syn::DocComment, Syn::DocComment) && !p.eof() {
		parsed_docs = true;
		trivia_0plus(p);
	}

	Attribute::parse_0plus(p);

	if parsed_docs || at_item(p) {
		item(p, mark);
	} else if !ROOT {
		statement(p, mark);
	} else {
		p.advance_err_and_close(
			mark,
			p.nth(0),
			Syn::Error,
			&[&["This is a placeholder error message!"]],
		);
	}
}

fn item(p: &mut Parser<Syn>, mark: OpenMark) {
	let token = p.nth(0);

	if FuncDecl::FIRST_SET.contains(&token) {
		FuncDecl::parse(p, mark);
	} else if ClassDef::FIRST_SET.contains(&token) {
		ClassDef::parse(p, mark);
	} else if ConstDef::FIRST_SET.contains(&token) {
		ConstDef::parse(p, mark);
	} else if MixinDef::FIRST_SET.contains(&token) {
		MixinDef::parse(p, mark);
	} else {
		p.advance_err_and_close(
			mark,
			token,
			Syn::Error,
			&[&["This is a placeholder error message!"]],
		);
	}
}

#[must_use]
fn at_item(p: &Parser<Syn>) -> bool {
	p.at_any(FuncDecl::FIRST_SET)
		|| p.at_any(ClassDef::FIRST_SET)
		|| p.at_any(ConstDef::FIRST_SET)
		|| p.at_any(MixinDef::FIRST_SET)
}
