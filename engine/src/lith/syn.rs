//! The lowest-level infrastructure in the LithScript frontend.
//!
//! This includes a 16-bit marker for different tokens and token composites
//! and a [`chumsky`]-based lexer.

use std::ops::Range;

use chumsky::{primitive, text, Parser};

/// Tags accompanying green tree nodes make up the lossless syntax tree (LST).
///
/// This contains everything, including whitespace, up from individual punctuation
/// marks (glyphs) to whole items like class definitions.
///
/// The lexer produces fairly low-level units:
/// - Glyphs such as `@` and `::`
/// - Preprocessor directives like `#include`
/// - Keywords like `struct`
/// - Literals of all kinds
/// - Identifiers
///
/// The parser then nests these within higher-level composites like expressions.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SyntaxKind {
	// Glyphs, composite glyphs, glyph-adjacent ////////////////////////////////
	Semicolon = 0,
	Comma,
	LParen,
	RParen,
	LBrace,
	RBrace,
	LBracket,
	RBracket,
	LAngle,
	RAngle,
	/// `@`
	At,
	/// `#`
	Pound,
	/// `~` a.k.a tilde.
	Grave,
	Question,
	Dollar,
	Ampersand,
	/// `|`
	Pipe,
	Plus,
	Asterisk,
	/// `**`
	Asterisk2,
	/// `/`
	Slash,
	/// `^`
	Caret,
	Percent,
	Underscore,
	Period,
	/// `..`
	Period2,
	/// `...`, a.k.a. ellipsis.
	Period3,
	/// `..=`
	Period2Eq,
	Colon,
	/// `::`
	Colon2,
	Eq,
	/// `==`
	Eq2,
	/// `=>`
	FatArrow,
	/// `!`
	Bang,
	/// `!=`
	BangEq,
	/// `~==`
	GraveEq,
	/// `~!=`
	GraveBangEq,
	Minus,
	/// `->`
	ThinArrow,
	/// `<=`
	LAngleEq,
	/// `>=`
	RAngleEq,
	/// `+=`
	PlusEq,
	/// `-=`
	MinusEq,
	/// `|=`
	PipeEq,
	/// `&=`
	AmpersandEq,
	/// `^=`
	CaretEq,
	/// `/=`
	SlashEq,
	/// `*=`
	AsteriskEq,
	/// `**=`
	Asterisk2Eq,
	/// `%=`
	PercentEq,
	/// `&&`
	Ampersand2,
	/// `||`
	Pipe2,
	/// `^^`
	Caret2,
	/// `&&=`
	Ampersand2Eq,
	/// `||=`
	Pipe2Eq,
	/// `^^`
	Caret2Eq,
	/// `<<`
	LeftAngle2,
	/// `>>`
	RightAngle2,
	/// `>>>`
	RightAngle3,
	/// `<<=`
	LeftAngle2Eq,
	/// `>>=`
	RightAngle2Eq,
	/// `>>>=`
	RightAngle3Eq,
	/// `++`
	Plus2,
	/// `--`
	Minus2,
	/// `is` (a specialized operator).
	Is,
	/// `!is` (a specialized operator).
	IsNot,

	// Literals ////////////////////////////////////////////////////////////////
	LitNull,
	LitFalse,
	LitTrue,
	LitInt,
	LitFloat,
	LitChar,
	LitString,

	// Keywords ////////////////////////////////////////////////////////////////
	KwAbstract,
	KwBitfield,
	KwBreak,
	KwCase,
	KwCEval,
	KwClass,
	KwConst,
	KwContinue,
	KwDefault,
	KwDo,
	KwElse,
	KwEnum,
	KwExtend,
	KwFinal,
	KwFor,
	KwIf,
	KwIn,
	KwLet,
	KwLoop,
	KwMixin,
	KwOut,
	KwOverride,
	KwPrivate,
	KwProtected,
	KwPublic,
	KwReturn,
	KwStatic,
	KwStruct,
	KwSwitch,
	KwUnion,
	KwUsing,
	KwVirtual,
	KwWhile,

	// Preprocessor ////////////////////////////////////////////////////////////
	/// The exact string `#include`.
	PreprocInclude,
	/// The exact string `#namespace`.
	PreprocNamespace,
	/// The exact string `#edition`.
	PreprocEdition,

	// Higher-level composites /////////////////////////////////////////////////
	ArgList,
	Annotation,
	BitfieldDef,
	BitfieldSubfield,
	Block,
	BlockLabel,
	ClassDef,
	ClassExt,
	Constant,
	EnumDef,
	EnumExt,
	EnumVariant,
	ExprArray,
	ExprBinary,
	ExprCall,
	ExprField,
	ExprIdent,
	ExprIndex,
	ExprLambda,
	ExprLiteral,
	ExprMethodCall,
	ExprPostfix,
	ExprPrefix,
	ExprStruct,
	/// e.g. `x = cond ? a : b`
	ExprTernary,
	/// A type expression may be a resolver, an array descriptor, a tuple
	/// descriptor, or `_` to make the compiler attempt inferrence.
	ExprType,
	FieldDecl,
	FunctionDef,
	Identifier,
	Literal,
	MacroInvoc,
	ParamList,
	/// A preprocessor directive and its "arguments".
	Preproc,
	StatBinding,
	StatBreak,
	StatContinue,
	/// e.g. `;`
	StatEmpty,
	StatExpr,
	StatIf,
	StatLoopDoUntil,
	StatLoopDoWhile,
	StatLoopFor,
	StatLoopInfinite,
	StatLoopWhile,
	StatReturn,
	StatSwitch,
	StructDef,
	StructExt,
	ResolverPart,
	Resolver,
	TypeAlias,
	UnionDef,
	UnionExt,
	UnionVariant,

	// Miscellaneous ///////////////////////////////////////////////////////////
	Whitespace,
	/// Input that the lexer considered to be invalid.
	Unknown,
	/// The top-level node.
	Root, // Ensure this is always the last variant!
}
use SyntaxKind as Syn;

