//! Abstract syntax tree elements shared between multiple languages.

use std::num::{ParseFloatError, ParseIntError};

use rowan::SyntaxToken;

use crate::LangExt;

use super::{cvarinfo, decorate, zscript};

/// Wrapper around a [`SyntaxToken`] with convenience functions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LitToken<L: LangExt>(SyntaxToken<L>);

impl LitToken<zscript::Syn> {
	#[must_use]
	pub fn bool(&self) -> Option<bool> {
		match self.0.kind() {
			zscript::Syn::KwTrue => Some(true),
			zscript::Syn::KwFalse => Some(false),
			_ => None,
		}
	}

	#[must_use]
	pub fn float(&self) -> Option<Result<f64, ParseFloatError>> {
		match self.0.kind() {
			zscript::Syn::FloatLit => Some(self.parse_float()),
			_ => None,
		}
	}

	#[must_use]
	pub fn int(&self) -> Option<Result<(u64, IntSuffix), ParseIntError>> {
		match self.0.kind() {
			zscript::Syn::IntLit => Some(self.parse_int()),
			_ => None,
		}
	}

	#[must_use]
	pub fn name(&self) -> Option<&str> {
		match self.0.kind() {
			zscript::Syn::NameLit => Some(self.get_name()),
			_ => None,
		}
	}

	/// Returns `true` if this token's kind is [`zscript::Syn::NullLit`].
	#[must_use]
	pub fn null(&self) -> bool {
		self.0.kind() == zscript::Syn::NullLit
	}

	#[must_use]
	pub fn string(&self) -> Option<&str> {
		match self.0.kind() {
			zscript::Syn::StringLit => Some(self.get_string()),
			_ => None,
		}
	}
}

impl LitToken<decorate::Syn> {
	#[must_use]
	pub fn bool(&self) -> Option<bool> {
		match self.0.kind() {
			decorate::Syn::KwTrue => Some(true),
			decorate::Syn::KwFalse => Some(false),
			_ => None,
		}
	}

	#[must_use]
	pub fn float(&self) -> Option<Result<f64, ParseFloatError>> {
		match self.0.kind() {
			decorate::Syn::FloatLit => Some(self.parse_float()),
			_ => None,
		}
	}

	#[must_use]
	pub fn int(&self) -> Option<Result<(u64, IntSuffix), ParseIntError>> {
		match self.0.kind() {
			decorate::Syn::IntLit => Some(self.parse_int()),
			_ => None,
		}
	}

	/// A convenience function whichs trims off delimiting single quotes.
	#[must_use]
	pub fn name(&self) -> Option<&str> {
		match self.0.kind() {
			decorate::Syn::NameLit => Some(self.get_name()),
			_ => None,
		}
	}

	/// A convenience function whichs trims off delimiting double quotes.
	#[must_use]
	pub fn string(&self) -> Option<&str> {
		match self.0.kind() {
			decorate::Syn::StringLit => Some(self.get_string()),
			_ => None,
		}
	}
}

impl LitToken<cvarinfo::Syn> {
	#[must_use]
	pub fn bool(&self) -> Option<bool> {
		match self.0.kind() {
			cvarinfo::Syn::TrueLit => Some(true),
			cvarinfo::Syn::FalseLit => Some(false),
			_ => None,
		}
	}

	#[must_use]
	pub fn float(&self) -> Option<Result<f64, ParseFloatError>> {
		match self.0.kind() {
			cvarinfo::Syn::FloatLit => Some(self.parse_float()),
			_ => None,
		}
	}

	#[must_use]
	pub fn int(&self) -> Option<Result<(u64, IntSuffix), ParseIntError>> {
		match self.0.kind() {
			cvarinfo::Syn::IntLit => Some(self.parse_int()),
			_ => None,
		}
	}

	/// A convenience function whichs trims off delimiting double quotes.
	#[must_use]
	pub fn string(&self) -> Option<&str> {
		match self.0.kind() {
			cvarinfo::Syn::StringLit => Some(self.get_string()),
			_ => None,
		}
	}
}

// Shared //////////////////////////////////////////////////////////////////////

impl<L: LangExt> LitToken<L> {
	#[must_use]
	pub fn new(token: SyntaxToken<L>) -> Self {
		Self(token)
	}

	#[must_use]
	pub fn syntax(&self) -> &SyntaxToken<L> {
		&self.0
	}

	fn parse_float(&self) -> Result<f64, ParseFloatError> {
		let text = self.0.text();

		let end = text.len()
			- text
				.chars()
				.rev()
				.position(|c| !c.eq_ignore_ascii_case(&'f'))
				.unwrap();

		text[..end].parse::<f64>()
	}

	fn parse_int(&self) -> Result<(u64, IntSuffix), ParseIntError> {
		let text = self.0.text();

		let radix = if text.len() > 2 {
			match &text[0..2] {
				"0x" => 16,
				_ => 10,
			}
		} else {
			10
		};

		// Identify the span between the prefix and suffix.
		let start = if radix != 10 { 2 } else { 0 };

		let end = text.len()
			- text
				.chars()
				.rev()
				.position(|c| !(c.eq_ignore_ascii_case(&'u') || c.eq_ignore_ascii_case(&'l')))
				.unwrap();

		let suffix = match &text[(text.len().saturating_sub(2))..] {
			"uu" | "UU" | "uU" | "Uu" => IntSuffix::UU,
			"ll" | "LL" | "lL" | "Ll" => IntSuffix::LL,
			"ul" | "UL" | "uL" | "Ul" | "lu" | "LU" | "lU" | "Lu" => IntSuffix::UL,
			"u" | "U" => IntSuffix::U,
			"l" | "L" => IntSuffix::L,
			_ => IntSuffix::None,
		};

		u64::from_str_radix(&text[start..end], radix).map(|u| (u, suffix))
	}

	#[must_use]
	fn get_name(&self) -> &str {
		let text = self.0.text();
		let start = text.chars().position(|c| c == '\'').unwrap();
		let end = text.chars().rev().position(|c| c == '\'').unwrap();
		text.get((start + 1)..(text.len() - end - 1)).unwrap()
	}

	#[must_use]
	fn get_string(&self) -> &str {
		let text = self.0.text();
		let start = text.chars().position(|c| c == '"').unwrap();
		let end = text.chars().rev().position(|c| c == '"').unwrap();
		text.get((start + 1)..(text.len() - end - 1)).unwrap()
	}
}

/// See [`LitToken::int`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntSuffix {
	None,
	U,
	L,
	UU,
	LL,
	UL,
}

#[cfg(test)]
mod test {
	use logos::Logos;
	use rowan::{ast::AstNode, GreenNode, GreenToken};

	use crate::zdoom::{
		self,
		zscript::{ast, Syn, SyntaxNode},
	};

	use super::*;

	#[test]
	fn smoke_int() {
		const SOURCE: &str = "1234567890Lu";

		let mut lexer =
			zdoom::lex::Token::lexer_with_extras(SOURCE, zdoom::lex::Context::ZSCRIPT_LATEST);

		let token = lexer.next().unwrap().unwrap();
		assert_eq!(token, zdoom::Token::IntLit);

		let green = GreenNode::new(
			Syn::Literal.into(),
			[GreenToken::new(Syn::IntLit.into(), SOURCE).into()],
		);

		let ast = SyntaxNode::new_root(green);
		let lit = ast::Literal::cast(ast).unwrap();
		let lit_tok = lit.token();

		assert_eq!(lit_tok.int(), Some(Ok((1234567890, IntSuffix::UL))));
	}
}
