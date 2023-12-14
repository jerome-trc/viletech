//! Parsing functions relevant to other parts of the syntax that don't belong anywhere else.

use doomfront::parser::Parser;

use crate::Syntax;

/// May or may not build a token tagged with one of the following:
/// - [`Syntax::Whitespace`]
/// - [`Syntax::Comment`]
pub(super) fn trivia(p: &mut Parser<Syntax>) -> bool {
	p.eat_any(&[
		(Syntax::Whitespace, Syntax::Whitespace),
		(Syntax::Comment, Syntax::Comment),
	])
}

/// Shorthand for `while trivia(p) {}`.
pub(super) fn trivia_0plus(p: &mut Parser<Syntax>) {
	while trivia(p) {}
}

pub(super) fn doc_comments(p: &mut Parser<Syntax>) {
	while p.eat(Syntax::DocComment, Syntax::DocComment) {
		trivia_0plus(p);
	}
}

#[must_use]
pub(super) fn at_block_label(p: &Parser<Syntax>) -> bool {
	p.at(Syntax::Colon2)
}

pub(super) fn block_label(p: &mut Parser<Syntax>) {
	let mark = p.open();
	p.expect(Syntax::Colon2, Syntax::Colon2, &[&["`::`"]]);
	trivia_0plus(p);
	p.expect(Syntax::Ident, Syntax::Ident, &[&["an identifier"]]);
	trivia_0plus(p);
	p.expect(Syntax::Colon2, Syntax::Colon2, &[&["`::`"]]);
	p.close(mark, Syntax::BlockLabel);
}
