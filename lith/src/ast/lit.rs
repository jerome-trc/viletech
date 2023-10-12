//! A wrapper around [`SyntaxToken`] to provide literal-reading conveniences.

use std::num::{ParseFloatError, ParseIntError};

use crate::{Syn, SyntaxToken};

/// Wrapper around a [`SyntaxToken`] with convenience functions.
/// Returned from calling [`Literal::token`].
/// See [`Syn::Literal`]'s documentation to see possible token tags.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LitToken(pub(super) SyntaxToken);

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

	/// Returns `None` if this is not tagged with [`Syn::LitFloat`].
	#[must_use]
	pub fn float(&self) -> Option<Result<f64, ParseFloatError>> {
		if !matches!(self.0.kind(), Syn::LitFloat) {
			return None;
		}

		let text = self.0.text();

		let inner = if text.ends_with("f32") || text.ends_with("f64") {
			&text[0..(text.len() - 3)]
		} else {
			text
		};

		// TODO: consider `SmallString` here.
		let mut temp = String::with_capacity(text.len());

		for c in inner.chars().filter(|c| *c != '_') {
			temp.push(c);
		}

		Some(temp.parse::<f64>())
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
		let end = text
			.chars()
			.position(|c| matches!(c, 'i' | 'u'))
			.unwrap_or(text.len());
		let inner = &text[start..end];
		let mut temp = String::with_capacity(inner.len());

		// TODO: consider `SmallString` here.
		for c in inner.chars().filter(|c| *c != '_') {
			temp.push(c);
		}

		Some(u64::from_str_radix(&temp, radix))
	}

	#[must_use]
	pub fn name(&self) -> Option<&str> {
		if self.kind() != Syn::LitName {
			return None;
		}

		let text = self.0.text();
		let start = text.chars().position(|c| c == '\'').unwrap();
		let end = text.chars().rev().position(|c| c == '\'').unwrap();
		text.get((start + 1)..(text.len() - end - 1))
	}

	/// If this wraps a [`Syn::LitString`] token, this returns the string's
	/// content with the delimiting quotation marks stripped away.
	/// Otherwise this returns `None`.
	#[must_use]
	pub fn string(&self) -> Option<&str> {
		if self.0.kind() == Syn::LitString {
			let text = self.0.text();
			text.get(1..(text.len() - 1))
		} else {
			None
		}
	}
}

impl std::ops::Deref for LitToken {
	type Target = SyntaxToken;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