impl From<SyntaxKind> for rowan::SyntaxKind {
	fn from(value: SyntaxKind) -> Self {
		Self(value as u16)
	}
}

impl rowan::Language for SyntaxKind {
	type Kind = Self;

	fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
		assert!(raw.0 <= Self::Root as u16);
		unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
	}

	fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
		kind.into()
	}
}

#[must_use]
pub fn lex(source: &str) -> Option<TokenStream> {
	if source.is_empty() {
		return None;
	}

	let (tokens, errs) = lexer().parse_recovery(source);

	tokens.map(|t| TokenStream {
		tokens: t,
		errors: errs,
	})
}

pub type Error = chumsky::prelude::Simple<char>;

#[derive(Clone, PartialEq)]
pub struct TokenStream {
	pub tokens: Vec<Token>,
	pub errors: Vec<Error>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
	pub kind: SyntaxKind,
	pub span: Range<usize>,
}

/// An implementation detail of [`lex`], but public so that the emitted
/// combinator lexer can be re-appropriated if need be.
pub fn lexer() -> impl Parser<char, Vec<Token>, Error = Error> {
	let whitespace = text::whitespace::<_, Error>().map_with_span(|_, sp| Token {
		kind: Syn::Whitespace,
		span: sp,
	});

	let ident = text::ident::<_, Error>().map_with_span(|_, sp| Token {
		kind: Syn::Identifier,
		span: sp,
	});

	let unknown = primitive::none_of(&[' ', '\n', '\t', '\r'])
		.repeated()
		.at_least(1)
		.map_with_span(|_, sp| Token {
			kind: Syn::Unknown,
			span: sp,
		});

	primitive::choice::<_, Error>((
		whitespace,
		lexer_keyword_control(),
		lexer_keyword_item(),
		lexer_keyword_qual(),
		lexer_glyph_un(),
		lexer_glyph_bin_arith(),
		lexer_glyph_bin_bit(),
		lexer_glyph_bin_compare(),
		lexer_glyph_bin_logic(),
		lexer_glyph_bin_misc(),
		lexer_literal(),
		ident,
		unknown,
	))
	.repeated()
	.then_ignore(primitive::end())
}

// It's necessary to break up these lexers because tuples stop meeting chumsky's
// trait requirements once they grow past a certain size.
// [Rat] Rust is sorely missing variadic generics or something similar

