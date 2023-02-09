//! Expression parsers.

use doomfront::{
	chumsky::{primitive, Parser},
	rowan::GreenNode,
	ParseError, ParseOut,
};

use crate::lith::Syn;

use super::common::*;

#[must_use]
pub(super) fn type_expr(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	primitive::choice((resolver(src),))
		.map(|n_or_t| ParseOut::Node(GreenNode::new(Syn::ExprType.into(), [n_or_t])))
}
