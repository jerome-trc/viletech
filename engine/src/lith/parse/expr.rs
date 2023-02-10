//! Expression parsers.

use doomfront::{
	chumsky::{primitive, Parser},
	help, ParseError, ParseOut,
};

use crate::lith::Syn;

use super::common::*;

pub(super) fn expr(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	primitive::choice((
		atom(src),
		// TODO: Unary, binary, ternary
	))
	.map(help::map_node::<Syn>(Syn::Expression))
}

fn atom(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	primitive::choice((
		type_expr(src),
		ident(src),
		// TODO: Literals
	))
}

#[must_use]
pub(super) fn type_expr(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	primitive::choice((resolver(src),)).map(help::map_node::<Syn>(Syn::ExprType))
}