#[must_use]
fn lexer_keyword_control() -> impl Parser<char, Token, Error = Error> {
	primitive::choice::<_, Error>((
		primitive::just("break").map_with_span(|_, sp| Token {
			kind: Syn::KwBreak,
			span: sp,
		}),
		primitive::just("case").map_with_span(|_, sp| Token {
			kind: Syn::KwCase,
			span: sp,
		}),
		primitive::just("continue").map_with_span(|_, sp| Token {
			kind: Syn::KwContinue,
			span: sp,
		}),
		primitive::just("default").map_with_span(|_, sp| Token {
			kind: Syn::KwDefault,
			span: sp,
		}),
		primitive::just("do").map_with_span(|_, sp| Token {
			kind: Syn::KwDo,
			span: sp,
		}),
		primitive::just("else").map_with_span(|_, sp| Token {
			kind: Syn::KwElse,
			span: sp,
		}),
		primitive::just("for").map_with_span(|_, sp| Token {
			kind: Syn::KwFor,
			span: sp,
		}),
		primitive::just("if").map_with_span(|_, sp| Token {
			kind: Syn::KwIf,
			span: sp,
		}),
		primitive::just("let").map_with_span(|_, sp| Token {
			kind: Syn::KwLet,
			span: sp,
		}),
		primitive::just("loop").map_with_span(|_, sp| Token {
			kind: Syn::KwLoop,
			span: sp,
		}),
		primitive::just("return").map_with_span(|_, sp| Token {
			kind: Syn::KwReturn,
			span: sp,
		}),
		primitive::just("switch").map_with_span(|_, sp| Token {
			kind: Syn::KwSwitch,
			span: sp,
		}),
		primitive::just("while").map_with_span(|_, sp| Token {
			kind: Syn::KwWhile,
			span: sp,
		}),
	))
}

#[must_use]
fn lexer_keyword_item() -> impl Parser<char, Token, Error = Error> {
	primitive::choice::<_, Error>((
		primitive::just("bitfield").map_with_span(|_, sp| Token {
			kind: Syn::KwBitfield,
			span: sp,
		}),
		primitive::just("class").map_with_span(|_, sp| Token {
			kind: Syn::KwClass,
			span: sp,
		}),
		primitive::just("enum").map_with_span(|_, sp| Token {
			kind: Syn::KwEnum,
			span: sp,
		}),
		primitive::just("extend").map_with_span(|_, sp| Token {
			kind: Syn::KwExtend,
			span: sp,
		}),
		primitive::just("mixin").map_with_span(|_, sp| Token {
			kind: Syn::KwMixin,
			span: sp,
		}),
		primitive::just("struct").map_with_span(|_, sp| Token {
			kind: Syn::KwStruct,
			span: sp,
		}),
		primitive::just("using").map_with_span(|_, sp| Token {
			kind: Syn::KwUsing,
			span: sp,
		}),
		primitive::just("union").map_with_span(|_, sp| Token {
			kind: Syn::KwUnion,
			span: sp,
		}),
	))
}

#[must_use]
fn lexer_keyword_qual() -> impl Parser<char, Token, Error = Error> {
	primitive::choice::<_, Error>((
		primitive::just("abstract").map_with_span(|_, sp| Token {
			kind: Syn::KwAbstract,
			span: sp,
		}),
		primitive::just("ceval").map_with_span(|_, sp| Token {
			kind: Syn::KwCEval,
			span: sp,
		}),
		primitive::just("const").map_with_span(|_, sp| Token {
			kind: Syn::KwConst,
			span: sp,
		}),
		primitive::just("final").map_with_span(|_, sp| Token {
			kind: Syn::KwFinal,
			span: sp,
		}),
		primitive::just("in").map_with_span(|_, sp| Token {
			kind: Syn::KwIn,
			span: sp,
		}),
		primitive::just("out").map_with_span(|_, sp| Token {
			kind: Syn::KwOut,
			span: sp,
		}),
		primitive::just("override").map_with_span(|_, sp| Token {
			kind: Syn::KwOverride,
			span: sp,
		}),
		primitive::just("private").map_with_span(|_, sp| Token {
			kind: Syn::KwPrivate,
			span: sp,
		}),
		primitive::just("protected").map_with_span(|_, sp| Token {
			kind: Syn::KwPrivate,
			span: sp,
		}),
		primitive::just("public").map_with_span(|_, sp| Token {
			kind: Syn::KwPublic,
			span: sp,
		}),
		primitive::just("static").map_with_span(|_, sp| Token {
			kind: Syn::KwStatic,
			span: sp,
		}),
		primitive::just("virtual").map_with_span(|_, sp| Token {
			kind: Syn::KwVirtual,
			span: sp,
		}),
	))
}

