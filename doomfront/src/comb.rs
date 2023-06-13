//! Various combinators which are broadly applicable elsewhere.

use chumsky::{primitive, Parser};
use rowan::{GreenToken, SyntaxKind};

use crate::{parser_t, ParseError, ParseState};

pub fn green_token<'i, T>(
	syn: impl Into<SyntaxKind> + Copy,
) -> impl Clone + Fn(T, logos::Span, &mut ParseState<'i>) -> GreenToken
where
	T: 'i + logos::Logos<'i> + Eq + Copy,
{
	move |_, span, state: &mut ParseState| GreenToken::new(syn.into(), &state.source[span])
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

/// Like [`just_ts`], but only matches `token` as long as it holds `string`,
/// ASCII case-insensitively.
///
/// This is needed for (G)ZDoom DSLs, many of which are unspecified and use only an
/// ad-hoc parser as DoomFront's reference implementation. Representing every niche
/// keyword used by every one of these languages would add complexity to every parser
/// (since each would have to treat foreign keywords as identifiers), so instead
/// make the smaller languages look for their keywords through identifiers.
pub fn string_nc<'i, T, L>(token: T, string: &'static str, syn: L) -> parser_t!(T, GreenToken)
where
	T: 'i + logos::Logos<'i> + Eq + Copy,
	L: 'i + Into<SyntaxKind> + Copy,
{
	primitive::just(token).try_map_with_state(
		move |_, span: logos::Span, state: &mut ParseState<'i>| {
			let text: &str = &state.source[span.clone()];

			if text.eq_ignore_ascii_case(string) {
				Ok(GreenToken::new(syn.into(), text))
			} else {
				Err(ParseError::custom(
					span,
					format!("expected `{string}`, found `{text}`"),
				))
			}
		},
	)
}
