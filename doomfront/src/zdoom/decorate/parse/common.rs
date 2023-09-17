//! Combinators applicable to multiple other parts of the syntax.

use crate::{
	parser::Parser,
	zdoom::{decorate::Syn, Token},
};

/// Builds a token tagged [`Syn::Ident`]. Backed by [`is_ident_lax`].
pub(super) fn ident_lax(p: &mut Parser<Syn>) {
	p.expect_if(is_ident_lax, Syn::Ident, &[&["an identifier"]]);
}

/// Checks for a [`Token::Ident`] or an actor state qualifier keyword.
#[must_use]
pub(super) fn is_ident_lax(token: Token) -> bool {
	matches!(
		token,
		Token::Ident
			| Token::KwBright
			| Token::KwFast
			| Token::KwSlow
			| Token::KwNoDelay
			| Token::KwCanRaise
			| Token::KwOffset
			| Token::KwLight
	)
}

/// Builds a token tagged [`Syn::Ident`]. Backed by [`is_ident_xlax`].
pub(super) fn ident_xlax(p: &mut Parser<Syn>) {
	p.expect_if(is_ident_xlax, Syn::Ident, &[&["an identifier"]]);
}

/// Checks for a [`Token::Ident`] or *any* keyword.
#[must_use]
pub(super) fn is_ident_xlax(token: Token) -> bool {
	token == Token::Ident || token.is_keyword()
}

/// Builds a token tagged [`Syn::NonWhitespace`].
pub(super) fn non_whitespace(p: &mut Parser<Syn>) {
	p.merge(
		Syn::NonWhitespace,
		|token| !token.is_trivia() && token != Token::Colon,
		Syn::from,
		&[&["any non-whitespace"]],
	);
}

/// May or may not build a token tagged with one of the following:
/// - [`Syn::Whitespace`]
/// - [`Syn::Comment`]
/// - [`Syn::RegionStart`]
/// - [`Syn::RegionEnd`]
pub(super) fn trivia(p: &mut Parser<Syn>) -> bool {
	p.eat_any(&[
		(Token::Whitespace, Syn::Whitespace),
		(Token::Comment, Syn::Comment),
		(Token::DocComment, Syn::Comment),
		(Token::RegionStart, Syn::RegionStart),
		(Token::RegionEnd, Syn::RegionEnd),
	])
}

/// Shorthand for `while trivia(p) {}`.
pub(super) fn trivia_0plus(p: &mut Parser<Syn>) {
	while trivia(p) {}
}
