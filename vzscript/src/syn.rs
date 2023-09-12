//! A [syntax tag type](Syn) with a macro-generated lexer for its tokens.

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
	/// `'#' ident arglist?`
	Annotation,
	/// `'[' nontypeexpr ']'`
	///
	/// Can start a [`Syn::TypeExpr`].
	ArrayPrefix,
	/// `'(' argument? (',' argument)* ')'`
	ArgList,
	/// `(ident ':')? expr`
	Argument,
	/// `'#' '[' ident arglist? ']'`
	Attribute,
	/// `blocklabel? '{' T* '}'` where `T` is a statement, [`Syn::Annotation`], or item.
	Block,
	/// `'::' ident '::'`
	BlockLabel,
	/// `'class' ident typespec? typequals '{' structinnard* '}'`
	ClassDef,
	/// `'const' ident typespec? '=' expr ';'`
	ConstDef,
	/// `'enum' ident typespec? '{' variant? (',' variant)* ','? '}'`
	EnumDef,
	/// `ident ('=' expr)?`
	EnumVariant,
	/// A sequence of tokens that did not form a valid syntax element.
	Error,
	/// `memberquals ident typespec ';'`
	FieldDecl,
	/// `'{' T* '}'` where `T` is a statement, [`Syn::Annotation`], or item.
	FuncBody,
	/// `memberquals 'function' ident paramlist returntype? block?`
	FuncDecl,
	/// A top-level node representing a whole file.
	FileRoot,
	/// TODO
	MemberQuals,
	/// `'.'? T | ('.'? T ('.' T)*)` where `T` is a [`Syn::Ident`] or [`Syn::NameLit`].
	///
	/// Counterpart to what is known in ZScript's grammar as a "dottable ID".
	NameChain,
	/// `'?'`
	///
	/// Can start a [`Syn::TypeExpr`].
	OptionPrefix,
	/// `'(' param? (',' param)* ')'`
	ParamList,
	/// `ident typespec ('=' expr)?`
	Parameter,
	/// `'&'`
	///
	/// Can start a [`Syn::TypeExpr`].
	RefPrefix,
	/// `'struct' ident typequals '{' structinnard* '}'`
	StructDef,
	/// `'default' ':' block | 'case' expr ':' block`
	SwitchCase,
	/// `':' expr`
	///
	/// A "type specifier".
	TypeSpec,
	/// `'union' ident '{' unioninnard* '}'`
	UnionDef,
	/// `ident '{' fielddecl* '}'`
	UnionVariant,
	// Nodes: statements ///////////////////////////////////////////////////////
	/// `('let' | 'readonly') 'ident' typespec? ('=' expr)? ';'`
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
	/// `'[' (expr ',')* ']'`
	ArrayExpr,
	/// `expr operator expr`
	BinExpr,
	/// `'{' statement* '}'`
	BlockExpr,
	/// `expr arglist`
	CallExpr,
	/// `class typespec? '{' classinnard* '}'`
	ClassExpr,
	/// `'.' '{' assignment* '}'`
	ConstructExpr,
	/// `'enum' typespec? '{' variant? (',' variant)* ','? '}'`
	EnumExpr,
	/// `'@' paramlist block`
	FunctionExpr,
	/// ident | ('.' ident)
	IdentExpr,
	/// `'if' expr block`
	IfExpr,
	/// `expr '.' ident`
	FieldExpr,
	/// `'for' ident ':' expr block`
	ForExpr,
	/// `'(' expr ')'`
	GroupExpr,
	/// `expr '[' expr ']'`
	IndexExpr,
	/// Parent to a single token of one of the following kinds:
	/// - [`Syn::FalseLit`]
	/// - [`Syn::FloatLit`]
	/// - [`Syn::IntLit`]
	/// - [`Syn::NameLit`]
	/// - [`Syn::NullLit`]
	/// - [`Syn::StringLit`]
	/// - [`Syn::TrueLit`]
	Literal,
	/// `expr operator`
	PostfixExpr,
	/// `operator expr`
	PrefixExpr,
	/// `'struct' '{' structinnard* '}'`
	StructExpr,
	/// `'switch' expr '{' switchcase* '}'`
	SwitchExpr,
	/// `'union' '{' unioninnard* '}'`
	UnionExpr,
	/// One of the following:
	/// - [`Syn::KwType`]
	/// - [`Syn::KwAuto`]
	/// - One or more type expr. prefixes followed by any other kind of expression.
	TypeExpr,
	/// `@ namechain '{' unionfield* '}'`
	VariantExpr,
	/// `'while' expr block`
	WhileExpr,
	// Tokens: literals ////////////////////////////////////////////////////////
	#[token("false")]
	FalseLit,
	#[regex(r"[0-9][0-9_]*([Ee][+-]?[0-9]+)[fF]?", priority = 4)]
	#[regex(r"[0-9]*\.[0-9_]+([Ee][+-]?[0-9]+)?[fF]?", priority = 3)]
	#[regex(r"[0-9][0-9_]*\.[0-9_]*([Ee][+-]?[0-9]+)?[fF]?", priority = 2)]
	FloatLit,
	#[regex("0[xX][a-fA-F0-9_]+([iIuU]8|16|32|64|128)?", priority = 4)]
	#[regex(r"0[0-9_]+([iIuU]8|16|32|64|128)?", priority = 3)]
	#[regex(r"[0-9][0-9_]*([iIuU]8|16|32|64|128)?", priority = 2)]
	IntLit,
	#[regex("'[^''\n]*'")]
	NameLit,
	#[token("null")]
	NullLit,
	/// `[gG]` for glob, `[xX]` for regex, `[fF]` for interpolated formatting.
	// TODO: raw string literals?
	#[regex(r#"[gGxX]?[fF]?"(([\\]["])|[^"])*""#, priority = 3)]
	StringLit,
	#[token("true")]
	TrueLit,
	// Tokens: keywords ////////////////////////////////////////////////////////
	#[token("abstract", priority = 5)]
	KwAbstract,
	#[token("auto", priority = 5)]
	KwAuto,
	#[token("break", priority = 5)]
	KwBreak,
	#[token("case", priority = 5)]
	KwCase,
	#[token("class", priority = 5)]
	KwClass,
	#[token("const", priority = 5)]
	KwConst,
	#[token("continue", priority = 5)]
	KwContinue,
	#[token("default", priority = 5)]
	KwDefault,
	#[token("do", priority = 5)]
	KwDo,
	#[token("else", priority = 5)]
	KwElse,
	#[token("enum", priority = 5)]
	KwEnum,
	#[token("extend", priority = 5)]
	KwExtend,
	#[token("for", priority = 5)]
	KwFor,
	#[token("final", priority = 5)]
	KwFinal,
	#[token("function", priority = 5)]
	KwFunction,
	#[token("if", priority = 5)]
	KwIf,
	#[token("is", priority = 5)]
	KwIs,
	#[token("isnot", priority = 5)]
	KwIsNot,
	#[token("let", priority = 5)]
	KwLet,
	#[token("override", priority = 5)]
	KwOverride,
	#[token("private", priority = 5)]
	KwPrivate,
	#[token("protected", priority = 5)]
	KwProtected,
	#[token("readonly", priority = 5)]
	KwReadonly,
	#[token("return", priority = 5)]
	KwReturn,
	#[token("static", priority = 5)]
	KwStatic,
	#[token("struct", priority = 5)]
	KwStruct,
	#[token("super", priority = 5)]
	KwSuper,
	#[token("switch", priority = 5)]
	KwSwitch,
	#[token("@type", priority = 5)]
	KwType,
	#[token("union", priority = 5)]
	KwUnion,
	#[token("virtual", priority = 5)]
	KwVirtual,
	#[token("while", priority = 5)]
	KwWhile,
	// Tokens: glyphs //////////////////////////////////////////////////////////
	#[token("&")]
	Ampersand,
	#[token("&&")]
	Ampersand2,
	#[token("&&=")]
	Ampersand2Eq,
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
	#[token("**")]
	Asterisk2,
	#[token("**=")]
	Asterisk2Eq,
	#[token("*=")]
	AsteriskEq,
	#[token("@")]
	At,
	#[token("@@")]
	At2,
	#[token("@[")]
	AtBracketL,
	#[token("@(")]
	AtParenL,
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
	#[token("..")]
	Dot2,
	#[token("..=")]
	Dot2Eq,
	#[token("...")]
	Dot3,
	#[token(".{")]
	DotBraceL,
	#[token("=")]
	Eq,
	#[token("==")]
	Eq2,
	#[token("-")]
	Minus,
	#[token("--")]
	Minus2,
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
	#[token("||=")]
	Pipe2Eq,
	#[token("|=")]
	PipeEq,
	#[token("+")]
	Plus,
	#[token("++")]
	Plus2,
	#[token("+=")]
	PlusEq,
	#[token("#")]
	Pound,
	#[token("#[")]
	PoundBracketL,
	#[token("?")]
	Question,
	#[token("??")]
	Question2,
	#[token("??=")]
	Question2Eq,
	#[token("?.")]
	QuestionDot,
	#[token(";")]
	Semicolon,
	#[token("/")]
	Slash,
	#[token("/=")]
	SlashEq,
	#[token("=>")]
	ThickArrow,
	#[token("->")]
	ThinArrow,
	#[token("~")]
	Tilde,
	#[token("~!=")]
	TildeBangEq,
	#[token("~==")]
	TildeEq2,
	// Tokens: miscellaenous ///////////////////////////////////////////////////
	/// Same rules as C identifiers.
	#[regex("[a-zA-Z_][a-zA-Z0-9_]*", priority = 4)]
	Ident,
	#[regex(r#"///([^/][^\n]*)?"#, priority = 2)]
	DocComment,
	/// Either single-line (C++-style) or multi-line (C-style).
	#[regex("//[^/\n]*\n*", priority = 1)]
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

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn smoke() {
		const SOURCE: &str = "9_._0.abs() .{ typeof(9_i8 + '9a')";

		const EXPECTED: &[Syn] = &[
			Syn::FloatLit,
			Syn::Dot,
			Syn::Ident,
			Syn::ParenL,
			Syn::ParenR,
			Syn::Whitespace,
			Syn::DotBraceL,
			Syn::Whitespace,
			Syn::Ident,
			Syn::ParenL,
			Syn::IntLit,
			Syn::Whitespace,
			Syn::Plus,
			Syn::Whitespace,
			Syn::NameLit,
			Syn::ParenR,
		];

		let lexer = Syn::lexer_with_extras(SOURCE, Version::new(0, 0, 0));

		for (i, token) in lexer.enumerate() {
			assert_eq!(token.unwrap(), EXPECTED[i]);
		}
	}
}
