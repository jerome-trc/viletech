//! AST nodes for representing literals.

use std::num::ParseIntError;

use doomfront::{
	rowan::{self, ast::AstNode},
	simple_astnode,
};
use serde::Serialize;

use crate::utils::string::unescape_char;

use super::{Syn, SyntaxNode, SyntaxToken};

/// Wraps a node tagged [`Syn::Literal`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[repr(transparent)]
pub struct Literal(pub(super) SyntaxNode);

simple_astnode!(Syn, Literal, Syn::Literal);

impl Literal {
	#[must_use]
	pub fn token(&self) -> LitToken {
		LitToken(self.0.first_token().unwrap())
	}
}

/// Wrapper around a [`SyntaxToken`] with convenience functions.
/// See [`Syn::Literal`]'s documentation to see possible token tags.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[repr(transparent)]
pub struct LitToken(SyntaxToken);

impl LitToken {
	/// If this wraps a [`Syn::LitTrue`] or [`Syn::LitFalse`] token,
	/// this returns the corresponding value. Otherwise this returns `None`.
	#[must_use]
	pub fn bool(&self) -> Option<bool> {
		match self.0.kind() {
			Syn::LitTrue => Some(true),
			Syn::LitFalse => Some(false),
			_ => None,
		}
	}

	/// If this wraps a [`Syn::LitChar`], this returns the character within
	/// the delimiting quotation marks. Otherwise this returns `None`.
	#[must_use]
	pub fn char(&self) -> Option<char> {
		if self.0.kind() == Syn::LitChar {
			let text = self.0.text();
			let start = text.chars().position(|c| c == '\'').unwrap();
			let end = text.chars().rev().position(|c| c == '\'').unwrap();
			let inner = text.get((start + 1)..(text.len() - end - 1)).unwrap();
			unescape_char(inner).ok()
		} else {
			None
		}
	}

	#[must_use]
	pub fn float(&self) -> Option<f64> {
		if !matches!(self.0.kind(), Syn::LitFloat) {
			return None;
		}

		let text = self.0.text();

		// Identify the position of the suffix.
		let end = text.len() - text.chars().rev().position(|c| c != 'f').unwrap();
		let inner = &text[0..end];
		let mut temp = String::with_capacity(text.len());

		for c in inner.chars().filter(|c| *c != '_') {
			temp.push(c);
		}

		temp.parse::<f64>().ok()
	}

	/// Shorthand for `self.syntax().kind() == Syn::LitVoid`.
	#[must_use]
	pub fn is_void(&self) -> bool {
		self.0.kind() == Syn::LitVoid
	}

	/// Shorthand for `self.syntax().kind() == Syn::LitNull`.
	#[must_use]
	pub fn is_null(&self) -> bool {
		self.0.kind() == Syn::LitNull
	}

	/// Returns `None` if this is not tagged with [`Syn::LitInt`].
	/// Returns `Some(Err)` if integer parsing fails,
	/// such as if the written value is too large to fit into a `u64`.
	#[must_use]
	pub fn int(&self) -> Option<Result<u64, ParseIntError>> {
		if !matches!(self.0.kind(), Syn::LitInt) {
			return None;
		}

		let text = self.0.text();

		let radix = if text.len() > 2 {
			match &text[0..2] {
				"0x" => 16,
				"0b" => 2,
				"0o" => 8,
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
				.position(|c| !matches!(c, 'i' | 'u'))
				.unwrap();
		let inner = &text[start..end];
		let mut temp = String::with_capacity(inner.len());

		for c in inner.chars().filter(|c| *c != '_') {
			temp.push(c);
		}

		Some(u64::from_str_radix(&temp, radix))
	}

	/// If this wraps a [`Syn::LitString`] token, this returns the string's
	/// content with the delimiting quotation marks stripped away.
	/// Otherwise this returns `None`.
	#[must_use]
	pub fn string(&self) -> Option<&str> {
		if self.0.kind() == Syn::LitString {
			let text = self.0.text();
			let start = text.chars().position(|c| c == '"').unwrap();
			let end = text.chars().rev().position(|c| c == '"').unwrap();
			text.get((start + 1)..(text.len() - end - 1))
		} else {
			None
		}
	}

	#[must_use]
	pub fn syntax(&self) -> &SyntaxToken {
		&self.0
	}
}
