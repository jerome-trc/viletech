//! Combinators applicable to multiple other parts of the syntax.

use chumsky::{primitive, text, IterParser, Parser};

use crate::{
	comb,
	util::{builder::GreenCache, state::*},
	zdoom::{decorate::Syn, lexer::*},
	Extra,
};

/// Matches ASCII case-insensitively. Includes `true` and `false`.
#[must_use]
pub(super) fn is_any_common_keyword(string: &str) -> bool {
	string.eq_ignore_ascii_case("actor")
		|| string.eq_ignore_ascii_case("const")
		|| string.eq_ignore_ascii_case("do")
		|| string.eq_ignore_ascii_case("enum")
		|| string.eq_ignore_ascii_case("false")
		|| string.eq_ignore_ascii_case("for")
		|| string.eq_ignore_ascii_case("states")
		|| string.eq_ignore_ascii_case("true")
		|| string.eq_ignore_ascii_case("while")
}

pub(super) fn _any_state_keyword<'i, C: 'i + GreenCache>(
) -> impl Parser<'i, &'i str, &'i str, Extra<'i, C>> + Clone {
	primitive::choice((
		comb::kw_nc("fail"),
		comb::kw_nc("loop"),
		comb::kw_nc("stop"),
		comb::kw_nc("wait"),
		comb::kw_nc("goto"),
	))
}

/// DECORATE actor class identifiers are allowed to consist solely of ASCII digits.
/// Note that this filters out non-contextual keywords.
pub(super) fn actor_ident<'i, C: 'i + GreenCache>(
) -> impl Parser<'i, &'i str, &'i str, Extra<'i, C>> + Clone {
	primitive::choice((
		comb::ascii_letter(),
		comb::dec_digit(),
		primitive::just('_'),
	))
	.repeated()
	.at_least(1)
	.slice()
	.filter(|&s| !is_any_common_keyword(s))
}

/// Shorthand for `chumsky::text::ident().filter(|&s| !is_any_common_keyword(s))`.
pub(super) fn ident<'i, C: 'i + GreenCache>(
) -> impl Parser<'i, &'i str, &'i str, Extra<'i, C>> + Clone {
	text::ident().filter(|&s| !is_any_common_keyword(s))
}

/// `ident (whitespace* '.' ident)*`
pub(super) fn ident_chain<'i, C: 'i + GreenCache>(
) -> impl Parser<'i, &'i str, (), Extra<'i, C>> + Clone {
	ident()
		.map_with_state(gtb_open_with(Syn::IdentChain.into(), Syn::Ident.into()))
		.then(
			primitive::group((
				primitive::empty().map_with_state(gtb_checkpoint()),
				wsp_ext().repeated().collect::<()>(),
				primitive::just(".").map_with_state(gtb_token(Syn::Period.into())),
				ident().map_with_state(gtb_token(Syn::Ident.into())),
			))
			.map_err_with_state(gtb_cancel_checkpoint())
			.repeated()
			.collect::<()>(),
		)
		.map_with_state(gtb_close())
		.map_err_with_state(gtb_cancel(Syn::IdentChain.into()))
}

/// In certain contexts, DECORATE allows providing number literals but not
/// expressions, so "negative literals" are required in place of unary negation.
pub(super) fn lit_int_negative<'i, C: 'i + GreenCache>(
) -> impl Parser<'i, &'i str, &'i str, Extra<'i, C>> + Clone {
	primitive::group((primitive::just("-"), lit_int())).slice()
}

/// In certain contexts, DECORATE allows providing number literals but not
/// expressions, so "negative literals" are required in place of unary negation.
pub(super) fn lit_float_negative<'i, C: 'i + GreenCache>(
) -> impl Parser<'i, &'i str, &'i str, Extra<'i, C>> + Clone {
	primitive::group((primitive::just("-"), lit_float())).slice()
}

/// Remember that this matches either a heterogenous whitespace span or a comment,
/// so to deal with both in one span requires repetition.
pub(super) fn wsp_ext<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> + Clone
{
	primitive::choice((
		comb::cpp_comment().map_with_state(gtb_token(Syn::Comment.into())),
		comb::c_comment().map_with_state(gtb_token(Syn::Comment.into())),
		comb::wsp().map_with_state(gtb_token(Syn::Whitespace.into())),
	))
}

/// A subset of [`wsp_ext`]. Only C comments, spaces, or tabs.
pub(super) fn wsp_1line<'i, C: 'i + GreenCache>(
) -> impl Parser<'i, &'i str, (), Extra<'i, C>> + Clone {
	primitive::choice((
		comb::c_comment().map_with_state(gtb_token(Syn::Comment.into())),
		primitive::one_of([' ', '\t'])
			.repeated()
			.at_least(1)
			.slice()
			.map_with_state(gtb_token(Syn::Whitespace.into())),
	))
}
