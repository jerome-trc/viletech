//! A [syntax tag type](Syn) with a macro-generated lexer for its tokens.

use doomfront::rowan;
use logos::Logos;

/// A stronger type over [`rowan::SyntaxKind`] representing all kinds of syntax elements.
#[repr(u16)]
#[derive(Logos, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[logos(error = Syn)]
#[allow(clippy::manual_non_exhaustive)]
pub enum Syn {
	/// A sequence of tokens that did not form a valid syntax element.
	Error,
	/// The top-level node representing a whole file.
	FileRoot,

	// Nodes: high-level constructs ////////////////////////////////////////////
	/// `'(' argument (',' argument)* ','? ')'`
	///
	/// Common to [call expressions](Syn::ExprCall) and [annotations](Syn::Annotation).
	ArgList,
	/// `((ident | namelit) ':')? expr`
	Argument,
	/// `'[' nontypeexpr ']'`
	///
	/// Can start a [`Syn::ExprType`].
	ArrayPrefix,
	/// `'::' ident '::'`
	BlockLabel,
	/// `memberquals 'function' ident paramlist returntype? (block | ';')`
	FunctionDecl,
	/// `'{' T* '}'` where `T` is a statement, [`Syn::Annotation`], or item.
	FunctionBody,
	/// `'import' stringlit ':' (importlist | importall) ';'`
	Import,
	/// `'*' '=>' ident`
	ImportAll,
	/// `(ident | namelit) ('=>' ident)?`
	ImportEntry,
	/// `importentry (',' importentry)* ','?`
	ImportList,
	/// `'const'? ident typespec ('=' expr)?`
	Parameter,
	/// `'(' param? (',' param)* ','? ')'`
	ParamList,
	/// `'const' ident typespec '=' expr ';'`
	///
	/// A "symbolic constant". Semantically equivalent to an immutable compile-time
	/// [binding statement](Syn::StmtBind) but permitted at container scope.
	SymConst,
	/// `':' typeexpr`
	TypeSpec,

	// Nodes: expressions //////////////////////////////////////////////////////
	/// `expr operator expr`
	ExprBin,
	/// `primaryexpr arglist`
	ExprCall,
	/// `primaryexpr '.' (ident | namelit)`
	ExprField,
	/// Parent to only a single [`Syn::Ident`] token.
	ExprIdent,
	/// `primaryexpr '[' expr ']'`
	ExprIndex,
	/// If this is not a string literal, it is parent to only one token tagged as
	/// one of the following:
	/// - [`Syn::LitFalse`]
	/// - [`Syn::LitFloat`]
	/// - [`Syn::LitInt`]
	/// - [`Syn::LitName`]
	/// - [`Syn::LitTrue`]
	/// - [`Syn::LitVoid`]
	/// If this is a string literal, it is parent to one token tagged as
	/// [`Syn::LitString`] which may be followed by a [`Syn::Ident`] suffix,
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
	/// One of the following:
	/// - [`Syn::KwTypedef`]
	/// - [`Syn::KwAny`]
	/// - One or more type expr. prefixes followed by any other kind of expression.
	ExprType,

	// Tokens: keywords ////////////////////////////////////////////////////////
	#[doc(hidden)]
	__FirstKeyword,

	/// `any`
	#[token("any")]
	KwAny,
	/// `const`
	#[token("const")]
	KwConst,
	/// `function`
	#[token("function")]
	KwFunction,
	/// `import`
	#[token("import")]
	KwImport,
	/// `typedef`
	#[token("typedef")]
	KwTypedef,

	#[doc(hidden)]
	__LastKeyword,

	// Tokens: glyphs //////////////////////////////////////////////////////////
	#[doc(hidden)]
	__FirstGlyph,

