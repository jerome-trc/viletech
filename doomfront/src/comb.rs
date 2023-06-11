//! Various combinators which are broadly applicable elsewhere.

use chumsky::{primitive, Parser};
use rowan::{GreenToken, SyntaxKind};

use crate::parser_t;

pub fn green_token<'i, T>(
	syn: impl Into<SyntaxKind> + Copy,
) -> impl Clone + Fn(T, logos::Span, &mut &str) -> GreenToken
where
	T: 'i + logos::Logos<'i> + Eq + Copy,
{
	move |_, span, source: &mut &str| GreenToken::new(syn.into(), &source[span.start..span.end])
}

/// Shorthand for the following:
///
/// ```
/// primitive::just(token).map_with_state(green_token(token))
/// ```
pub fn just<'i, T>(token: T) -> parser_t!(T, GreenToken)
where
	T: 'i + logos::Logos<'i> + Into<SyntaxKind> + Eq + Copy,
{
	primitive::just(token).map_with_state(green_token(token))
}

/// "Just token-to-syntax". Like [`just`] but for languages which have a
/// different token type from their [`SyntaxKind`] type.
pub fn just_ts<'i, T, L>(token: T, syn: L) -> parser_t!(T, GreenToken)
where
	T: 'i + logos::Logos<'i> + Eq + Copy,
	L: 'i + Into<SyntaxKind> + Copy,
{
	primitive::just(token).map_with_state(green_token(syn))
}
