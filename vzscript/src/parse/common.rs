//! Combinators applicable to multiple other parts of the syntax.

use doomfront::{
	chumsky::{primitive, IterParser, Parser},
	comb,
	util::{builder::GreenCache, state::ParseState},
};

use crate::{Syn, TokenStream};

use super::{Extra, ParserBuilder};

impl<C: GreenCache> ParserBuilder<C> {
	/// Builds a [`Syn::Annotation`] node.
	pub fn annotation<'i>(
		&self,
	) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone {
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

	/// Builds a series of [`Syn::Annotation`] nodes.
	pub fn annotations<'i>(
		&self,
	) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone {
		self.annotation().repeated().collect::<()>()
	}

	/// Builds a [`Syn::Ident`] token, possibly converted from a [`Syn::IdentRaw`].
	pub fn ident<'i>(&self) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone {
		primitive::choice((primitive::just(Syn::Ident), primitive::just(Syn::IdentRaw)))
			.map_with_state(|_, span, state: &mut ParseState<C>| {
				state.gtb.token(Syn::Ident.into(), &state.source[span]);
			})
	}

	/// Builds a [`Syn::IdentChain`] node.
	pub fn ident_chain<'i>(
		&self,
	) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone {
		comb::node(
			Syn::IdentChain.into(),
			primitive::group((
				self.ident(),
				comb::checkpointed(primitive::group((
					self.trivia_0plus(),
					comb::just(Syn::Dot),
					self.trivia_0plus(),
					self.ident(),
				)))
				.repeated()
				.collect::<()>(),
			)),
		)
	}

	/// Builds a [`Syn::Whitespace`] or [`Syn::Comment`] token.
	pub fn trivia<'i>(&self) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone {
		primitive::choice((comb::just(Syn::Whitespace), comb::just(Syn::Comment))).map(|_| ())
	}

	pub(super) fn trivia_0plus<'i>(
		&self,
	) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone {
		self.trivia().repeated().collect::<()>()
	}

	pub(super) fn trivia_1plus<'i>(
		&self,
	) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone {
		self.trivia().repeated().at_least(1).collect::<()>()
	}
}
