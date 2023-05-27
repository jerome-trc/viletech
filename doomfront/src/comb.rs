//! Various combinators which are broadly applicable elsewhere.

use chumsky::{error::Error as ChumskyError, extra::ParserExtra, primitive, text, Parser};
use rowan::SyntaxKind;

use crate::{
	util::{builder::GreenCache, state::*},
	Extra,
};

/// Shorthand for
/// `primitive::one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ")`,
/// since writing it out manually is error-prone.
pub fn ascii_letter<'i, E>() -> impl Parser<'i, &'i str, char, E> + Clone
where
	E: ParserExtra<'i, &'i str>,
{
	primitive::one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ")
}

/// For ASCII-case-insensitive keywords used by (G)ZDoom's DSLs.
///
/// Chumsky offers no good singular combinator for dealing with these, so
/// doomfront brings its own. Note that it can also be used for tokens like `#include`.
pub fn kw_nc<'i, E>(string: &'static str) -> impl Parser<'i, &'i str, &'i str, E> + Clone
where
	E: ParserExtra<'i, &'i str>,
{
	primitive::any::<&'i str, E>()
		.repeated()
		.exactly(string.len())
		.slice()
		.try_map(move |s, span| {
			s.eq_ignore_ascii_case(string)
				.then_some(s)
				.ok_or_else(|| E::Error::expected_found(None, None, span))
		})
}

/// Shorthand for `chumsky::primitive::one_of("0123456789")`.
pub fn dec_digit<'i, E>() -> impl Parser<'i, &'i str, char, E> + Clone
where
	E: ParserExtra<'i, &'i str>,
{
	primitive::one_of("0123456789")
}

/// Shorthand for `chumsky::primitive::one_of("0123456789abcdefABCDEF")`.
pub fn hex_digit<'i, E>() -> impl Parser<'i, &'i str, char, E> + Clone
where
	E: ParserExtra<'i, &'i str>,
{
	primitive::one_of("0123456789abcdefABCDEF")
}

/// Shorthand for `chumsky::primitive::one_of("01234567")`.
pub fn oct_digit<'i, E>() -> impl Parser<'i, &'i str, char, E> + Clone
where
	E: ParserExtra<'i, &'i str>,
{
	primitive::one_of("01234567")
}

/// Shorthand for the following idiom:
///
/// ```
/// primitive::empty()
///     .map_with_state(gtb_open(kind))
///     .then(primitive::group((
///         parser1,
///         parser2,
///         ...
///     )))
///     .map_with_state(gtb_close())
///     .map_err_with_state(gtb_cancel(kind))
/// ```
pub fn node<'i, O, C: GreenCache>(
	kind: SyntaxKind,
	group: impl Parser<'i, &'i str, O, Extra<'i, C>> + Clone,
) -> impl Parser<'i, &'i str, (), Extra<'i, C>> + Clone {
	primitive::empty()
		.map_with_state(gtb_open(kind))
		.then(group)
		.map_with_state(gtb_close())
		.map_err_with_state(gtb_cancel(kind))
}

/// Shorthand for the following idiom:
///
/// ```
/// primitive::empty()
///     .map_with_state(gtb_checkpoint())
///     .then(primitive::group((
///         parser1,
///         parser2,
///         ...
///     )))
///     .map(|_| ())
///     .map_err_with_state(gtb_cancel_checkpoint())
/// ```
pub fn checkpointed<'i, O, C: GreenCache>(
	group: impl Parser<'i, &'i str, O, Extra<'i, C>> + Clone,
) -> impl Parser<'i, &'i str, (), Extra<'i, C>> + Clone {
	primitive::empty()
		.map_with_state(gtb_checkpoint())
		.then(group)
		.map(|_| ())
		.map_err_with_state(gtb_cancel_checkpoint())
}

// Whitespace, comments ////////////////////////////////////////////////////////

/// The most common kind of whitespace;
/// spaces, carriage returns, newlines, and/or tabs, repeated one or more times.
pub fn wsp<'i, E>() -> impl Parser<'i, &'i str, &'i str, E> + Clone
where
	E: ParserExtra<'i, &'i str>,
{
	primitive::one_of([' ', '\r', '\n', '\t'])
		.repeated()
		.at_least(1)
		.slice()
}

/// Multi-line comments delimited by `/*` and `*/`.
/// Used by ACS and (G)ZDoom's languages.
pub fn c_comment<'i, E>() -> impl Parser<'i, &'i str, &'i str, E> + Clone
where
	E: ParserExtra<'i, &'i str>,
{
	primitive::just("/*")
		.then(
			primitive::any()
				.and_is(primitive::just("*/").not())
				.repeated(),
		)
		.then(primitive::just("*/"))
		.slice()
}

/// Single-line comments delimited by `//`. Used by ACS and (G)ZDoom's languages.
pub fn cpp_comment<'i, E>() -> impl Parser<'i, &'i str, &'i str, E> + Clone
where
	E: ParserExtra<'i, &'i str>,
{
	primitive::just("//")
		.then(primitive::any().and_is(text::newline().not()).repeated())
		.slice()
}
