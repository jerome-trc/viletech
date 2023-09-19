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
		let Syn::FloatLit = self.0.kind() else {
			return None;
		};

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
	pub fn int(&self) -> Option<Result<(u64, IntSuffix), ParseIntError>> {
		let Syn::IntLit = self.0.kind() else {
			return None;
		};

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

		const INT_SUFFIXES: &[(&str, IntSuffix)] = &[
			("u8", IntSuffix::U8),
			("U8", IntSuffix::U8),
			("u16", IntSuffix::U16),
			("U16", IntSuffix::U16),
			("u32", IntSuffix::U32),
			("U32", IntSuffix::U32),
			("u64", IntSuffix::U64),
			("U64", IntSuffix::U64),
			("u128", IntSuffix::U128),
			("U128", IntSuffix::U128),
			("i8", IntSuffix::I8),
			("I8", IntSuffix::I8),
			("i16", IntSuffix::I16),
			("I16", IntSuffix::I16),
			("i32", IntSuffix::I32),
			("I32", IntSuffix::I32),
			("i64", IntSuffix::I64),
			("I64", IntSuffix::I64),
			("i128", IntSuffix::I128),
			("I128", IntSuffix::I128),
		];

		let mut suffix = IntSuffix::None;

		for (suf_str, suf_v) in INT_SUFFIXES.iter().copied() {
			if text.ends_with(suf_str) {
				suffix = suf_v;
				break;
			}
		}

		Some(u64::from_str_radix(&string, radix).map(|u| (u, suffix)))
	}

	#[must_use]
	pub fn name(&self) -> Option<&str> {
		let Syn::NameLit = self.0.kind() else {
			return None;
		};

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
		let Syn::StringLit = self.0.kind() else {
			return None;
		};

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

impl std::fmt::Display for LitToken {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self.0.kind() {
			Syn::FalseLit => write!(f, "`false`"),
			Syn::FloatLit => write!(f, "floating-point number {}", self.0.text()),
			Syn::IntLit => write!(f, "integer {}", self.0.text()),
			Syn::NameLit => write!(f, "name {}", self.0.text()),
			Syn::NullLit => write!(f, "`null`"),
			Syn::StringLit => write!(f, "string {}", self.0.text()),
			Syn::TrueLit => write!(f, "`true`"),
			_ => unreachable!(),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntSuffix {
	None,
	I8,
	U8,
	I16,
	U16,
	I32,
	U32,
	I64,
	U64,
	I128,
	U128,
}
