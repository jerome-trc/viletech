//! Parsing functions relevant to other parts of the syntax that don't belong anywhere else.

use doomfront::parser::Parser;

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
