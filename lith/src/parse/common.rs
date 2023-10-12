//! Parsing functions relevant to other parts of the syntax that don't belong anywhere else.

use doomfront::parser::{CloseMark, OpenMark, Parser};

use crate::Syn;

/// May or may not build a token tagged with one of the following:
/// - [`Syn::Whitespace`]
/// - [`Syn::Comment`]
pub(super) fn trivia(p: &mut Parser<Syn>) -> bool {
	p.eat_any(&[
		(Syn::Whitespace, Syn::Whitespace),
		(Syn::Comment, Syn::Comment),
	])
}

/// Shorthand for `while trivia(p) {}`.
pub(super) fn trivia_0plus(p: &mut Parser<Syn>) {
	while trivia(p) {}
}

pub(super) fn _doc_comments(p: &mut Parser<Syn>) {
	while p.eat(Syn::DocComment, Syn::DocComment) {
		trivia_0plus(p);
	}
}

pub(super) fn block(
	p: &mut Parser<Syn>,
	mark: OpenMark,
	kind: Syn,
	allow_label: bool,
) -> CloseMark {
	if allow_label && at_block_label(p) {
		block_label(p);
	}

	p.expect(Syn::BraceL, Syn::BraceL, &[&["`{`"]]);
	trivia_0plus(p);

	while !p.eof() && !p.at(Syn::BraceR) {
		super::core_element::<false>(p);
		trivia_0plus(p);
	}

	p.expect(Syn::BraceR, Syn::BraceR, &[&["`}`"]]);
	p.close(mark, kind)
}

#[must_use]
pub(super) fn at_block_label(p: &Parser<Syn>) -> bool {
	p.at(Syn::Colon2)
}

pub(super) fn block_label(p: &mut Parser<Syn>) {
	let mark = p.open();
	p.expect(Syn::Colon2, Syn::Colon2, &[&["`::`"]]);
	trivia_0plus(p);
	p.expect(Syn::Ident, Syn::Ident, &[&["an identifier"]]);
	trivia_0plus(p);
	p.expect(Syn::Colon2, Syn::Colon2, &[&["`::`"]]);
	p.close(mark, Syn::BlockLabel);
}

#[must_use]
pub(super) fn at_type_spec(p: &Parser<Syn>) -> bool {
	p.at(Syn::Colon)
}

pub(super) fn type_spec(p: &mut Parser<Syn>) {
	let mark = p.open();
	p.expect(Syn::Colon, Syn::Colon, &[&["`:`"]]);
	trivia_0plus(p);
	let _ = super::expr(p);
	p.close(mark, Syn::TypeSpec);
}
