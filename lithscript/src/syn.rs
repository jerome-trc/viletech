//! A [syntax tag type] with a procedurally-generated lexer for its tokens.
//!
//! [syntax tag type]: Syn

use doomfront::rowan;
use logos::Logos;

use crate::Version;

/// A stronger type over [`rowan::SyntaxKind`] representing all kinds of syntax elements.
#[repr(u16)]
#[derive(Logos, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[logos(extras = Version, error = Syn)]
#[allow(clippy::manual_non_exhaustive)]
pub enum Syn {
	// Nodes: high-level composites ////////////////////////////////////////////
	/// `'#' '!'? '[' resolver arglist? ']'`
	Annotation,
	/// `'(' expr? (',' expr)* ')'`
	ArgList,
	/// `blocklabel? '{' statement* '}'`
	Block,
	/// `'::' ident '::'`
	BlockLabel,
	/// A sequence of tokens that did not form a valid syntax element.
	Error,
	/// A top-level node representing a whole file.
	FileRoot,
	/// `annotation* 'func' ident paramlist returntype? (funcbody | ';')`
	FuncDecl,
	/// `'{' statement* '}'`
	FuncBody,
	/// `ident ('.' ident)*`
	IdentChain,
	/// `'(' param? (',' param)* ')'`
	ParamList,
	/// `ident typespec`
	Parameter,
	/// A top-level node representing a whole REPL submission.
	ReplRoot,
	/// `':' expr`
	TypeSpec,
	// Nodes: statements ///////////////////////////////////////////////////////
	/// `'ceval'? ('let' | 'const') 'ident' typespec? ('=' expr)? ';'`
	BindStat,
	/// `'break' blocklabel? ';'`
	BreakStat,
	/// `'continue' blocklabel? ';'`
	ContinueStat,
	/// `expr ';'`
	ExprStat,
	/// `'return' expr? ';'`
	ReturnStat,
	// Nodes: expressions //////////////////////////////////////////////////////
	ArrayExpr,
	BinExpr,
	/// `expr arglist`
	CallExpr,
	/// `expr '.' ident`
	FieldExpr,
	/// `'(' expr ')'`
	GroupExpr,
	/// Is parent to only a [`Syn::Ident`] token.
	IdentExpr,
	IndexExpr,
	/// Will have a single child token tagged as one of the following:
	/// - [`Syn::FalseLit`]
	/// - [`Syn::FloatLit`]
	/// - [`Syn::IntLit`]
	/// - [`Syn::StringLit`]
	/// - [`Syn::TrueLit`]
	Literal,
	/// `expr '.' ident arglist`
	MethodExpr,
	StructExpr,
	PostfixExpr,
	PrefixExpr,
	// Tokens: literals ////////////////////////////////////////////////////////
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
	#[token("true")]
	TrueLit,
	// Tokens: keywords ////////////////////////////////////////////////////////
	#[token("break", priority = 5)]
	KwBreak,
	#[token("ceval", priority = 5)]
	KwCeval,
	#[token("const", priority = 5)]
	KwConst,
	#[token("continue", priority = 5)]
	KwContinue,
	#[token("else", priority = 5)]
	KwElse,
	#[token("for", priority = 5)]
	KwFor,
	#[token("func", priority = 5)]
	KwFunc,
	#[token("if", priority = 5)]
	KwIf,
	#[token("let", priority = 5)]
	KwLet,
	#[token("return", priority = 5)]
	KwReturn,
	#[token("static", priority = 5)]
	KwStatic,
	#[token("struct", priority = 5)]
	KwStruct,
	#[token("while", priority = 5)]
	KwWhile,
	// Tokens: glyphs //////////////////////////////////////////////////////////
	#[token("&")]
	Ampersand,
	#[token("&&")]
	Ampersand2,
	#[token("&=")]
	AmpersandEq,
	#[token("<")]
	AngleL,
	#[token(">")]
	AngleR,
	#[token("<<")]
	AngleL2,
	#[token(">>")]
	AngleR2,
	#[token("<<=")]
	AngleL2Eq,
	#[token(">>=")]
	AngleR2Eq,
	#[token("<=")]
	AngleLEq,
	#[token(">=")]
	AngleREq,
	#[token("*")]
	Asterisk,
	#[token("*=")]
	AsteriskEq,
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
	#[token("^")]
	Caret,
	#[token("^=")]
	CaretEq,
	#[token(":")]
	Colon,
	#[token("::")]
	Colon2,
	#[token(",")]
	Comma,
	#[token(".")]
	Dot,
	#[token("=")]
	Eq,
	#[token("==")]
	Eq2,
	#[token("-")]
	Minus,
	#[token("-=")]
	MinusEq,
	#[token("(")]
	ParenL,
	#[token(")")]
	ParenR,
	#[token("%")]
	Percent,
	#[token("%=")]
	PercentEq,
	#[token("|")]
	Pipe,
	#[token("||")]
	Pipe2,
	#[token("|=")]
	PipeEq,
	#[token("+")]
	Plus,
	#[token("+=")]
	PlusEq,
	#[token("#")]
	Pound,
	#[token(";")]
	Semicolon,
	#[token("/")]
	Slash,
	#[token("/=")]
	SlashEq,
	#[token("~")]
	Tilde,
	// Tokens: miscellaenous ///////////////////////////////////////////////////
	#[regex("[a-zA-Z_][a-zA-Z0-9_]*", priority = 4)]
	Ident,
	#[regex("r#[a-zA-Z0-9_][a-zA-Z0-9_]*")]
	IdentRaw,
	#[regex(r#"///([^/][^\n]*)?"#, priority = 2)]
	DocComment,
	/// Either single-line (C++-style) or multi-line (C-style).
	#[regex("//[^\n]*\n*", priority = 1)]
	#[regex("//")]
	#[regex(r"/[*]([^*]|([*][^/]))*[*]+/")]
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
	__Last,
}

