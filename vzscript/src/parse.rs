//! A [builder](ParserBuilder) for emitting parser combinators.
//!
//! To start you will likely want to use [`ParserBuilder::repl`] or [`ParserBuilder::file`].

mod common;
mod expr;

use std::path::{Path, PathBuf};

use doomfront::{
	chumsky::{primitive, IterParser, Parser},
	util::builder::GreenCache,
};

use crate::{ParseTree, Syn, TokenStream, Version};

pub type Extra<'i, C> = doomfront::Extra<'i, Syn, C>;
pub type Error<'i> = doomfront::ParseError<'i, Syn>;

/// Gives context to functions yielding parser combinators
/// (e.g. the user's selected VZScript version).
///
/// Thus, this information never has to be passed through deep call trees, and any
/// breaking changes to this context are minimal in scope.
#[derive(Debug)]
#[non_exhaustive]
pub struct ParserBuilder {
	pub(self) _version: Version,
}

impl ParserBuilder {
	#[must_use]
	pub fn new(version: Version) -> Self {
		Self { _version: version }
	}

	/// Does not build a node by itself; use [`doomfront::parse`] and pass
	/// [`Syn::FileRoot`](crate::Syn::FileRoot).
	pub fn file<'i, C>(&self) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
	where
		C: GreenCache,
	{
		primitive::choice((
			self.trivia(),
			// Only "inner" annotations are allowed at file scope.
			self.annotation(),
		))
		.repeated()
		.collect::<()>()
		.boxed()
	}

	/// Does not build a node by itself; use [`doomfront::parse`] and pass
	/// [`Syn::ReplRoot`](crate::Syn::ReplRoot).
	pub fn repl<'i, C>(&self) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
	where
		C: GreenCache,
	{
		primitive::choice((self.trivia(), self.expr()))
			.repeated()
			.collect::<()>()
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
