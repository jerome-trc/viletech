mod common;
mod expr;
mod stat;
mod structure;
mod top;

use chumsky::{primitive, IterParser, Parser};
use rowan::GreenNode;

use crate::{
	parser_t,
	zdoom::{self, Token},
};

use super::Syn;

/// Gives context to functions yielding parser combinators
/// (e.g. the user's selected ZScript version).
///
/// Thus, this information never has to be passed through deep call trees, and any
/// breaking changes to this context are minimal in scope.
#[derive(Debug, Clone)]
pub struct ParserBuilder {
	pub(self) _version: zdoom::Version,
}

impl ParserBuilder {
	#[must_use]
	pub fn new(version: zdoom::Version) -> Self {
		Self { _version: version }
	}

	/// The returned parser emits a [`Syn::Root`] node.
	pub fn file<'i>(&self) -> parser_t!(GreenNode) {
		// TODO: Single-class file syntax.

		primitive::choice((
			self.trivia(),
			self.const_def().map(|gnode| gnode.into()),
			self.enum_def().map(|gnode| gnode.into()),
			self.include_directive().map(|gnode| gnode.into()),
			self.version_directive().map(|gnode| gnode.into()),
		))
		.repeated()
		.collect::<Vec<_>>()
		.map(|elems| GreenNode::new(Syn::Root.into(), elems))
		.boxed()
	}
}