#[must_use]
fn lexer_glyph_un() -> impl Parser<char, Token, Error = Error> {
	primitive::choice::<_, Error>((
		primitive::just("++").map_with_span(|_, sp| Token {
			kind: Syn::Plus2,
			span: sp,
		}),
		primitive::just("+").map_with_span(|_, sp| Token {
			kind: Syn::Plus,
			span: sp,
		}),
		primitive::just("--").map_with_span(|_, sp| Token {
			kind: Syn::Minus2,
			span: sp,
		}),
		primitive::just("-").map_with_span(|_, sp| Token {
			kind: Syn::Minus,
			span: sp,
		}),
		primitive::just("!").map_with_span(|_, sp| Token {
			kind: Syn::Bang,
			span: sp,
		}),
		primitive::just("~").map_with_span(|_, sp| Token {
			kind: Syn::Grave,
			span: sp,
		}),
	))
}

#[must_use]
fn lexer_glyph_bin_misc() -> impl Parser<char, Token, Error = Error> {
	primitive::choice::<_, Error>((
		primitive::just("::").map_with_span(|_, sp| Token {
			kind: Syn::Colon2,
			span: sp,
		}),
		primitive::just("!is").map_with_span(|_, sp| Token {
			kind: Syn::IsNot,
			span: sp,
		}),
		primitive::just("is").map_with_span(|_, sp| Token {
			kind: Syn::Is,
			span: sp,
		}),
		primitive::just("..=").map_with_span(|_, sp| Token {
			kind: Syn::Period2Eq,
			span: sp,
		}),
		primitive::just("..").map_with_span(|_, sp| Token {
			kind: Syn::Period2,
			span: sp,
		}),
		primitive::just(".").map_with_span(|_, sp| Token {
			kind: Syn::Period,
			span: sp,
		}),
		primitive::just("=").map_with_span(|_, sp| Token {
			kind: Syn::Eq,
			span: sp,
		}),
	))
}

#[must_use]
fn lexer_glyph_bin_arith() -> impl Parser<char, Token, Error = Error> {
	primitive::choice::<_, Error>((
		// Compound assignment /////////////////////////////////////////////////
		primitive::just("%=").map_with_span(|_, sp| Token {
			kind: Syn::PercentEq,
			span: sp,
		}),
		primitive::just("**=").map_with_span(|_, sp| Token {
			kind: Syn::Asterisk2Eq,
			span: sp,
		}),
		primitive::just("*=").map_with_span(|_, sp| Token {
			kind: Syn::AsteriskEq,
			span: sp,
		}),
		primitive::just("/=").map_with_span(|_, sp| Token {
			kind: Syn::SlashEq,
			span: sp,
		}),
		primitive::just("-=").map_with_span(|_, sp| Token {
			kind: Syn::MinusEq,
			span: sp,
		}),
		primitive::just("+=").map_with_span(|_, sp| Token {
			kind: Syn::PlusEq,
			span: sp,
		}),
		// Non-compound ////////////////////////////////////////////////////////
		primitive::just("%").map_with_span(|_, sp| Token {
			kind: Syn::Percent,
			span: sp,
		}),
		primitive::just("**").map_with_span(|_, sp| Token {
			kind: Syn::Asterisk2,
			span: sp,
		}),
		primitive::just("*").map_with_span(|_, sp| Token {
			kind: Syn::Asterisk,
			span: sp,
		}),
		primitive::just("/").map_with_span(|_, sp| Token {
			kind: Syn::Slash,
			span: sp,
		}),
		primitive::just("-").map_with_span(|_, sp| Token {
			kind: Syn::Minus,
			span: sp,
		}),
		primitive::just("+").map_with_span(|_, sp| Token {
			kind: Syn::Plus,
			span: sp,
		}),
	))
}

