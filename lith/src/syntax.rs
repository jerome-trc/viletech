//! A [syntax tag type](Syntax) with a macro-generated lexer for its tokens.

use doomfront::rowan;
use logos::Logos;

/// A stronger type over [`rowan::SyntaxKind`] representing all kinds of syntax elements.
#[repr(u16)]
#[derive(Logos, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[logos(error = Syntax, extras = LexContext)]
#[allow(clippy::manual_non_exhaustive)]
pub enum Syntax {
	/// A sequence of tokens that did not form a valid syntax element.
	Error,
	/// The top-level node representing a whole file.
	FileRoot,

	// Nodes: high-level constructs ////////////////////////////////////////////
	/// Any of the following:
	/// `'.' ident '=' expr`
	/// `'[' expr ']' '=' expr`
	/// `expr`
	AggregateInit,
	/// `'#' '!'? '[' (ident '.')? ident arglist? ']'`
	Annotation,
	/// `'(' argument (',' argument)* (',' | (',' '...'))? ')'` or `'(' '...' ')'`
	///
	/// Common to [call expressions](Syntax::ExprCall) and [annotations](Syntax::Annotation).
	ArgList,
	/// `((ident | namelit) ':')? expr`
	Argument,
	/// `'[' nontypeexpr ']'`
	///
	/// Can start a [`Syntax::ExprType`].
	ArrayPrefix,
	/// `'::' ident '::'`
	BlockLabel,
	/// `ident typespec ';'`
	///
	/// Part of [struct expressions] and [union expressions].
	///
	/// [struct expressions]: Syntax::ExprStruct
	/// [union expressions]: Syntax::ExprUnion
	FieldDecl,
	/// `memberquals 'function' ident paramlist returntype? (block | ';')`
	FunctionDecl,
	/// `'{' T* '}'` where `T` is a statement, [`Syntax::Annotation`], or item.
	FunctionBody,

	/// `'const'? ('&' 'var'?)? ident typespec ('=' expr)?`
	Parameter,
	/// `'(' (param? (',' param)* ','?) | '...' ')'`
	ParamList,
	/// `'const' ident typespec '=' expr ';'`
	///
	/// A "symbolic constant". Semantically equivalent to an immutable compile-time
	/// [binding statement](Syntax::StmtBind) but permitted at container scope.
	SymConst,
	/// `':' (expr | 'any_t' | 'type_t')`
	///
	/// - Expressions in this position are forbidden from using [`Syntax::Eq`] as an infix operator.
	/// - `any_t` is only a valid parse in [parameters](Syntax::Parameter).
	TypeSpec,

	// Nodes: statements ///////////////////////////////////////////////////////
	/// `('let' | 'var') 'const'? pattern typespec? ('=' expr)? ';'`
	StmtBind,
	/// `'break' blocklabel? expr? ';'`
	StmtBreak,
	/// `'continue' blocklabel? ';'`
	StmtContinue,
	/// `expr ';'`
	///
	/// The trailing semicolon is optional if the expression
	/// ends with a curly-brace-delimited block.
	StmtExpr,
	/// `'return' expr? ';'`
	StmtReturn,

	// Nodes: patterns /////////////////////////////////////////////////////////
	/// `'(' pattern ')'`
	PatGrouped,
	/// `ident`
	PatIdent,
	/// One of the following:
	/// - `'-'? intlit` or `'-'? floatlit`
	/// - `stringlit` or `namelit`
	/// - `'true'` or `'false'`
	PatLit,
	/// `'[' (pattern (',' pattern)* ','?)? ']'`
	PatSlice,
	/// `'_'`
	PatWildcard,