	/// `&`; the bit-wise AND [binary operator](Syn::ExprBin).
	#[token("&")]
	Ampersand,
	/// `&&`; the logical AND comparison [binary operator](Syn::ExprBin).
	#[token("&&")]
	Ampersand2,
	/// `&=`; the bit-wise AND compound assignment [binary operator](Syn::ExprBin).
	#[token("&=")]
	AmpersandEq,
	/// `&&=`; the logical AND compound assignment [binary operator](Syn::ExprBin).
	#[token("&&=")]
	Ampersand2Eq,
	/// `<`; the numeric less-than comparison [binary operator](Syn::ExprBin).
	#[token("<")]
	AngleL,
	/// `<=`; the numeric less-than-or-equals comparison [binary operator](Syn::ExprBin).
	#[token("<=")]
	AngleLEq,
	/// `>`; the numeric greater-than comparison [binary operator](Syn::ExprBin).
	#[token(">")]
	AngleR,
	/// `>=`; the numeric greater-than-or-equals comparison [binary operator](Syn::ExprBin).
	#[token(">=")]
	AngleREq,
	/// `<<`; the bit-wise leftwards shift [binary operator](Syn::ExprBin).
	#[token("<<")]
	AngleL2,
	/// `<<=`; the bit-wise leftwards shift compound assignment [binary operator](Syn::ExprBin).
	#[token("<<=")]
	AngleL2Eq,
	/// `>>`; the bit-wise rightwards shift [binary operator](Syn::ExprBin).
	#[token(">>")]
	AngleR2,
	/// `>>=`; the bit-wise rightwards shift compound assignment [binary operator](Syn::ExprBin).
	#[token(">>=")]
	AngleR2Eq,
	/// `*`; the multiplication [binary operator](Syn::ExprBin).
	#[token("*")]
	Asterisk,
	/// `**`; the exponentiation [binary operator](Syn::ExprBin).
	#[token("**")]
	Asterisk2,
	/// `**=`; the exponentation compound assignment [binary operator](Syn::ExprBin).
	#[token("**=")]
	Asterisk2Eq,
	/// `*=`; the multiplication compound assignment [binary operator](Syn::ExprBin).
	#[token("*=")]
	AsteriskEq,
	/// `@`, used for user-defined [binary operators](Syn::ExprBin).
	#[token("@")]
	At,
	/// `!`; the logical negation [prefix operator](Syn::ExprPrefix).
	#[token("!")]
	Bang,
	/// `!=`; the logical inequality comparison [binary operator](Syn::ExprBin).
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
	/// [annotations]: Syn::Annotation
	/// [array expressions]: Syn::ExprArray
	/// [index expressions]: Syn::ExprIndex
	/// [array type prefixes]: Syn::ArrayPrefix
	#[token("[")]
	BracketL,
	/// `]`; part of [annotations], [array expressions],
	/// [index expressions], and [array type prefixes].
	///
	/// [annotations]: Syn::Annotation
	/// [array expressions]: Syn::ExprArray
	/// [index expressions]: Syn::ExprIndex
	/// [array type prefixes]: Syn::ArrayPrefix
	#[token("]")]
	BracketR,
	/// `^`; the bit-wise XOR [binary operator](Syn::ExprBin).
	#[token("^")]
	Caret,
	/// `^=`; the bit-wise XOR compound assignment [binary operator](Syn::ExprBin).
	#[token("^=")]
	CaretEq,
	/// `:`; used in [type specifiers](Syn::TypeSpec).
	#[token(":")]
	Colon,
	/// `::`; used in [block labels](Syn::BlockLabel).
	#[token("::")]
	Colon2,
	/// `,`
	#[token(",")]
	Comma,
	/// `.`; part of [field expressions](Syn::ExprField).
	#[token(".")]
	Dot,
	/// `..`; the [range expression](Syn::ExprRange) operator.
	#[token("..")]
	Dot2,
	/// `..` the inclusive-end [range expression](Syn::ExprRange) operator.
	#[token("..=")]
	Dot2Eq,
	/// `=`; part of [assignment statements](Syn::StmtAssign)
	/// and [symbolic constants](Syn::SymConst).
	#[token("=")]
	Eq,
	/// `==`; the logical equality comparison [binary operator](Syn::ExprBin).
	#[token("==")]
	Eq2,
	/// `-`; the subtraction [binary operator](Syn::ExprBin) as well as the
	/// numeric negation [prefix operator](Syn::ExprPrefix).
	#[token("-")]
	Minus,
	/// `-=`; the subtraction compound assignment [binary operator](Syn::ExprBin).
	#[token("-=")]
	MinusEq,
	/// `(`; part of [group expressions](Syn::ExprGroup).
	#[token("(")]
	ParenL,
	/// `)`; part of [group expressions](Syn::ExprGroup).
	#[token(")")]
	ParenR,
	/// `%`; the remainder [binary operator](Syn::ExprBin).
	#[token("%")]
	Percent,
	/// `%=`; the remainder compound assigment [binary operator](Syn::ExprBin).
	#[token("%=")]
	PercentEq,
	/// `|`; the bit-wise OR [binary operator](Syn::ExprBin).
	#[token("|")]
	Pipe,
	/// `|=`; the bit-wise OR compound assignment [binary operator](Syn::ExprBin).
	#[token("|=")]
	PipeEq,
	/// `||`; the logical OR [binary operator](Syn::ExprBin).
	#[token("||")]
	Pipe2,
	/// `||=`; the logical OR compound assignment [binary operator](Syn::ExprBin).
	#[token("||=")]
	Pipe2Eq,
	/// `+`; the addition [binary operator](Syn::ExprBin).
	#[token("+")]
	Plus,
	/// `++`; the string concatenation [binary operator](Syn::ExprBin).
	#[token("++")]
	Plus2,
	/// `++=`; the string concatenation compound assignment [binary operator](Syn::ExprBin).
	#[token("++=")]
	Plus2Eq,
	/// `+=`; the addition compound assignment [binary operator](Syn::ExprBin).
	#[token("+=")]
	PlusEq,
	/// `;`; used as a terminator, like in C.
	#[token(";")]
	Semicolon,
	/// `/`; the division [binary operator](Syn::ExprBin).
	#[token("/")]
	Slash,
	/// `/=`; the division compound assignment [binary operator](Syn::ExprBin).
	#[token("/=")]
	SlashEq,
	/// `=>`; used in [import syntax](Syn::Import) to rename symbols.
	#[token("=>")]
	ThickArrow,
	/// `~`; the bitwise negation [prefix operator](Syn::ExprPrefix).
	#[token("~")]
	Tilde,
	/// `~==`; the ASCII case-insensitive string comparison [binary operator](Syn::ExprBin).
	#[token("~==")]
	TildeEq2,

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

impl Syn {
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
