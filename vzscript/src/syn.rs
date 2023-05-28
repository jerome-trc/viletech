//! Tags representing syntax nodes, from low-level primitives to high-level composites.

use doomfront::rowan;

/// Tags representing syntax nodes, from low-level primitives to high-level composites.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(clippy::manual_non_exhaustive)]
pub enum Syn {
	// Nodes: high-level composites ////////////////////////////////////////////
	/// `'#' '!'? '[' resolver arglist? ']'`
	Annotation,
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
	FalseLit,
	/// Uses the same syntax as Rust floating-point literals.
	FloatLit,
	/// Uses the same syntax as Rust integer literals.
	IntLit,
	/// Uses the same syntax as Rust string literals.
	StringLit,
	/// `true`
	TrueLit,
	/// `()`
	VoidLit,
	// Tokens: keywords ////////////////////////////////////////////////////////
	KwBreak,
	KwConst,
	KwContinue,
	KwElse,
	KwFor,
	KwIf,
	KwStatic,
	KwStruct,
	KwUntil,
	KwWhile,
	// Tokens: glyphs //////////////////////////////////////////////////////////
	/// `!`
	Bang,
	/// `!=`
	BangEq,
	// `{`
	BraceL,
	/// `}`
	BraceR,
	/// `[`
	BracketL,
	/// `]`
	BracketR,
	/// `-`
	Minus,
	/// `(`
	ParenL,
	/// `)`
	ParenR,
	/// `+`
	Plus,
	/// `#`
	Pound,
	// Tokens: miscellaenous ///////////////////////////////////////////////////
	/// Either single-line (C++-style) or multi-line (C-style).
	Comment,
	/// Spaces, newlines, carriage returns, or tabs.
	Whitespace,
	/// Input considered by the lexer or parser to be invalid.
	Error,
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
