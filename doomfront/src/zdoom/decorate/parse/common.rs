//! Combinators applicable to multiple other parts of the syntax.

use chumsky::{primitive, IterParser, Parser};
use rowan::{GreenNode, GreenToken};

use crate::{
	comb, parser_t,
	parsing::*,
	zdoom::{decorate::Syn, Token},
	GreenElement, ParseError, ParseState,
};

/// The returned parser emits a [`Syn::Ident`] token.
pub fn actor_ident<'i>() -> parser_t!(GreenToken) {
	primitive::none_of([
		Token::Whitespace,
		Token::Comment,
		Token::Colon,
		Token::BraceL,
	])
	.repeated()
	.collect::<()>()
	.map_with_state(|_, span, state: &mut ParseState| {
		GreenToken::new(Syn::Ident.into(), &state.source[span])
	})
}

/// The returned parser emits a [`Syn::IdentChain`] node.
pub fn ident_chain<'i>() -> parser_t!(GreenNode) {
	let ident = primitive::any()
		.filter(|t: &Token| {
			if matches!(
				t,
				Token::KwStates | Token::KwVar | Token::KwEnum | Token::KwConst
			) {
				return false;
			}

			t.is_keyword() || *t == Token::Ident
		})
		.map_with_state(comb::green_token(Syn::Ident));

	primitive::group((
		ident.clone(),
		primitive::group((
			trivia_0plus(),
			comb::just_ts(Token::Dot, Syn::Dot),
			ident.clone(),
		))
		.repeated()
		.collect::<Vec<_>>(),
	))
	.map(|group| coalesce_node(group, Syn::IdentChain))
}

/// The returned parser emits a [`Syn::IntLit`] token.
/// In certain contexts, DECORATE allows providing number literals but not
/// expressions, so "negative literals" are required in place of unary negation.
pub(super) fn int_lit_negative<'i>() -> parser_t!(GreenToken) {
	primitive::group((
		primitive::just(Token::Minus),
		primitive::just(Token::IntLit),
	))
	.map_with_state(|_, span, state: &mut ParseState| {
		GreenToken::new(Syn::IntLit.into(), &state.source[span])
	})
}

/// The returned parser emits a [`Syn::FloatLit`] token.
/// In certain contexts, DECORATE allows providing number literals but not
/// expressions, so "negative literals" are required in place of unary negation.
pub(super) fn float_lit_negative<'i>() -> parser_t!(GreenToken) {
	primitive::group((
		comb::just_ts(Token::Minus, Syn::Minus),
		comb::just_ts(Token::FloatLit, Syn::FloatLit),
	))
	.map_with_state(|_, span, state: &mut ParseState| {
		GreenToken::new(Syn::FloatLit.into(), &state.source[span])
	})
}

pub(super) fn recover_general<'i>(
	start: parser_t!(GreenToken),
	inner: parser_t!(Vec<GreenToken>),
	until: parser_t!(()),
) -> parser_t!(GreenElement) {
	primitive::group((start, inner, until.rewind())).map(|(start, mut inner, _)| {
		inner.insert(0, start);

		GreenNode::new(Syn::Error.into(), inner.into_iter().map(GreenElement::from)).into()
	})
}

#[must_use]
pub(super) fn recover_token(token: Token) -> Syn {
	match token {
		Token::Whitespace => Syn::Whitespace,
		Token::Comment => Syn::Comment,
		_ => Syn::Unknown,
	}
}

/// The returned parser emits a [`Syn::Whitespace`] or [`Syn::Comment`] token.
pub(super) fn trivia<'i>() -> parser_t!(GreenElement) {
	primitive::choice((
		comb::just_ts(Token::Whitespace, Syn::Whitespace),
		comb::just_ts(Token::Comment, Syn::Comment),
		comb::just_ts(Token::RegionStart, Syn::RegionStart),
		comb::just_ts(Token::RegionEnd, Syn::RegionEnd),
	))
	.map(|gtok| gtok.into())
}

/// Shorthand for `self.trivia().repeated().collect()`.
pub(super) fn trivia_0plus<'i>() -> parser_t!(Vec<GreenElement>) {
	trivia().repeated().collect()
}

/// Shorthand for `self.trivia().repeated().at_least(1).collect()`.
pub(super) fn trivia_1plus<'i>() -> parser_t!(Vec<GreenElement>) {
	trivia().repeated().at_least(1).collect()
}

/// The returned parser emits one or more [`Syn::Whitespace`] or [`Syn::Comment`] tokens.
///
/// A subset of [`trivia`]; fails if a carriage return or newline appears in a
/// matched [`Token::Whitespace`] or [`Token::Comment`]. Necessary for delimiting
/// parts in an actor state definition.
pub(super) fn trivia_1line<'i>() -> parser_t!(Vec<GreenElement>) {
	primitive::choice((
		primitive::just(Token::Whitespace),
		primitive::just(Token::Comment),
	))
	.try_map_with_state(|token, span: logos::Span, state: &mut ParseState| {
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

		Ok(GreenToken::new(syn.into(), &state.source[span]).into())
	})
	.repeated()
	.at_least(1)
	.collect()
}