	// Nodes: expressions //////////////////////////////////////////////////////
	/// `'.{' aggregateinit? (',' aggregateinit)* ','? '}'`
	ExprAggregate,
	/// `expr operator expr`
	ExprBin,
	/// `'{' T* '}'` where `T` is a statement, [`Syntax::Annotation`], or item.
	ExprBlock,
	/// `primaryexpr arglist`
	ExprCall,
	/// `expr '{' aggregateinit? (',' aggregateinit)* ','? '}'`
	ExprConstruct,
	/// `primaryexpr '.' (ident | namelit)`
	ExprField,
	/// Parent to only a single [`Syntax::Ident`] token.
	ExprIdent,
	/// `primaryexpr '[' expr ']'`
	ExprIndex,
	/// If this is not a string literal, it is parent to only one token tagged as
	/// one of the following:
	/// - [`Syntax::LitFalse`]
	/// - [`Syntax::LitFloat`]
	/// - [`Syntax::LitInt`]
	/// - [`Syntax::LitName`]
	/// - [`Syntax::LitTrue`]
	/// - [`Syntax::LitVoid`]
	/// If this is a string literal, it is parent to one token tagged as
	/// [`Syntax::LitString`] which may be followed by a [`Syntax::Ident`] suffix,
	/// with no allowance for trivia in between.
	ExprLit,
	/// `'(' expr ')'`
	ExprGroup,
	/// `expr operator`
	ExprPostfix,
	/// `operator expr`
	ExprPrefix,
	/// `expr? ('..' | '..=') expr?`
	ExprRange,
	/// `'struct' '{' coreelement* '}'`
	ExprStruct,
	/// Another kind of expression preceded by one or more type expression prefixes.
	ExprType,

	// Tokens: keywords ////////////////////////////////////////////////////////
	#[doc(hidden)]
	__FirstKeyword,

	/// `any_t`; used in [type expressions](Syntax::ExprType).
	#[token("any")]
	KwAnyT,
	/// `break`; used in [break statements](Syntax::StmtBreak).
	#[token("break")]
	KwBreak,
	/// `const`; used in [parameters] and for [symbolic constants].
	///
	/// [parameters]: Syntax::Parameter
	/// [symbolic constants]: Syntax::SymConst
	#[token("const")]
	KwConst,
	/// `continue`; used in [continue statements](Syntax::StmtContinue).
	#[token("continue")]
	KwContinue,
	/// `function`; used in [function declarations](Syntax::FunctionDecl).
	#[token("function")]
	KwFunction,
	/// `let`; used in [binding statements](Syntax::StmtBind).
	#[token("let")]
	KwLet,
	/// `return`; used in [return statements](Syntax::StmtReturn).
	#[token("return")]
	KwReturn,
	/// `struct`; used in [structure expressions](Syntax::ExprStruct).
	#[token("struct")]
	KwStruct,
	/// `type_t`; used in [type expressions](Syntax::ExprType).
	#[token("type_t")]
	KwTypeT,
	/// `var`; used in [binding statements](Syntax::StmtBind).
	#[token("var")]
	KwVar,

	#[doc(hidden)]
	__LastKeyword,

	// Tokens: glyphs //////////////////////////////////////////////////////////
	#[doc(hidden)]
	__FirstGlyph,

