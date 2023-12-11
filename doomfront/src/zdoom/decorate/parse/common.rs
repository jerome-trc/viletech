//! Combinators applicable to multiple other parts of the syntax.

use crate::{
	parser::Parser,
	zdoom::{decorate::Syntax, Token},
};

/// Builds a token tagged [`Syntax::Ident`]. Backed by [`is_ident_lax`].
pub(super) fn ident_lax(p: &mut Parser<Syntax>) {
	p.expect_if(is_ident_lax, Syntax::Ident, &[&["an identifier"]]);
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

/// Builds a token tagged [`Syntax::Ident`]. Backed by [`is_ident_xlax`].
pub(super) fn ident_xlax(p: &mut Parser<Syntax>) {
	p.expect_if(is_ident_xlax, Syntax::Ident, &[&["an identifier"]]);
}

/// Checks for a [`Token::Ident`] or *any* keyword.
#[must_use]
pub(super) fn is_ident_xlax(token: Token) -> bool {
	token == Token::Ident || token.is_keyword()
}

/// Builds a token tagged [`Syntax::NonWhitespace`].
pub(super) fn non_whitespace(p: &mut Parser<Syntax>) {
	p.merge(
		Syntax::NonWhitespace,
		|token| !token.is_trivia() && token != Token::Colon,
		Syntax::from,
		&[&["any non-whitespace"]],
	);
}

/// May or may not build a token tagged with one of the following:
/// - [`Syntax::Whitespace`]
/// - [`Syntax::Comment`]
/// - [`Syntax::RegionStart`]
/// - [`Syntax::RegionEnd`]
pub(super) fn trivia(p: &mut Parser<Syntax>) -> bool {
	p.eat_any(&[
		(Token::Whitespace, Syntax::Whitespace),
		(Token::Comment, Syntax::Comment),
		(Token::DocComment, Syntax::Comment),
		(Token::RegionStart, Syntax::RegionStart),
		(Token::RegionEnd, Syntax::RegionEnd),
	])
}

/// Shorthand for `while trivia(p) {}`.
pub(super) fn trivia_0plus(p: &mut Parser<Syntax>) {
	while trivia(p) {}
}
