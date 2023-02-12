//! Expression parsers.

use doomfront::{
	chumsky::{primitive, Parser},
	comb, help, ParseError, ParseOut,
};

use crate::lith::Syn;

use super::{common::*, lit::*, type_ref};

pub(super) fn expr(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	primitive::choice((
		literal(src),
		name(src),
		type_expr(src),
		// TODO: Unary, binary, ternary
	))
}

pub(super) fn type_expr(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	comb::just::<Syn>("@", Syn::At)
		.map(help::map_nvec())
		.then(comb::just::<Syn>("[", Syn::LBracket))
		.map(help::map_push())
		.then(wsp_ext(src).or_not())
		.map(help::map_push_opt())
		.then(type_ref(src))
		.map(help::map_push())
		.then(wsp_ext(src).or_not())
		.map(help::map_push_opt())
		.then(comb::just::<Syn>("]", Syn::RBracket))
		.map(help::map_push())
		.map(help::map_collect::<Syn>(Syn::ExprType))
}
