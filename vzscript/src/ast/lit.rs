//! A helper for extracting the values from literals.

use std::num::{ParseFloatError, ParseIntError};

use util::SmallString;

use crate::{Syn, SyntaxToken};

/// Wrapper around a [`SyntaxToken`] with convenience functions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LitToken(pub(super) SyntaxToken);

impl LitToken {
	#[must_use]
	pub fn bool(&self) -> Option<bool> {
		match self.0.kind() {
			Syn::TrueLit => Some(true),
			Syn::FalseLit => Some(false),
			_ => None,
		}
	}

	#[must_use]
	pub fn float(&self) -> Option<Result<f64, ParseFloatError>> {
		let Syn::FloatLit = self.0.kind() else { return None; };

		let text = self.0.text();

		let end = text.len()
			- text
				.chars()
				.rev()
				.position(|c| !c.eq_ignore_ascii_case(&'f'))
				.unwrap();

		let mut string = SmallString::new();

		for c in text[..end].chars() {
			if c != '_' {
				string.push(c);
			}
		}

		Some(string.parse::<f64>())
	}

	#[must_use]
	pub fn int(&self) -> Option<Result<u64, ParseIntError>> {
		let Syn::IntLit = self.0.kind() else { return None; };

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

		let mut string = SmallString::new();

		for c in text[start..end].chars() {
			if c != '_' {
				string.push(c);
			}
		}

		Some(u64::from_str_radix(&string, radix))
	}

	#[must_use]
	pub fn name(&self) -> Option<&str> {
		let Syn::NameLit = self.0.kind() else { return None; };

		let text = self.0.text();
		let start = text.chars().position(|c| c == '\'').unwrap();
		let end = text.chars().rev().position(|c| c == '\'').unwrap();
		Some(text.get((start + 1)..(text.len() - end - 1)).unwrap())
	}

	/// Returns `true` if this token's kind is [`Syn::NullLit`].
	#[must_use]
	pub fn null(&self) -> bool {
		self.0.kind() == Syn::NullLit
	}

	#[must_use]
	pub fn string(&self) -> Option<&str> {
		let Syn::StringLit = self.0.kind() else { return None; };

		let text = self.0.text();
		let start = text.chars().position(|c| c == '"').unwrap();
		let end = text.chars().rev().position(|c| c == '"').unwrap();
		Some(text.get((start + 1)..(text.len() - end - 1)).unwrap())
	}
}

impl std::ops::Deref for LitToken {
	type Target = SyntaxToken;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
