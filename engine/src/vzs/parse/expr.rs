//! Expression parsers.

use doomfront::{
	chumsky::{primitive, Parser},
	util::builder::GreenCache,
	Extra,
};

use super::{common::*, lit::*};

/// Builds a node.
pub(super) fn expr<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> + Clone {
	doomfront::chumsky::recursive::recursive(|_| primitive::choice((literal(), resolver())))
}
