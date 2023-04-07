//! Expression parsers.

use doomfront::{
	chumsky::{primitive, Parser},
	ParseError, ParseOut,
};

use super::{common::*, lit::*};

pub(super) fn expr(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + Clone + '_ {
	primitive::choice((literal(src), name(src)))
}
