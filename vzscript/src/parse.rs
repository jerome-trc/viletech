//! A [builder](ParserBuilder) for emitting parser combinators.
//!
//! To start you will likely want to use [`ParserBuilder::repl`] or [`ParserBuilder::file`].

mod common;
mod expr;
mod item;

use std::{
	marker::PhantomData,
	path::{Path, PathBuf},
};

use doomfront::{
	chumsky::{primitive, IterParser, Parser},
	gcache::GreenCache,
	parser_t,
	rowan::GreenNode,
};

use crate::{ParseTree, Syn, Version};

pub type Error<'i> = doomfront::ParseError<'i, Syn>;

/// Gives context to functions yielding parser combinators
/// (e.g. the user's selected VZScript version).
///
/// Thus, this information never has to be passed through deep call trees, and any
/// breaking changes to this context are minimal in scope.
#[derive(Debug)]
#[non_exhaustive]
pub struct ParserBuilder<C: GreenCache> {
	pub(self) _version: Version,
	phantom: PhantomData<C>,
}

impl<C: GreenCache> ParserBuilder<C> {
	#[must_use]
	pub fn new(version: Version) -> Self {
		Self {
			_version: version,
			phantom: PhantomData,
		}
	}

	/// The returned parser emits a [`Syn::FileRoot`] node.
	pub fn file<'i>(&self) -> parser_t!(Syn, GreenNode) {
		primitive::choice((
			self.trivia(),
			// Only "inner" annotations are allowed at file scope.
			self.annotation().map(|gnode| gnode.into()),
			self.func_decl().map(|gnode| gnode.into()),
		))
		.repeated()
		.collect::<Vec<_>>()
		.map(|elems| GreenNode::new(Syn::FileRoot.into(), elems))
		.boxed()
	}

	/// The returned parser emits a [`Syn::ReplRoot`] node.
	pub fn repl<'i>(&self) -> parser_t!(Syn, GreenNode) {
		primitive::choice((self.trivia(), self.expr().map(|gnode| gnode.into())))
			.repeated()
			.collect::<Vec<_>>()
			.map(|elems| GreenNode::new(Syn::ReplRoot.into(), elems))
			.boxed()
	}
}

/// Gets compiled into one [module](crate::module::Module).
#[derive(Debug)]
pub struct FileParseTree {
	inner: ParseTree<'static>,
	path: PathBuf,
}

impl FileParseTree {
	#[must_use]
	pub fn path(&self) -> &Path {
		&self.path
	}

	#[must_use]
	pub fn into_inner(self) -> ParseTree<'static> {
		self.inner
	}
}

impl std::ops::Deref for FileParseTree {
	type Target = ParseTree<'static>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

/// Gets compiled into one [library](crate::library::Library).
#[derive(Debug, Default)]
pub struct IncludeTree {
	pub files: Vec<FileParseTree>,
}

impl IncludeTree {
	#[must_use]
	pub fn new() -> Self {
		unimplemented!("Include tree parsing pending DoomFront's `Filesystem` trait.")
	}

	#[must_use]
	pub fn files(&self) -> &[FileParseTree] {
		&self.files
	}

	#[must_use]
	pub fn into_inner(self) -> Vec<FileParseTree> {
		self.files
	}

	#[must_use]
	pub fn any_errors(&self) -> bool {
		self.files.iter().any(|ptree| !ptree.errors.is_empty())
	}
}
