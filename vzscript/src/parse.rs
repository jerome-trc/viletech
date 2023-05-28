//! A [builder](ParserBuilder) for emitting parser combinators.
//!
//! To start you will likely want to use [`ParserBuilder::repl`] or [`ParserBuilder::file`].

mod common;
mod expr;

use doomfront::{
	chumsky::{primitive, IterParser, Parser},
	util::builder::GreenCache,
};

use crate::{
	lex::{Token, TokenStream},
	Version,
};

pub type Extra<'i, C> = doomfront::Extra<'i, Token, C>;

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
		let ret = primitive::choice((
			self.trivia(),
			// Only "inner" annotations are allowed at file scope.
			self.annotation(),
		))
		.repeated()
		.collect::<()>();

		#[cfg(any(debug_assertions, test))]
		{
			ret.boxed()
		}
		#[cfg(not(any(debug_assertions, test)))]
		{
			ret
		}
	}

	/// Does not build a node by itself; use [`doomfront::parse`] and pass
	/// [`Syn::ReplRoot`](crate::Syn::ReplRoot).
	pub fn repl<'i, C>(&self) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
	where
		C: GreenCache,
	{
		let ret = primitive::choice((self.trivia(), self.expr()))
			.repeated()
			.collect::<()>();

		#[cfg(any(debug_assertions, test))]
		{
			ret.boxed()
		}
		#[cfg(not(any(debug_assertions, test)))]
		{
			ret
		}
	}
}
