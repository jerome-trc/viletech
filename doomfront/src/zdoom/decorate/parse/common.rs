//! Combinators applicable to multiple other parts of the syntax.

use chumsky::{primitive, IterParser, Parser};

use crate::{
	comb,
	util::{builder::GreenCache, state::ParseState},
	zdoom::{
		decorate::Syn,
		lex::{Token, TokenStream},
		Extra,
	},
	ParseError,
};

/// Builds a [`Syn::Ident`] token.
///
/// DECORATE actor class identifiers are allowed to consist solely of ASCII digits.
/// Note that this filters out non-contextual keywords.
pub fn actor_ident<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: 'i + GreenCache,
{
	primitive::none_of([Token::Whitespace, Token::Comment])
		.repeated()
		.collect::<()>()
		.map_with_state(|(), span, state: &mut ParseState<C>| {
			state.gtb.token(Syn::Ident.into(), &state.source[span])
		})
}

/// `ident (trivia* '.' trivia* ident)*`
pub fn ident_chain<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: 'i + GreenCache,
{
	comb::node(
		Syn::IdentChain.into(),
		primitive::group((
			comb::just_ts(Token::Ident, Syn::Ident.into()),
			comb::checkpointed(primitive::group((
				trivia_0plus(),
				comb::just_ts(Token::Dot, Syn::Dot.into()),
				trivia_0plus(),
				comb::just_ts(Token::Ident, Syn::Ident.into()),
			)))
			.repeated()
			.collect::<()>(),
		)),
	)
}

/// In certain contexts, DECORATE allows providing number literals but not
/// expressions, so "negative literals" are required in place of unary negation.
pub(super) fn int_lit_negative<'i, C>(
) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	comb::checkpointed(primitive::group((
		comb::just_ts(Token::Minus, Syn::Minus.into()),
		comb::just_ts(Token::IntLit, Syn::IntLit.into()),
	)))
}

/// In certain contexts, DECORATE allows providing number literals but not
/// expressions, so "negative literals" are required in place of unary negation.
pub(super) fn float_lit_negative<'i, C>(
) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	comb::checkpointed(primitive::group((
		comb::just_ts(Token::Minus, Syn::Minus.into()),
		comb::just_ts(Token::FloatLit, Syn::FloatLit.into()),
	)))
}

/// Remember that this matches either a heterogenous whitespace span or a comment,
/// so to deal with both in one span requires repetition.
pub fn trivia<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	primitive::choice((
		comb::just_ts(Token::Whitespace, Syn::Whitespace.into()),
		comb::just_ts(Token::Comment, Syn::Comment.into()),
	))
	.map(|_| ())
	.boxed()
}

pub(super) fn trivia_0plus<'i, C>(
) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	trivia().repeated().collect::<()>()
}

pub(super) fn trivia_1plus<'i, C>(
) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	trivia().repeated().at_least(1).collect::<()>()
}

/// A subset of [`trivia`]; fails if a carriage return or newline appears in a
/// matched [`Token::Whitespace`] or [`Token::Comment`].
///
/// Necessary for delimiting parts in an actor state definition.
pub(super) fn trivia_1line<'i, C>(
) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	primitive::choice((
		primitive::just(Token::Whitespace),
		primitive::just(Token::Comment),
	))
	.try_map_with_state(|token, span: logos::Span, state: &mut ParseState<C>| {
		let multiline = state.source[span.clone()].contains(['\r', '\n']);

		let syn = match token {
			Token::Whitespace => {
				if !multiline {
					Syn::Whitespace
				} else {
					return Err(ParseError::custom(
						span,
						"expected single-line whitespace, found multi-line whitespace",
					));
				}
			}
			Token::Comment => {
				if !multiline {
					Syn::Comment
				} else {
					return Err(ParseError::custom(
						span,
						"expected multi-line comment, found single-line comment",
					));
				}
			}
			_ => unreachable!(),
		};

		state.gtb.token(syn.into(), &state.source[span]);
		Ok(())
	})
}
