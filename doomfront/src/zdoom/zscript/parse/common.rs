//! Combinators applicable to multiple other parts of the syntax.

use chumsky::{primitive, IterParser, Parser};

use crate::{
	comb,
	util::builder::GreenCache,
	zdoom::{
		lex::{Token, TokenStream},
		zscript::Syn,
		Extra,
	},
};

/// Builds a [`Syn::Comment`] or [`Syn::Whitespace`] token.
/// Remember that this matches either a heterogenous whitespace span or a comment,
/// so to deal with both in one span requires repetition.
pub fn trivia<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: 'i + GreenCache,
{
	primitive::choice((
		comb::just(Token::Whitespace, Syn::Whitespace.into()),
		comb::just(Token::Comment, Syn::Comment.into()),
	))
	.map(|_| ())
	.boxed()
}

/// Shorthand for `trivia().repeated().collect::<()>()`.
/// Builds a series of [`Syn::Comment`] or [`Syn::Whitespace`] tokens.
pub fn trivia_0plus<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: 'i + GreenCache,
{
	trivia().repeated().collect::<()>()
}

/// Shorthand for `trivia().repeated().at_least(1).collect::<()>()`.
/// Builds a series of [`Syn::Comment`] or [`Syn::Whitespace`] tokens.
pub fn trivia_1plus<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: 'i + GreenCache,
{
	trivia().repeated().at_least(1).collect::<()>()
}
