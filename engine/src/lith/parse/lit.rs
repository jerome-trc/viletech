//! Literal parsers.

use doomfront::{
	chumsky::{primitive, Parser},
	comb, help, ParseError, ParseOut,
};

use crate::lith::Syn;

pub(super) fn literal(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	primitive::choice((
		lit_char(src),
		lit_string(src),
		lit_float(src),
		lit_int(src),
		lit_null(),
		lit_bool(),
	))
	.map(help::map_node::<Syn>(Syn::Literal))
}

pub(super) fn lit_bool() -> impl Parser<char, ParseOut, Error = ParseError> {
	primitive::choice((
		comb::just::<Syn>("false", Syn::LitFalse),
		comb::just::<Syn>("true", Syn::LitTrue),
	))
}

pub(super) fn lit_char(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
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

pub(super) fn lit_float(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	let notail = lit_dec()
		.then(primitive::just("."))
		.then_ignore(
			primitive::filter(|c| *c != '.' && *c != '_' && !unicode_ident::is_xid_start(*c))
				.rewind(),
		)
		.map(|_| ());

	let tail = lit_dec()
		.then(primitive::just("."))
		.then(lit_dec())
		.map(|_| ());

	let exp = primitive::one_of("eE")
		.then(primitive::one_of("+-").or_not())
		.then(
			primitive::one_of("0123456789_")
				.repeated()
				.at_least(1)
				.validate(|chars, span, emit| {
					if !chars.iter().any(|c: &char| c.is_ascii_digit()) {
						emit(ParseError::custom(
							span,
							"Float exponent has no digits.".to_string(),
						));
					}

					chars
				}),
		)
		.map(|_| ());

	let w_exp = lit_dec()
		.then((primitive::just(".").then(lit_dec())).or_not())
		.then(exp)
		.map(|_| ());

	primitive::choice((w_exp, tail, notail))
		.map_with_span(help::map_tok::<Syn, _>(src, Syn::LitFloat))
}

pub(super) fn lit_int(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	let dec = lit_dec().map(|_| ());

	let bin = primitive::just("0b")
		.then(
			primitive::one_of("01_")
				.repeated()
				.at_least(1)
				.validate(|chars, span, emit| {
					if !chars.iter().any(|c: &char| c.is_ascii_digit()) {
						emit(ParseError::custom(
							span,
							"Binary literal has no digits.".to_string(),
						));
					}

					chars
				}),
		)
		.map(|_| ());

	let oct = primitive::just("0o")
		.then(
			primitive::one_of("01234567_")
				.repeated()
				.at_least(1)
				.validate(|chars, span, emit| {
					if !chars.iter().any(|c: &char| c.is_ascii_digit()) {
						emit(ParseError::custom(
							span,
							"Octal literal has no digits.".to_string(),
						));
					}

					chars
				}),
		)
		.map(|_| ());

	let hex = primitive::just("0x")
		.then(
			primitive::one_of("0123456789abcdefABCDEF_")
				.repeated()
				.at_least(1)
				.validate(|chars, span, emit| {
					if !chars.iter().any(|c: &char| c.is_ascii_alphanumeric()) {
						emit(ParseError::custom(
							span,
							"Hexadecimal literal has no digits.".to_string(),
						));
					}

					chars
				}),
		)
		.map(|_| ());

	primitive::choice((bin, oct, hex, dec)).map_with_span(help::map_tok::<Syn, _>(src, Syn::LitInt))
}

fn lit_dec() -> impl Parser<char, (), Error = ParseError> {
	primitive::one_of("0123456789")
		.then(primitive::one_of("0123456789_").repeated())
		.map(|_| ())
}

pub(super) fn lit_null() -> impl Parser<char, ParseOut, Error = ParseError> {
	comb::just::<Syn>("null", Syn::LitNull)
}

pub(super) fn lit_string(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	let inner = primitive::none_of("\"\\").map(|_| ());
	let strcont = primitive::just('\\')
		.then(primitive::just('\n'))
		.map(|_| ());

	primitive::just('"')
		.then(
			primitive::choice((
				inner,
				quote_escape(),
				ascii_escape(),
				unicode_escape(),
				strcont,
			))
			.repeated(),
		)
		.then(primitive::just('"'))
		.map_with_span(help::map_tok::<Syn, _>(src, Syn::LitString))
}

fn ascii_escape() -> impl Parser<char, (), Error = ParseError> {
	primitive::choice((
		primitive::just("\\x")
			.then(comb::oct_digit())
			.then(comb::hex_digit())
			.map(|_| ()),
		primitive::just("\\n").map(|_| ()),
		primitive::just("\\r").map(|_| ()),
		primitive::just("\\t").map(|_| ()),
		primitive::just("\\\\").map(|_| ()),
		primitive::just("\\0").map(|_| ()),
	))
	.map(|_| ())
}

fn quote_escape() -> impl Parser<char, (), Error = ParseError> {
	primitive::choice((primitive::just("\\'"), primitive::just("\\\""))).map(|_| ())
}

fn unicode_escape() -> impl Parser<char, (), Error = ParseError> {
	let digit = comb::hex_digit().then(primitive::just('_').or_not());

	primitive::just("\\u{")
		.then(digit.repeated().at_least(1).at_most(6))
		.then(primitive::just("}"))
		.map(|_| ())
}