#[must_use]
fn lexer_glyph_bin_bit() -> impl Parser<char, Token, Error = Error> {
	primitive::choice::<_, Error>((
		// Compound assignment /////////////////////////////////////////////////
		primitive::just("<<=").map_with_span(|_, sp| Token {
			kind: Syn::LeftAngle2Eq,
			span: sp,
		}),
		primitive::just(">>=").map_with_span(|_, sp| Token {
			kind: Syn::RightAngle2Eq,
			span: sp,
		}),
		primitive::just(">>>=").map_with_span(|_, sp| Token {
			kind: Syn::RightAngle3Eq,
			span: sp,
		}),
		primitive::just("&=").map_with_span(|_, sp| Token {
			kind: Syn::AmpersandEq,
			span: sp,
		}),
		primitive::just("|=").map_with_span(|_, sp| Token {
			kind: Syn::PipeEq,
			span: sp,
		}),
		primitive::just("^=").map_with_span(|_, sp| Token {
			kind: Syn::CaretEq,
			span: sp,
		}),
		// Non-compound ////////////////////////////////////////////////////////
		primitive::just("<<").map_with_span(|_, sp| Token {
			kind: Syn::LeftAngle2,
			span: sp,
		}),
		primitive::just(">>").map_with_span(|_, sp| Token {
			kind: Syn::RightAngle2,
			span: sp,
		}),
		primitive::just(">>>").map_with_span(|_, sp| Token {
			kind: Syn::RightAngle3,
			span: sp,
		}),
		primitive::just("&").map_with_span(|_, sp| Token {
			kind: Syn::Ampersand,
			span: sp,
		}),
		primitive::just("|").map_with_span(|_, sp| Token {
			kind: Syn::Pipe,
			span: sp,
		}),
		primitive::just("^").map_with_span(|_, sp| Token {
			kind: Syn::Caret,
			span: sp,
		}),
	))
}

#[must_use]
fn lexer_glyph_bin_logic() -> impl Parser<char, Token, Error = Error> {
	primitive::choice::<_, Error>((
		// Compound assignment /////////////////////////////////////////////////
		primitive::just("&&=").map_with_span(|_, sp| Token {
			kind: Syn::Ampersand2Eq,
			span: sp,
		}),
		primitive::just("||=").map_with_span(|_, sp| Token {
			kind: Syn::Pipe2Eq,
			span: sp,
		}),
		primitive::just("^^=").map_with_span(|_, sp| Token {
			kind: Syn::Caret2Eq,
			span: sp,
		}),
		// Non-compound ////////////////////////////////////////////////////////
		primitive::just("&&").map_with_span(|_, sp| Token {
			kind: Syn::Ampersand2,
			span: sp,
		}),
		primitive::just("||").map_with_span(|_, sp| Token {
			kind: Syn::Pipe2,
			span: sp,
		}),
		primitive::just("^^").map_with_span(|_, sp| Token {
			kind: Syn::Caret2,
			span: sp,
		}),
	))
}

#[must_use]
fn lexer_glyph_bin_compare() -> impl Parser<char, Token, Error = Error> {
	primitive::choice::<_, Error>((
		primitive::just("~==").map_with_span(|_, sp| Token {
			kind: Syn::GraveEq,
			span: sp,
		}),
		primitive::just("~!=").map_with_span(|_, sp| Token {
			kind: Syn::GraveBangEq,
			span: sp,
		}),
		primitive::just("==").map_with_span(|_, sp| Token {
			kind: Syn::Eq2,
			span: sp,
		}),
		primitive::just("<=").map_with_span(|_, sp| Token {
			kind: Syn::LAngleEq,
			span: sp,
		}),
		primitive::just(">=").map_with_span(|_, sp| Token {
			kind: Syn::RAngleEq,
			span: sp,
		}),
		primitive::just("<").map_with_span(|_, sp| Token {
			kind: Syn::LAngle,
			span: sp,
		}),
		primitive::just(">").map_with_span(|_, sp| Token {
			kind: Syn::RAngle,
			span: sp,
		}),
	))
}

#[must_use]
fn lexer_literal() -> impl Parser<char, Token, Error = Error> {
	primitive::choice::<_, Error>((
		primitive::just("null").map_with_span(|_, sp| Token {
			kind: Syn::Literal,
			span: sp,
		}),
		primitive::just("true").map_with_span(|_, sp| Token {
			kind: Syn::Literal,
			span: sp,
		}),
		primitive::just("false").map_with_span(|_, sp| Token {
			kind: Syn::Literal,
			span: sp,
		}),
		// Soon!
	))
}
