//! A wrapper around [`SyntaxToken`] to provide literal-reading conveniences.

use std::num::{ParseFloatError, ParseIntError};

use crate::{Syntax, SyntaxToken};

/// Wrapper around a [`SyntaxToken`] with convenience functions.
/// Returned from calling [`Literal::token`].
/// See [`Syntax::Literal`]'s documentation to see possible token tags.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LitToken(pub(super) SyntaxToken);

impl LitToken {
	/// If this wraps a [`Syntax::LitTrue`] or [`Syntax::LitFalse`] token,
	/// this returns the corresponding value. Otherwise this returns `None`.
	#[must_use]
	pub fn bool(&self) -> Option<bool> {
		match self.0.kind() {
			Syntax::LitTrue => Some(true),
			Syntax::LitFalse => Some(false),
			_ => None,
		}
	}

	/// Returns `None` if this is not tagged with [`Syntax::LitFloat`].
	#[must_use]
	pub fn float(&self) -> Option<Result<FloatLit, ParseFloatError>> {
		if !matches!(self.0.kind(), Syntax::LitFloat) {
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

		let num = match temp.parse::<f64>() {
			Ok(n) => n,
			Err(err) => return Some(Err(err)),
		};

		match text.get((text.len().saturating_sub(3))..) {
			Some("f32") => Some(Ok(FloatLit::F32(num))),
			Some("f64") => Some(Ok(FloatLit::F64(num))),
			_ => Some(Ok(FloatLit::NoSuffix(num))),
		}
	}

	/// Returns `None` if this is not tagged with [`Syntax::LitInt`].
	/// Returns `Some(Err)` if integer parsing fails,
	/// such as if the written value is too large to fit into a `u128`.
	#[must_use]
	pub fn int(&self) -> Option<Result<IntLit, ParseIntError>> {
		if !matches!(self.0.kind(), Syntax::LitInt) {
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

		let num = match u128::from_str_radix(&temp, radix) {
			Ok(n) => n,
			Err(err) => return Some(Err(err)),
		};

		match text.get((text.len().saturating_sub(4))..) {
			Some("i128") => return Some(Ok(IntLit::I128(num))),
			Some("u128") => return Some(Ok(IntLit::U128(num))),
			_ => {}
		}

		match text.get((text.len().saturating_sub(3))..) {
			Some("i64") => return Some(Ok(IntLit::I64(num))),
			Some("u64") => return Some(Ok(IntLit::U64(num))),
			Some("i32") => return Some(Ok(IntLit::I32(num))),
			Some("u32") => return Some(Ok(IntLit::U32(num))),
			Some("i16") => return Some(Ok(IntLit::I16(num))),
			Some("u16") => return Some(Ok(IntLit::U16(num))),
			_ => {}
		}

		match text.get((text.len().saturating_sub(2))..) {
			Some("i8") => Some(Ok(IntLit::I8(num))),
			Some("u8") => Some(Ok(IntLit::U8(num))),
			_ => Some(Ok(IntLit::NoSuffix(num))),
		}
	}

	#[must_use]
	pub fn name(&self) -> Option<&str> {
		if self.kind() != Syntax::LitName {
			return None;
		}

		let text = self.0.text();
		let start = text.chars().position(|c| c == '\'').unwrap();
		let end = text.chars().rev().position(|c| c == '\'').unwrap();
		text.get((start + 1)..(text.len() - end - 1))
	}

	/// If this wraps a [`Syntax::LitString`] token, this returns the string's
	/// content with the delimiting quotation marks stripped away.
	/// Otherwise this returns `None`.
	#[must_use]
	pub fn string(&self) -> Option<&str> {
		if self.0.kind() == Syntax::LitString {
			let text = self.0.text();
			text.get(1..(text.len() - 1))
		} else {
			None
		}
	}
}

impl std::fmt::Display for LitToken {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self.0.kind() {
			Syntax::LitFalse => write!(f, "`false`"),
			Syntax::LitFloat => write!(f, "floating-point number {}", self.0.text()),
			Syntax::LitInt => write!(f, "integer {}", self.0.text()),
			Syntax::LitName => write!(f, "name {}", self.0.text()),
			Syntax::LitString => write!(f, "string {}", self.0.text()),
			Syntax::LitTrue => write!(f, "`true`"),
			_ => unreachable!(),
		}
	}
}

impl std::ops::Deref for LitToken {
	type Target = SyntaxToken;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

/// Attaches a suffix tag to the output of [`LitToken::int`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntLit {
	NoSuffix(u128),
	I8(u128),
	I16(u128),
	I32(u128),
	I64(u128),
	I128(u128),
	U8(u128),
	U16(u128),
	U32(u128),
	U64(u128),
	U128(u128),
}

impl From<IntLit> for u128 {
	fn from(value: IntLit) -> Self {
		match value {
			IntLit::NoSuffix(i)
			| IntLit::I8(i)
			| IntLit::I16(i)
			| IntLit::I32(i)
			| IntLit::I64(i)
			| IntLit::I128(i)
			| IntLit::U8(i)
			| IntLit::U16(i)
			| IntLit::U32(i)
			| IntLit::U64(i)
			| IntLit::U128(i) => i,
		}
	}
}

/// Attaches a suffix tag to the output of [`LitToken::float`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FloatLit {
	NoSuffix(f64),
	F32(f64),
	F64(f64),
}
