//! A [syntax tag type](Syn) with a macro-generated lexer for its tokens.

use doomfront::rowan;
use logos::Logos;

/// A stronger type over [`rowan::SyntaxKind`] representing all kinds of syntax elements.
#[repr(u16)]
#[derive(Logos, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[logos(error = Syn)]
#[allow(clippy::manual_non_exhaustive)]
pub enum Syn {
	// Nodes: high-level constructs ////////////////////////////////////////////
	/// A sequence of tokens that did not form a valid syntax element.
	Error,
	// Tokens: literals ////////////////////////////////////////////////////////
	/// `false`
	#[token("false")]
	LitFalse,
	/// Uses the same syntax as Rust floating-point literals.
	#[regex(r#"[0-9][0-9_]*\.[^._\p{XID_Start}]*(f32|f64)?"#)]
	#[regex(r#"[0-9][0-9_]*\.[0-9][0-9_]*(f32|f64)?"#)]
	#[regex(r#"[0-9][0-9_]*(\.[0-9][0-9_]*)?[eE][+-]?[0-9_]*[0-9][0-9_]*(f32|f64)?"#)]
	LitFloat,
	/// Uses the same syntax as Rust integer literals.
	#[regex("[0-9][0-9_]*(u8|u16|u32|u64|i8|i16|i32|i64)?")]
	#[regex("0b[01_]*[01][01_]*(u8|u16|u32|u64|i8|i16|i32|i64)?")]
	#[regex("0o[0-7_]*[0-7][0-7_]*(u8|u16|u32|u64|i8|i16|i32|i64)?")]
	#[regex("0x[0-9a-fA-F_]*[0-9a-fA-F][0-9a-fA-F_]*(u8|u16|u32|u64|i8|i16|i32|i64)?")]
	LitInt,
	#[regex("'[^''\n]*'")]
	LitName,
	/// Uses the same syntax as Rust string literals.
	#[regex(
		r##"(?x)"
		(
			[^"\\]|
			(\\')|
			(\\")|
			(\\n)|
			(\\r)|
			(\\t)|
			(\\\\)|
			(\\0)|
			(\\x[0-8][0-9a-fA-F])|
			(\\u\{([0-9a-fA-F]_*){1,6}\})|
			(\\\n)
		)*
		""##
	)]
	LitString,
	/// `true`
	#[token("true")]
	LitTrue,
	// Tokens: miscellaneous ///////////////////////////////////////////////////
	/// Same rules as C identifiers.
	#[regex("[a-zA-Z_][a-zA-Z0-9_]*", priority = 4)]
	Ident,
	/// A Zig-style single-line documentation comment.
	#[regex(r#"///[^\n]*[\n]*"#, priority = 2)]
	DocComment,
	/// Like Zig, Lith only has single-line comments in the C++/post-C99 style.
	#[regex("//[^/\n][^\n]*", priority = 1)]
	#[regex("////[^\n]*")]
	#[regex("//")]
	Comment,
	/// Spaces, newlines, carriage returns, or tabs.
	#[regex("[ \r\n\t]+")]
	Whitespace,
	/// Input not recognized by the lexer.
	#[default]
	Unknown,
	/// A dummy token for [`doomfront::LangExt`].
	Eof,
	#[doc(hidden)]
	__Last, // Only in service of `kind_from_raw`.
}

impl Syn {
	#[must_use]
	pub fn is_trivia(self) -> bool {
		matches!(self, Self::Whitespace | Self::Comment)
	}
}

impl From<Syn> for rowan::SyntaxKind {
	fn from(value: Syn) -> Self {
		Self(value as u16)
	}
}

impl rowan::Language for Syn {
	type Kind = Self;

	fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
		assert!(raw.0 < Self::__Last as u16);
		unsafe { std::mem::transmute::<u16, Syn>(raw.0) }
	}

	fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
		kind.into()
	}
}

impl doomfront::LangExt for Syn {
	type Token = Self;
	const EOF: Self::Token = Self::Eof;
	const ERR_NODE: Self::Kind = Self::Error;
}

/// A placeholder type to prevent API breaks in the future if the lexer needs to,
/// for instance, tokenize keywords version-sensitively.
#[derive(Debug, Default)]
pub struct Context {}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn comments() {
		const SOURCES: &[&str] = &["//", "// ", "////"];

		for source in SOURCES {
			let mut lexer = Syn::lexer(source);
			let t0 = lexer.next().unwrap().unwrap();
			assert_eq!(t0, Syn::Comment);
		}
	}

	#[test]
	fn doc_comments() {
		const SOURCES: &[&str] = &["///", "/// ", "/// lorem ipsum"];

		for source in SOURCES {
			let mut lexer = Syn::lexer(source);
			let t0 = lexer.next().unwrap().unwrap();
			assert_eq!(t0, Syn::DocComment);
		}
	}
}
