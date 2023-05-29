use doomfront::{
	chumsky::{primitive, IterParser, Parser},
	comb,
	util::builder::GreenCache,
};

use crate::{
	lex::{Token, TokenStream},
	Syn,
};

use super::{Extra, ParserBuilder};

impl ParserBuilder {
	/// Builds a [`Syn::Annotation`] node.
	pub fn annotation<'i, C>(
		&self,
	) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
	where
		C: GreenCache,
	{
		comb::node(
			Syn::Annotation.into(),
			primitive::group((
				comb::just(Token::Pound, Syn::Pound.into()),
				comb::just(Token::Bang, Syn::Bang.into()).or_not(),
				comb::just(Token::BracketL, Syn::BracketL.into()),
				// TODO: How are names referenced? What do arg lists look like?
				comb::just(Token::BracketR, Syn::BracketR.into()),
			)),
		)
	}

	/// Builds a [`Syn::Whitespace`] or [`Syn::Comment`] token.
	pub fn trivia<'i, C>(&self) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
	where
		C: GreenCache,
	{
		primitive::choice((
			comb::just(Token::Whitespace, Syn::Whitespace.into()),
			comb::just(Token::Comment, Syn::Comment.into()),
		))
		.map(|_| ())
	}

	pub(super) fn trivia_0plus<'i, C>(
		&self,
	) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
	where
		C: GreenCache,
	{
		self.trivia().repeated().collect::<()>()
	}

	pub(super) fn _trivia_1plus<'i, C>(
		&self,
	) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
	where
		C: GreenCache,
	{
		self.trivia().repeated().at_least(1).collect::<()>()
	}
}