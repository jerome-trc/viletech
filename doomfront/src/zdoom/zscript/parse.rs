mod actor;
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
	GreenElement,
};

use super::Syn;

pub use self::{actor::*, common::*, expr::*, stat::*, structure::*, top::*};

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
			self.class_def().map(GreenElement::from),
			self.struct_def().map(GreenElement::from),
			self.enum_def().map(GreenElement::from),
			self.const_def().map(GreenElement::from),
			self.class_extend().map(GreenElement::from),
			self.struct_extend().map(GreenElement::from),
			self.mixin_class_def().map(GreenElement::from),
			self.include_directive().map(GreenElement::from),
			self.version_directive().map(GreenElement::from),
		))
		.repeated()
		.collect::<Vec<_>>()
		.map(|elems| GreenNode::new(Syn::Root.into(), elems))
		.boxed()
	}
}

pub fn file(p: &mut crate::parser::Parser<Syn>) {
	let root = p.open();

	while !p.eof() {
		if trivia(p) {
			continue;
		}

		match p.nth(0) {
			Token::KwConst => const_def(p),
			Token::KwEnum => enum_def(p),
			Token::PoundInclude => include_directive(p),
			Token::KwVersion => version_directive(p),
			_ => p.advance_with_error(
				Syn::from(p.nth(0)),
				&[
					"`const`",
					"`enum`",
					"`class`",
					"`struct`",
					"`mixin`",
					"`extend`",
					"`#include`",
					"`version`",
				],
			),
		}
	}

	p.close(root, Syn::Root);
}

#[cfg(test)]
#[cfg(any())]
mod test {
	use crate::{
		testing::*,
		zdoom::{zscript::ParseTree, Version},
	};

	use super::*;

	#[test]
	fn with_sample_data() {
		let (_, sample) = match read_sample_data("DOOMFRONT_ZSCRIPT_SAMPLE") {
			Ok(s) => s,
			Err(err) => {
				eprintln!("Skipping ZScript sample data-based unit test. Reason: {err}");
				return;
			}
		};

		let parser = ParserBuilder::new(Version::default()).file();
		let tbuf = crate::_scan(&sample, zdoom::Version::default());
		let result = crate::_parse(parser, &sample, &tbuf);
		let ptree: ParseTree = unwrap_parse_tree(result);
		assert_no_errors(&ptree);
	}
}