	/// `&`; the bit-wise AND [binary operator](Syntax::ExprBin).
	#[token("&")]
	Ampersand,
	/// `&&`; the logical AND comparison [binary operator](Syntax::ExprBin).
	#[token("&&")]
	Ampersand2,
	/// `&=`; the bit-wise AND compound assignment [binary operator](Syntax::ExprBin).
	#[token("&=")]
	AmpersandEq,
	/// `&&=`; the logical AND compound assignment [binary operator](Syntax::ExprBin).
	#[token("&&=")]
	Ampersand2Eq,
	/// `<`; the numeric less-than comparison [binary operator](Syntax::ExprBin).
	#[token("<")]
	AngleL,
	/// `<=`; the numeric less-than-or-equals comparison [binary operator](Syntax::ExprBin).
	#[token("<=")]
	AngleLEq,
	/// `>`; the numeric greater-than comparison [binary operator](Syntax::ExprBin).
	#[token(">")]
	AngleR,
	/// `>=`; the numeric greater-than-or-equals comparison [binary operator](Syntax::ExprBin).
	#[token(">=")]
	AngleREq,
	/// `<<`; the bit-wise leftwards shift [binary operator](Syntax::ExprBin).
	#[token("<<")]
	AngleL2,
	/// `<<=`; the bit-wise leftwards shift compound assignment [binary operator](Syntax::ExprBin).
	#[token("<<=")]
	AngleL2Eq,
	/// `>>`; the bit-wise rightwards shift [binary operator](Syntax::ExprBin).
	#[token(">>")]
	AngleR2,
	/// `>>=`; the bit-wise rightwards shift compound assignment [binary operator](Syntax::ExprBin).
	#[token(">>=")]
	AngleR2Eq,
	/// `*`; the multiplication [binary operator](Syntax::ExprBin).
	#[token("*")]
	Asterisk,
	/// `**`; the exponentiation [binary operator](Syntax::ExprBin).
	#[token("**")]
	Asterisk2,
	/// `**=`; the exponentation compound assignment [binary operator](Syntax::ExprBin).
	#[token("**=")]
	Asterisk2Eq,
	/// `*=`; the multiplication compound assignment [binary operator](Syntax::ExprBin).
	#[token("*=")]
	AsteriskEq,
	/// `@`, used for user-defined [binary operators](Syntax::ExprBin).
	#[token("@")]
	At,
	/// `!`; the logical negation [prefix operator](Syntax::ExprPrefix).
	#[token("!")]
	Bang,
	/// `!=`; the logical inequality comparison [binary operator](Syntax::ExprBin).
	#[token("!=")]
	BangEq,
	/// `{`; used to open blocks.
	#[token("{")]
	BraceL,
	/// `}`; used to close blocks.
	#[token("}")]
	BraceR,
	/// `[`; part of [annotations], [array expressions],
	/// [index expressions], and [array type prefixes].
	///
	/// [annotations]: Syntax::Annotation
	/// [array expressions]: Syntax::ExprArray
	/// [index expressions]: Syntax::ExprIndex
	/// [array type prefixes]: Syntax::ArrayPrefix
	#[token("[")]
	BracketL,
	/// `]`; part of [annotations], [array expressions],
	/// [index expressions], and [array type prefixes].
	///
	/// [annotations]: Syntax::Annotation
	/// [array expressions]: Syntax::ExprArray
	/// [index expressions]: Syntax::ExprIndex
	/// [array type prefixes]: Syntax::ArrayPrefix
	#[token("]")]
	BracketR,
	/// `^`; the bit-wise XOR [binary operator](Syntax::ExprBin).
	#[token("^")]
	Caret,
	/// `^=`; the bit-wise XOR compound assignment [binary operator](Syntax::ExprBin).
	#[token("^=")]
	CaretEq,
	/// `:`; used in [type specifiers](Syntax::TypeSpec).
	#[token(":")]
	Colon,
	/// `::`; used in [block labels](Syntax::BlockLabel).
	#[token("::")]
	Colon2,
	/// `,`
	#[token(",")]
	Comma,
	/// `.`; part of [field expressions](Syntax::ExprField).
	#[token(".")]
	Dot,
	/// `.{`; part of [aggregate expressions](Syntax::ExprAggregate).
	#[token(".{")]
	DotBraceL,
	/// `..`; the [range expression](Syntax::ExprRange) operator.
	#[token("..")]
	Dot2,
	/// `..=`; the inclusive-end [range expression](Syntax::ExprRange) operator.
	#[token("..=")]
	Dot2Eq,
	/// `...`; a.k.a. "ellipsis".
	///
	/// Used in:
	/// - [Parameter lists](Syntax::ParamList) of certain compiler intrinsic functions.
	/// - [Argument lists](Syntax::ArgList) to indicate that parameter defaults be used.
	#[token("...")]
	Dot3,
	/// `=`; part of [binding statements](Syntax::StmtBind) and [symbolic constants](Syntax::SymConst).
	#[token("=")]
	Eq,
	/// `==`; the logical equality comparison [binary operator](Syntax::ExprBin).
	#[token("==")]
	Eq2,
	/// `-`; the subtraction [binary operator](Syntax::ExprBin) as well as the
	/// numeric negation [prefix operator](Syntax::ExprPrefix).
	/// Can also be used in [number literal patterns](Syntax::PatLit).
	#[token("-")]
	Minus,
	/// `-=`; the subtraction compound assignment [binary operator](Syntax::ExprBin).
	#[token("-=")]
	MinusEq,
	/// `(`; part of [group expressions](Syntax::ExprGroup).
	#[token("(")]
	ParenL,
	/// `)`; part of [group expressions](Syntax::ExprGroup).
	#[token(")")]
	ParenR,
	/// `%`; the remainder [binary operator](Syntax::ExprBin).
	#[token("%")]
	Percent,
	/// `%=`; the remainder compound assigment [binary operator](Syntax::ExprBin).
	#[token("%=")]
	PercentEq,
	/// `|`; the bit-wise OR [binary operator](Syntax::ExprBin).
	#[token("|")]
	Pipe,
	/// `|=`; the bit-wise OR compound assignment [binary operator](Syntax::ExprBin).
	#[token("|=")]
	PipeEq,
	/// `||`; the logical OR [binary operator](Syntax::ExprBin).
	#[token("||")]
	Pipe2,
	/// `||=`; the logical OR compound assignment [binary operator](Syntax::ExprBin).
	#[token("||=")]
	Pipe2Eq,
	/// `+`; the addition [binary operator](Syntax::ExprBin).
	#[token("+")]
	Plus,
	/// `++`; the string concatenation [binary operator](Syntax::ExprBin).
	#[token("++")]
	Plus2,
	/// `++=`; the string concatenation compound assignment [binary operator](Syntax::ExprBin).
	#[token("++=")]
	Plus2Eq,
	/// `+=`; the addition compound assignment [binary operator](Syntax::ExprBin).
	#[token("+=")]
	PlusEq,
	/// `#`; used to start [annotations](Syntax::Annotation).
	#[token("#")]
	Pound,
	/// `;`; used as a terminator, like in C.
	#[token(";")]
	Semicolon,
	/// `/`; the division [binary operator](Syntax::ExprBin).
	#[token("/")]
	Slash,
	/// `/=`; the division compound assignment [binary operator](Syntax::ExprBin).
	#[token("/=")]
	SlashEq,
	/// `=>`; currently unused.
	#[token("=>")]
	ThickArrow,
	/// `~`; the bitwise negation [prefix operator](Syntax::ExprPrefix).
	#[token("~")]
	Tilde,
	/// `~==`; the ASCII case-insensitive string comparison [binary operator](Syntax::ExprBin).
	#[token("~==")]
	TildeEq2,
	/// `_`; used in [wildcard patterns](Syntax::PatWildcard).
	#[token("_")]
	Underscore,

