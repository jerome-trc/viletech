//! Combinators applicable to multiple other parts of the syntax.

use chumsky::{primitive, IterParser, Parser};

use crate::{
	comb, parser_t,
	zdoom::{zscript::Syn, Token},
	GreenElement,
};

use super::ParserBuilder;

impl ParserBuilder {
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
