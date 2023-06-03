use doomfront::{
	chumsky::{primitive, IterParser, Parser},
	comb,
	util::builder::GreenCache,
};

use crate::{Syn, TokenStream};

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
				comb::just(Syn::Pound),
				comb::just(Syn::Bang).or_not(),
				comb::just(Syn::BracketL),
				// TODO: How are names referenced? What do arg lists look like?
				comb::just(Syn::BracketR),
			)),
		)
	}

	/// Builds a [`Syn::Whitespace`] or [`Syn::Comment`] token.
	pub fn trivia<'i, C>(&self) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
	where
		C: GreenCache,
	{
		primitive::choice((comb::just(Syn::Whitespace), comb::just(Syn::Comment))).map(|_| ())
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