	#[doc(hidden)]
	__LastGlyph,

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
	#[regex("[0-9][0-9_]*((?:u|i)(?:8|16|32|64|128))?")]
	#[regex("0b[01_]*[01][01_]*((?:u|i)(?:8|16|32|64|128))?")]
	#[regex("0o[0-7_]*[0-7][0-7_]*((?:u|i)(?:8|16|32|64|128))?")]
	#[regex("0x[0-9a-fA-F_]*[0-9a-fA-F][0-9a-fA-F_]*((?:u|i)(?:8|16|32|64|128))?")]
	LitInt,
	#[regex("'[^'\n]*'")]
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
	/// `|_|`
	#[token("|_|")]
	LitVoid,

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

impl Syntax {
	#[must_use]
	pub fn is_glyph(self) -> bool {
		let u = self as u16;
		u > (Self::__FirstGlyph as u16) && u < (Self::__LastGlyph as u16)
	}

	#[must_use]
	pub fn is_keyword(self) -> bool {
		let u = self as u16;
		u > (Self::__FirstKeyword as u16) && u < (Self::__LastKeyword as u16)
	}

	#[must_use]
	pub fn is_trivia(self) -> bool {
		matches!(self, Self::Whitespace | Self::Comment)
	}
}

/// A placeholder type to prevent API breaks in the future if the lexer needs to,
/// for instance, tokenize keywords version-sensitively.
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct LexContext;

impl From<Syntax> for rowan::SyntaxKind {
	fn from(value: Syntax) -> Self {
		Self(value as u16)
	}
}

impl rowan::Language for Syntax {
	type Kind = Self;

	fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
		assert!(raw.0 < Self::__Last as u16);
		unsafe { std::mem::transmute::<u16, Syntax>(raw.0) }
	}

	fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
		kind.into()
	}
}

impl doomfront::LangExt for Syntax {
	type Token = Self;
	const EOF: Self::Token = Self::Eof;
	const ERR_NODE: Self::Kind = Self::Error;
}

impl std::fmt::Display for Syntax {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Error => write!(f, "<error>"),
			Self::FileRoot => write!(f, "file"),
			Self::AggregateInit => write!(f, "aggregate initializer"),
			Self::Annotation => write!(f, "annotation"),
			Self::ArgList => write!(f, "argument list"),
			Self::Argument => write!(f, "argument"),
			Self::ArrayPrefix => write!(f, "array prefix"),
			Self::BlockLabel => write!(f, "block label"),
			Self::FieldDecl => write!(f, "field declaration"),
			Self::FunctionDecl => write!(f, "function declaration"),
			Self::FunctionBody => write!(f, "function body"),
			Self::Parameter => write!(f, "parameter"),
			Self::ParamList => write!(f, "parameter list"),
			Self::SymConst => write!(f, "symbolic constant"),
			Self::TypeSpec => write!(f, "type specifier"),
			Self::StmtBind => write!(f, "binding statement"),
			Self::StmtBreak => write!(f, "`break` statement"),
			Self::StmtContinue => write!(f, "`continue` statement"),
			Self::StmtExpr => write!(f, "expression statement"),
			Self::StmtReturn => write!(f, "`return` statement"),
			Self::PatGrouped => write!(f, "grouped pattern"),
			Self::PatIdent => write!(f, "identifier pattern"),
			Self::PatLit => write!(f, "literal pattern"),
			Self::PatSlice => write!(f, "slice pattern"),
			Self::PatWildcard => write!(f, "wildcard pattern"),
			Self::ExprAggregate => write!(f, "aggregate expression"),
			Self::ExprBin => write!(f, "binary expression"),
			Self::ExprBlock => write!(f, "block expression"),
			Self::ExprCall => write!(f, "call expression"),
			Self::ExprConstruct => write!(f, "construction expression"),
			Self::ExprField => write!(f, "field expression"),
			Self::ExprIdent => write!(f, "identifier expression"),
			Self::ExprIndex => write!(f, "index expression"),
			Self::ExprLit => write!(f, "literal expression"),
			Self::ExprGroup => write!(f, "group expression"),
			Self::ExprPostfix => write!(f, "postfix expression"),
			Self::ExprPrefix => write!(f, "prefix expression"),
			Self::ExprRange => write!(f, "range expression"),
			Self::ExprStruct => write!(f, "structure expression"),
			Self::ExprType => write!(f, "type expression"),
			Self::KwAnyT => write!(f, "`any_t`"),
			Self::KwBreak => write!(f, "`break`"),
			Self::KwConst => write!(f, "`const`"),
			Self::KwContinue => write!(f, "`continue`"),
			Self::KwFunction => write!(f, "`function`"),
			Self::KwLet => write!(f, "`let`"),
			Self::KwReturn => write!(f, "`return`"),
			Self::KwStruct => write!(f, "`struct`"),
			Self::KwTypeT => write!(f, "`type_t`"),
			Self::KwVar => write!(f, "`var`"),
			Self::Ampersand => write!(f, "`&`"),
			Self::Ampersand2 => write!(f, "`&&`"),
			Self::AmpersandEq => write!(f, "`&=`"),
			Self::Ampersand2Eq => write!(f, "`&&=`"),
			Self::AngleL => write!(f, "`<`"),
			Self::AngleLEq => write!(f, "`<=`"),
			Self::AngleR => write!(f, "`>`"),
			Self::AngleREq => write!(f, "`>=`"),
			Self::AngleL2 => write!(f, "`<<`"),
			Self::AngleL2Eq => write!(f, "`<<=`"),
			Self::AngleR2 => write!(f, "`>>`"),
			Self::AngleR2Eq => write!(f, "`>>=`"),
			Self::Asterisk => write!(f, "`*`"),
			Self::Asterisk2 => write!(f, "`**`"),
			Self::Asterisk2Eq => write!(f, "`**=`"),
			Self::AsteriskEq => write!(f, "`*=`"),
			Self::At => write!(f, "`@`"),
			Self::Bang => write!(f, "`!`"),
			Self::BangEq => write!(f, "`!=`"),
			Self::BraceL => write!(f, "`{{`"),
			Self::BraceR => write!(f, "`}}`"),
			Self::BracketL => write!(f, "`[`"),
			Self::BracketR => write!(f, "`]`"),
			Self::Caret => write!(f, "`^`"),
			Self::CaretEq => write!(f, "`^=`"),
			Self::Colon => write!(f, "`:`"),
			Self::Colon2 => write!(f, "`::`"),
			Self::Comma => write!(f, "`,`"),
			Self::Dot => write!(f, "`.`"),
			Self::DotBraceL => write!(f, "`.{{`"),
			Self::Dot2 => write!(f, "`..`"),
			Self::Dot2Eq => write!(f, "`..=`"),
			Self::Dot3 => write!(f, "`...`"),
			Self::Eq => write!(f, "`=`"),
			Self::Eq2 => write!(f, "`==`"),
			Self::Minus => write!(f, "`-`"),
			Self::MinusEq => write!(f, "`-=`"),
			Self::ParenL => write!(f, "`(`"),
			Self::ParenR => write!(f, "`)`"),
			Self::Percent => write!(f, "`%`"),
			Self::PercentEq => write!(f, "`%=`"),
			Self::Pipe => write!(f, "`|`"),
			Self::PipeEq => write!(f, "`|=`"),
			Self::Pipe2 => write!(f, "`||`"),
			Self::Pipe2Eq => write!(f, "`||=`"),
			Self::Plus => write!(f, "`+`"),
			Self::Plus2 => write!(f, "`++`"),
			Self::Plus2Eq => write!(f, "`++=`"),
			Self::PlusEq => write!(f, "`+=`"),
			Self::Pound => write!(f, "`#`"),
			Self::Semicolon => write!(f, "`;`"),
			Self::Slash => write!(f, "`/`"),
			Self::SlashEq => write!(f, "`/=`"),
			Self::ThickArrow => write!(f, "`=>`"),
			Self::Tilde => write!(f, "`~`"),
			Self::TildeEq2 => write!(f, "`~==`"),
			Self::Underscore => write!(f, "`_`"),
			Self::LitFalse => write!(f, "`false`"),
			Self::LitFloat => write!(f, "floating-point literal"),
			Self::LitInt => write!(f, "integer literal"),
			Self::LitName => write!(f, "name literal"),
			Self::LitString => write!(f, "string literal"),
			Self::LitTrue => write!(f, "`true`"),
			Self::LitVoid => write!(f, "`|_|`"),
			Self::Ident => write!(f, "identifier"),
			Self::DocComment => write!(f, "doc comment"),
			Self::Comment => write!(f, "comment"),
			Self::Whitespace => write!(f, "whitespace"),
			Self::Unknown => write!(f, "unknown token"),
			Self::Eof => write!(f, "end-of-input"),
			Self::__FirstKeyword
			| Self::__LastKeyword
			| Self::__FirstGlyph
			| Self::__LastGlyph
			| Self::__Last => unreachable!(),
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn comments() {
		const SAMPLES: &[&str] = &["//", "// ", "////"];

		for sample in SAMPLES {
			let mut lexer = Syntax::lexer(sample);
			let t0 = lexer.next().unwrap().unwrap();
			assert_eq!(t0, Syntax::Comment);
		}
	}

	#[test]
	fn doc_comments() {
		const SAMPLES: &[&str] = &["///", "/// ", "/// lorem ipsum"];

		for sample in SAMPLES {
			let mut lexer = Syntax::lexer(sample);
			let t0 = lexer.next().unwrap().unwrap();
			assert_eq!(t0, Syntax::DocComment);
		}
	}
}
