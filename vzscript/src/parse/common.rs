//! Combinators applicable to multiple other parts of the syntax.

use doomfront::{
	chumsky::{primitive, IterParser, Parser},
	comb,
	gcache::GreenCache,
	parser_t,
	parsing::*,
	rowan::{GreenNode, GreenToken},
	GreenElement,
};

use crate::Syn;

use super::ParserBuilder;

impl<C: GreenCache> ParserBuilder<C> {
	/// The returned parser emits a [`Syn::Annotation`] node.
	pub fn annotation<'i>(&self) -> parser_t!(Syn, GreenNode) {
		primitive::group((
			comb::just(Syn::Pound),
			comb::just(Syn::Bang).or_not(),
			comb::just(Syn::BracketL),
			// TODO: How are names referenced? What do arg lists look like?
			comb::just(Syn::BracketR),
		))
		.map(|group| coalesce_node(group, Syn::Annotation))
	}

	/// Combines [`Self::annotation`] (0 or more) with [`Self::trivia`] padding.
	pub fn annotations<'i>(&self) -> parser_t!(Syn, Vec<GreenElement>) {
		primitive::group((self.annotation(), self.trivia_0plus())).map(coalesce_vec)
	}

	/// The returned parser emits a [`Syn::Ident`] token (possibly converted
	/// from a [`Syn::IdentRaw`] coming from the lexer).
	pub fn ident<'i>(&self) -> parser_t!(Syn, GreenToken) {
		primitive::choice((primitive::just(Syn::Ident), primitive::just(Syn::IdentRaw)))
			.map_with_state(comb::green_token(Syn::Ident))
	}

	/// The returned parser emits a [`Syn::Whitespace`] or [`Syn::Comment`] token.
	pub fn trivia<'i>(&self) -> parser_t!(Syn, GreenElement) {
		primitive::choice((comb::just(Syn::Whitespace), comb::just(Syn::Comment)))
			.map(|gtok| gtok.into())
	}

	/// Shorthand for `self.trivia().repeated().collect()`.
	pub(super) fn trivia_0plus<'i>(&self) -> parser_t!(Syn, Vec<GreenElement>) {
		self.trivia().repeated().collect()
	}

	/// Shorthand for `self.trivia().repeated().at_least(1).collect()`.
	pub(super) fn trivia_1plus<'i>(&self) -> parser_t!(Syn, Vec<GreenElement>) {
		self.trivia().repeated().at_least(1).collect()
	}
}
