//! A [syntax tag type] with a procedurally-generated lexer for its tokens.
//!
//! [syntax tag type]: Syn

use doomfront::{
	chumsky::{self, prelude::Input},
	rowan,
};
use logos::Logos;

use crate::{TokenMapper, TokenStream, Version};

/// A stronger type over [`rowan::SyntaxKind`] representing all kinds of syntax elements.
#[repr(u16)]
#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[logos(extras = Version)]
#[allow(clippy::manual_non_exhaustive)]
pub enum Syn {
	// Nodes: high-level composites ////////////////////////////////////////////
	/// `'#' '!'? '[' resolver arglist? ']'`
	Annotation,
	/// A sequence of tokens that did not form a valid syntax element.
	Error,
	/// A top-level node representing a whole file.
	FileRoot,
	/// A top-level node representing a whole REPL submission.
	ReplRoot,
	// Nodes: expressions //////////////////////////////////////////////////////
	BinExpr,
	/// `'(' expr ')'`
	GroupedExpr,
	/// Will have a single child token tagged as one of the following:
	/// - [`Syn::FalseLit`]
	/// - [`Syn::FloatLit`]
	/// - [`Syn::IntLit`]
	/// - [`Syn::StringLit`]
	/// - [`Syn::TrueLit`]
	/// - [`Syn::VoidLit`]
	Literal,
	// Tokens: literals ////////////////////////////////////////////////////////
	/// `false`
	#[token("false")]
	FalseLit,
	/// Uses the same syntax as Rust floating-point literals.
	#[regex(r#"[0-9][0-9_]*\.[^._\p{XID_Start}]*(f32|f64)?"#)]
	#[regex(r#"[0-9][0-9_]*\.[0-9][0-9_]*(f32|f64)?"#)]
	#[regex(r#"[0-9][0-9_]*(\.[0-9][0-9_]*)?[eE][+-]?[0-9_]*[0-9][0-9_]*(f32|f64)?"#)]
	FloatLit,
	/// Uses the same syntax as Rust integer literals.
	#[regex("[0-9][0-9_]*(u8|u16|u32|u64|i8|i16|i32|i64)?")]
	#[regex("0b[01_]*[01][01_]*(u8|u16|u32|u64|i8|i16|i32|i64)?")]
	#[regex("0o[0-7_]*[0-7][0-7_]*(u8|u16|u32|u64|i8|i16|i32|i64)?")]
	#[regex("0x[0-9a-fA-F_]*[0-9a-fA-F][0-9a-fA-F_]*(u8|u16|u32|u64|i8|i16|i32|i64)?")]
	IntLit,
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
	StringLit,
	/// `true`
	#[token("true")]
	TrueLit,
	/// `()`
	#[token("()")]
	VoidLit,
	// Tokens: keywords ////////////////////////////////////////////////////////
	#[token("break", priority = 5)]
	KwBreak,
	#[token("const", priority = 5)]
	KwConst,
	#[token("continue", priority = 5)]
	KwContinue,
	#[token("else", priority = 5)]
	KwElse,
	#[token("for", priority = 5)]
	KwFor,
	#[token("if", priority = 5)]
	KwIf,
	#[token("static", priority = 5)]
	KwStatic,
	#[token("struct", priority = 5)]
	KwStruct,
	#[token("until", priority = 5)]
	KwUntil,
	#[token("while", priority = 5)]
	KwWhile,
	// Tokens: glyphs //////////////////////////////////////////////////////////
	#[token("!")]
	Bang,
	#[token("!=")]
	BangEq,
	#[token("{")]
	BraceL,
	#[token("}")]
	BraceR,
	#[token("[")]
	BracketL,
	#[token("]")]
	BracketR,
	#[token("-")]
	Minus,
	#[token("(")]
	ParenL,
	#[token(")")]
	ParenR,
	#[token("+")]
	Plus,
	#[token("#")]
	Pound,
	// Tokens: miscellaenous ///////////////////////////////////////////////////
	#[regex("[a-zA-Z_][a-zA-Z0-9_]*", priority = 4)]
	Ident,
	#[regex(r#"///([^/][^\n]*)?"#, priority = 2)]
	DocComment,
	/// Either single-line (C++-style) or multi-line (C-style).
	#[regex("//[^\n]*\n", priority = 1)]
	#[regex(r"/[*]([^*]|([*][^/]))*[*]+/")]
	Comment,
	/// Spaces, newlines, carriage returns, or tabs.
	#[regex("[ \r\n\t]+")]
	Whitespace,
	/// Input not recognized by the lexer.
	Unknown,
	#[doc(hidden)]
	__Last,
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

impl Syn {
	#[must_use]
	pub fn stream(source: &str, version: Version) -> TokenStream<'_> {
		fn mapper(input: (Result<Syn, ()>, logos::Span)) -> (Syn, logos::Span) {
			(input.0.unwrap_or(Syn::Unknown), input.1)
		}

		let f: TokenMapper = mapper; // Yes, this is necessary.

		chumsky::input::Stream::from_iter(Syn::lexer_with_extras(source, version).spanned().map(f))
			.spanned(source.len()..source.len())
	}
}
