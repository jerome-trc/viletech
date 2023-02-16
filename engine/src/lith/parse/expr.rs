//! Expression parsers.

use doomfront::{
	chumsky::{primitive, Parser},
	comb,
	ext::{Parser1, ParserVec},
	ParseError, ParseOut,
};

use crate::lith::Syn;

use super::{common::*, lit::*, type_ref};

pub(super) fn expr(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + Clone + '_ {
	primitive::choice((
		literal(src),
		name(src),
		type_expr(src),
		// TODO: Unary, binary, ternary
	))
}

pub(super) fn type_expr(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + Clone + '_ {
	comb::just::<Syn, _>('@', Syn::At, src)
		.start_vec()
		.chain_push(comb::just::<Syn, _>('[', Syn::BracketL, src))
		.chain_append(wsp_ext(src).repeated())
		.chain_push(primitive::choice(
			// TODO: Array notation, tuple notation
			(type_ref(src),),
		))
		.chain_append(wsp_ext(src).repeated())
		.chain_push(comb::just::<Syn, _>(']', Syn::BracketR, src))
		.collect_n::<Syn, { Syn::ExprType as u16 }>()
}
