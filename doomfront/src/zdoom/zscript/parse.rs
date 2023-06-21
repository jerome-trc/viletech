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
			Token::KwClass => class_def(p),
			Token::KwStruct => struct_def(p),
			Token::KwMixin => mixin_class_def(p),
			Token::KwExtend => class_or_struct_extend(p),
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
mod test {
	use std::borrow::Cow;

	use crate::{
		testing::*,
		zdoom::{
			self,
			zscript::{IncludeTree, ParseTree},
		},
	};

	use super::*;

	#[test]
	fn smoke_empty() {
		let ptree: ParseTree = crate::parse("", file, zdoom::Version::default());
		assert_no_errors(&ptree);
	}

	#[test]
	fn with_sample_data() {
		let (_, sample) = match read_sample_data("DOOMFRONT_ZSCRIPT_SAMPLE") {
			Ok(s) => s,
			Err(err) => {
				eprintln!("Skipping ZScript sample data-based unit test. Reason: {err}");
				return;
			}
		};

		let ptree: ParseTree = crate::parse(&sample, file, zdoom::Version::default());
		assert_no_errors(&ptree);
		prettyprint_maybe(ptree.cursor());
	}

	#[test]
	fn inctree() {
		let (root_path, _) = match read_sample_data("DOOMFRONT_ZSCRIPT_SAMPLE") {
			Ok(s) => s,
			Err(err) => {
				eprintln!("Skipping ZScript include tree unit test. Reason: {err}");
				return;
			}
		};

		let Some(root_parent_path) = root_path.parent() else {
			eprintln!(
				"Skipping ZScript include tree unit test. Reason: `{}` has no parent.",
				root_path.display()
			);
			return;
		};

		let inctree = IncludeTree::new(
			&root_path,
			|path| {
				let p = root_parent_path.join(path);

				if !p.exists() {
					return None;
				}

				let bytes = std::fs::read(p)
					.map_err(|err| panic!("file I/O failure: {err}"))
					.unwrap();
				let source = String::from_utf8_lossy(&bytes);
				Some(Cow::Owned(source.as_ref().to_owned()))
			},
			file,
			zdoom::Version::default(),
			Syn::IncludeDirective,
			Syn::StringLit,
		);

		assert!(inctree.missing.is_empty());

		for ptree in inctree.files {
			eprintln!("Checking `{}`...", ptree.path().display());
			assert_no_errors(&ptree);
		}
	}
}