impl Syn {
	#[must_use]
	pub fn is_trivia(self) -> bool {
		matches!(self, Self::Whitespace | Self::Comment)
	}

	/// For parser error reporting. Likely to go away eventually.
	pub(crate) fn pretty(self) -> &'static str {
		match self {
			// Literals ////////////////////////////////////////////////////////
			Self::FalseLit => "`false`",
			Self::FloatLit => "a floating-point number",
			Self::IntLit => "an integer",
			Self::StringLit => "a string",
			Self::TrueLit => "`true`",
			// Keywords ////////////////////////////////////////////////////////
			Self::KwBreak => "`break`",
			Self::KwCeval => "`ceval`",
			Self::KwConst => "`const`",
			Self::KwContinue => "`continue`",
			Self::KwElse => "`else`",
			Self::KwFor => "`for`",
			Self::KwFunc => "`func`",
			Self::KwIf => "`if`",
			Self::KwLet => "`let`",
			Self::KwReturn => "`return`",
			Self::KwStatic => "`static`",
			Self::KwStruct => "`struct`",
			Self::KwWhile => "`while`",
			// Glyphs //////////////////////////////////////////////////////////
			Self::Ampersand => "`&`",
			Self::Ampersand2 => "`&&`",
			Self::AmpersandEq => "`&=`",
			Self::AngleL => "`<`",
			Self::AngleR => "`>`",
			Self::AngleL2 => "`<<`",
			Self::AngleR2 => "`>>`",
			Self::AngleL2Eq => "`<<=`",
			Self::AngleR2Eq => "`>>=`",
			Self::AngleLEq => "`<=`",
			Self::AngleREq => "`>=`",
			Self::Asterisk => "`*`",
			Self::AsteriskEq => "`*=`",
			Self::Bang => "`!`",
			Self::BangEq => "`!=`",
			Self::BraceL => "`{`",
			Self::BraceR => "`}`",
			Self::BracketL => "`[`",
			Self::BracketR => "`]`",
			Self::Caret => "`^`",
			Self::CaretEq => "`^=`",
			Self::Colon => "`:`",
			Self::Colon2 => "`::`",
			Self::Comma => "`,`",
			Self::Dot => "`.`",
			Self::Eq => "`=`",
			Self::Eq2 => "`==`",
			Self::Minus => "`-`",
			Self::MinusEq => "`-=`",
			Self::ParenL => "`(`",
			Self::ParenR => "`)`",
			Self::Pipe => "`|`",
			Self::Pipe2 => "`||`",
			Self::Plus => "`+`",
			Self::PlusEq => "`+=`",
			Self::Pound => "`#`",
			Self::Semicolon => "`;`",
			Self::Slash => "`/`",
			Self::SlashEq => "`/=`",
			Self::Tilde => "`~`",
			// Miscellaneous ///////////////////////////////////////////////////
			Self::Ident | Self::IdentRaw => "an identifier",
			Self::Comment => "a comment",
			Self::Whitespace => "whitespace",
			other => unreachable!("called `Syn::pretty` illegally: {other:#?}"),
		}
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

#[cfg(test)]
#[test]
fn smoke() {
	const SOURCE: &str = "typeof(9 + 10)";

	const EXPECTED: &[Syn] = &[
		Syn::Ident,
		Syn::ParenL,
		Syn::IntLit,
		Syn::Whitespace,
		Syn::Plus,
		Syn::Whitespace,
		Syn::IntLit,
		Syn::ParenR,
	];

	let lexer = Syn::lexer_with_extras(SOURCE, Version::new(0, 0, 0));

	for (i, token) in lexer.enumerate() {
		assert_eq!(token.unwrap(), EXPECTED[i]);
	}
}
