//! (G)ZDoom's [common scanner], re-implemented via combinators.
//!
//! [common scanner]: https://github.com/ZDoom/gzdoom/blob/master/src/common/engine/sc_man_scanner.re

use chumsky::{extra::ParserExtra, primitive, Parser};

use crate::comb;

/// C/C++-style floating-point literals.
pub fn lit_float<'i, E>() -> impl Parser<'i, &'i str, &'i str, E> + Clone
where
	E: ParserExtra<'i, &'i str>,
{
	let fl_exp = primitive::group((
		primitive::one_of(['e', 'E']),
		primitive::one_of(['+', '-']).or_not(),
		comb::dec_digit().repeated().at_least(1),
	));

	let fl_suffix = primitive::one_of(['f', 'F']);

	let no_point = primitive::group((
		comb::dec_digit().repeated().at_least(1),
		fl_exp.clone(),
		fl_suffix.clone().or_not(),
	))
	.slice();

	let l_opt = primitive::group((
		comb::dec_digit().repeated(),
		primitive::just('.'),
		comb::dec_digit().repeated().at_least(1),
		fl_exp.clone().or_not(),
		fl_suffix.clone().or_not(),
	))
	.slice();

	let r_opt = primitive::group((
		comb::dec_digit().repeated().at_least(1),
		primitive::just('.'),
		comb::dec_digit().repeated(),
		fl_exp.or_not(),
		fl_suffix.or_not(),
	))
	.slice();

	primitive::choice((no_point, l_opt, r_opt))
}

/// C/C++-style integer literals (`z` suffix excluded).
pub fn lit_int<'i, E>() -> impl Parser<'i, &'i str, &'i str, E> + Clone
where
	E: ParserExtra<'i, &'i str>,
{
	let hex = primitive::group((
		primitive::just('0'),
		primitive::one_of(['x', 'X']),
		comb::hex_digit().repeated().at_least(1),
		primitive::one_of(['u', 'U', 'l', 'L']).or_not(),
		primitive::one_of(['u', 'U', 'l', 'L']).or_not(),
	))
	.slice();

	let oct = primitive::group((
		primitive::just('0'),
		comb::oct_digit().repeated().at_least(1),
		primitive::one_of(['u', 'U', 'l', 'L']).or_not(),
		primitive::one_of(['u', 'U', 'l', 'L']).or_not(),
	))
	.slice();

	let dec = primitive::group((
		comb::dec_digit().repeated().at_least(1),
		primitive::one_of(['u', 'U', 'l', 'L']).or_not(),
		primitive::one_of(['u', 'U', 'l', 'L']).or_not(),
	))
	.slice();

	primitive::choice((hex, oct, dec))
}

/// (G)ZDoom "name" literals (used primarily by DECORATE and ZScript) are like
/// string literals but for special circumstances, like referring to classes.
/// They follow the RE2C regular expression `['] (any\[\n'])* [']`.
pub fn lit_name<'i, E>() -> impl Parser<'i, &'i str, &'i str, E> + Clone
where
	E: ParserExtra<'i, &'i str>,
{
	primitive::group((
		primitive::just('\''),
		primitive::none_of(['\n', '\'']).repeated(),
		primitive::just('\''),
	))
	.slice()
}

/// (G)ZDoom string literals follow the RE2C regular expression `["](([\\]["])|[^"])*["]`.
pub fn lit_string<'i, E>() -> impl Parser<'i, &'i str, &'i str, E> + Clone
where
	E: ParserExtra<'i, &'i str>,
{
	primitive::group((
		primitive::just('"'),
		primitive::choice((
			primitive::just("\\\"").slice(),
			primitive::none_of('"').slice(),
		))
		.repeated(),
		primitive::just('"'),
	))
	.slice()
}
