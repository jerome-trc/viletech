//! Combinators applicable to multiple other parts of the syntax.

use chumsky::{primitive, IterParser, Parser};
use rowan::GreenToken;

use crate::{
	comb, parser_t,
	zdoom::{zscript::Syn, Token},
	GreenElement,
};

use super::ParserBuilder;

impl ParserBuilder {
	pub(super) fn _ident<'i>(&self) -> parser_t!(GreenToken) {
		primitive::any()
			.filter(|token: &Token| {
				matches!(
					token,
					Token::Ident
						| Token::KwBright | Token::KwFast
						| Token::KwSlow | Token::KwNoDelay
						| Token::KwCanRaise | Token::KwOffset
						| Token::KwLight
				)
			})
			.map_with_state(comb::green_token(Syn::Ident))
	}

	pub(super) fn trivia<'i>(&self) -> parser_t!(GreenElement) {
		primitive::choice((
			comb::just_ts(Token::Whitespace, Syn::Whitespace),
			comb::just_ts(Token::Comment, Syn::Comment),
		))
		.map(|token| token.into())
	}

	pub(super) fn trivia_0plus<'i>(&self) -> parser_t!(Vec<GreenElement>) {
		self.trivia().repeated().collect()
	}

	pub(super) fn _trivia_1plus<'i>(&self) -> parser_t!(Vec<GreenElement>) {
		self.trivia().repeated().at_least(1).collect()
	}
}
