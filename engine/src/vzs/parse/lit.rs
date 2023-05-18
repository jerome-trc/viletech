//! Literal parsers.

use doomfront::{
	chumsky::{primitive, Parser},
	comb,
	util::{builder::GreenCache, state::*},
	Extra, ParseError,
};

use crate::vzs::Syn;

/// Builds a [`Syn::Literal`] node.
pub(super) fn literal<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> + Clone
{
	primitive::group((
		primitive::empty().map_with_state(gtb_open(Syn::Literal.into())),
		primitive::choice((lit_string(), lit_float(), lit_bool(), lit_int(), lit_void())),
	))
	.map_with_state(gtb_close())
	.map_err_with_state(gtb_cancel(Syn::Literal.into()))
}

/// Builds a [`Syn::LitVoid`] token.
pub(super) fn lit_void<'i, C: 'i + GreenCache>(
) -> impl Parser<'i, &'i str, (), Extra<'i, C>> + Clone {
	primitive::just("()").map_with_state(gtb_token(Syn::LitVoid.into()))
}

/// Builds a [`Syn::LitFalse`] or [`Syn::LitTrue`] token.
pub(super) fn lit_bool<'i, C: 'i + GreenCache>(
) -> impl Parser<'i, &'i str, (), Extra<'i, C>> + Clone {
	primitive::choice((
		primitive::just("false").map_with_state(gtb_token(Syn::LitFalse.into())),
		primitive::just("true").map_with_state(gtb_token(Syn::LitTrue.into())),
	))
}

// Numeric /////////////////////////////////////////////////////////////////////

fn lit_dec<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, &'i str, Extra<'i, C>> + Clone {
	primitive::group((
		comb::dec_digit(),
		primitive::choice((comb::dec_digit(), primitive::just('_'))).repeated(),
	))
	.slice()
}

/// Builds a [`Syn::LitFloat`] token.
pub(super) fn lit_float<'i, C: 'i + GreenCache>(
) -> impl Parser<'i, &'i str, (), Extra<'i, C>> + Clone {
	let after_notail = primitive::none_of(['.', '_'])
		.filter(|i| !unicode_ident::is_xid_start(*i))
		.map(|_| ())
		.or(primitive::end());

	let notail = primitive::group((lit_dec(), primitive::just('.')))
		.then_ignore(after_notail.rewind())
		.slice();

	let tail = primitive::group((lit_dec(), primitive::just('.'), lit_dec())).slice();

	let exp = primitive::group((
		primitive::one_of("eE"),
		primitive::one_of("+-").or_not(),
		primitive::one_of("0123456789_")
			.repeated()
			.at_least(1)
			.slice()
			.validate(|text: &str, span, emitter| {
				if !text.chars().any(|c| c.is_ascii_digit()) {
					emitter.emit(ParseError::custom(
						span,
						"float exponent has no digits".to_string(),
					));
				}

				text
			}),
	));

	let w_exp = primitive::group((
		lit_dec(),
		primitive::group((primitive::just('.'), lit_dec())).or_not(),
		exp,
	))
	.slice();

	primitive::choice((w_exp, tail, notail)).map_with_state(gtb_token(Syn::LitFloat.into()))
}

/// Builds a [`Syn::LitInt`] token.
pub(super) fn lit_int<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> + Clone
{
	let bin = primitive::group((
		primitive::just("0b"),
		primitive::one_of("01_")
			.repeated()
			.at_least(1)
			.slice()
			.validate(|text: &str, span, emitter| {
				if !text.chars().any(|c| c.is_ascii_digit()) {
					emitter.emit(ParseError::custom(
						span,
						"binary literal has no digits".to_string(),
					));
				}

				text
			}),
	))
	.slice();

	let hex = primitive::group((
		primitive::just("0x"),
		primitive::one_of("0123456789abcdefABCDEF_")
			.repeated()
			.at_least(1)
			.slice()
			.validate(|text: &str, span, emitter| {
				if !text.chars().any(|c| c.is_ascii_digit()) {
					emitter.emit(ParseError::custom(
						span,
						"hexadecimal literal has no digits".to_string(),
					));
				}

				text
			}),
	))
	.slice();

	let oct = primitive::group((
		primitive::just("0o"),
		primitive::one_of("01234567_")
			.repeated()
			.at_least(1)
			.slice()
			.validate(|text: &str, span, emitter| {
				if !text.chars().any(|c| c.is_ascii_digit()) {
					emitter.emit(ParseError::custom(
						span,
						"octal literal has no digits".to_string(),
					));
				}

				text
			}),
	))
	.slice();

	primitive::choice((bin, hex, oct, lit_dec())).map_with_state(gtb_token(Syn::LitInt.into()))
}

// String //////////////////////////////////////////////////////////////////////

/// Builds a [`Syn::LitString`] token.
pub(super) fn lit_string<'i, C: 'i + GreenCache>(
) -> impl Parser<'i, &'i str, (), Extra<'i, C>> + Clone {
	let inner = primitive::none_of("\"\\").slice();

	let strcont = primitive::just('\\').then(primitive::just('\n')).slice();

	primitive::group((
		primitive::just('"'),
		primitive::choice((
			inner,
			quote_escape(),
			ascii_escape(),
			unicode_escape(),
			strcont,
		))
		.repeated(),
		primitive::just('"'),
	))
	.slice()
	.map_with_state(gtb_token(Syn::LitString.into()))
}

fn ascii_escape<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, &'i str, Extra<'i, C>> + Clone
{
	primitive::choice((
		primitive::group((primitive::just("\\x"), comb::oct_digit(), comb::hex_digit())).slice(),
		primitive::just("\\n"),
		primitive::just("\\r"),
		primitive::just("\\t"),
		primitive::just("\\\\"),
		primitive::just("\\0"),
	))
	.slice()
}

fn quote_escape<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, &'i str, Extra<'i, C>> + Clone
{
	primitive::choice((primitive::just("\\'"), primitive::just("\\\""))).slice()
}

fn unicode_escape<'i, C: 'i + GreenCache>(
) -> impl Parser<'i, &'i str, &'i str, Extra<'i, C>> + Clone {
	primitive::group((
		primitive::just("\\u{"),
		primitive::group((comb::hex_digit(), primitive::just('_').or_not()))
			.repeated()
			.at_least(1)
			.at_most(6),
		primitive::just("}"),
	))
	.slice()
}

/*

pub(super) fn lit_char(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + Clone + '_ {
	let inner = primitive::none_of("'\\\n\r\t").map(|_| ());

	primitive::just('\'')
		.then(primitive::choice((
			inner,
			quote_escape(),
			ascii_escape(),
			unicode_escape(),
		)))
		.then(primitive::just('\''))
		.map_with_span(help::map_tok::<Syn, _>(src, Syn::LitChar))
}

pub(super) fn lit_null(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + Clone + '_ {
	comb::just::<Syn, _>("null", Syn::LitNull, src)
}

*/
