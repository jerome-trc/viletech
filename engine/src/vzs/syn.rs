//! Syntax tags.

use doomfront::rowan;

use super::ast;

/// Tags representing syntax nodes, from low-level primitives to high-level composites.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Syn {
	// Higher-level composites /////////////////////////////////////////////////
	/// `#![resolver(args)]`. `!` and arguments are optional.
	Annotation,
	/// For both function calls and annotations. Wraps zero or more [`Syn::Argument`]s.
	/// Like in C#, arguments can be passed by parameter name.
	ArgList,
	/// `expr` or `label: expr`. Given to function calls and annotations.
	Argument,
	/// `{` then `}`, optionally with statements in between.
	Block,
	/// A group of declaration qualifier keywords separated by whitespace.
	DeclQualifiers,
	/// `quals returntypes name(params) {}` or `quals returntypes name(params);`
	FunctionDecl,
	/// `ident:`. Used to distinguish blocks and for naming passed arguments.
	/// Distinct from [`Syn::Name`] since it does not introduce a name into scope.
	Label,
	/// Will have one of the following tokens as a child:
	/// - [`Syn::LitChar`]
	/// - [`Syn::LitFalse`]
	/// - [`Syn::LitFloat`]
	/// - [`Syn::LitInt`]
	/// - [`Syn::LitNull`]
	/// - [`Syn::LitString`]
	/// - [`Syn::LitTrue`]
	Literal,
	/// Syntax node with a [`Syn::Ident`] token as a child.
	/// Used as part of function declarations, variable bindings, et cetera.
	Name,
	/// Part of a function definition.
	ParamList,
	/// `name`, `name::name`, `name::name::name`, and so on.
	Resolver,
	/// A [`Syn::Name`], `Super`, or `Self`.
	ResolverPart,
	/// Part of a function declaration, after qualifiers.
	/// One or more written types separated by commas.
	ReturnTypes,
	/// A type reference may be a [`Syn::Resolver`], an array descriptor, a
	/// tuple descriptor, or `_` to make the compiler attempt inferrence.
	TypeRef,
	/// "Type specifier". A [`Syn::Colon`] followed by a [`Syn::TypeRef`].
	TypeSpec,

	// Literals ////////////////////////////////////////////////////////////////
	/// The exact string `null`.
	LitNull,
	/// The exact string `false`.
	LitFalse,
	/// The exact string `true`.
	LitTrue,
	/// VZS integer literals use similar syntax to that of Rust:
	/// <https://doc.rust-lang.org/stable/reference/tokens.html#integer-literals>
	LitInt,
	/// VZS floating-point literals use similar syntax to that of Rust:
	/// <https://doc.rust-lang.org/stable/reference/tokens.html#floating-point-literals>
	LitFloat,
	/// VZS character literals use similar syntax to that of Rust:
	/// <https://doc.rust-lang.org/stable/reference/tokens.html#character-literals>
	LitChar,
	/// VZS string literals use similar syntax to that of Rust:
	/// <https://doc.rust-lang.org/stable/reference/tokens.html#string-literals>
	LitString,

	// Glyphs, composite glyphs, glyph-adjacent ////////////////////////////////
	/// `&`
	Ampersand,
	/// `&&`
	Ampersand2,
	/// `&&=`
	Ampersand2Eq,
	/// `&=`
	AmpersandEq,
	/// `<`
	AngleL,
	/// `<<`
	AngleL2,
	/// `<<=`
	AngleL2Eq,
	/// `<=`
	AngleLEq,
	/// `>`
	AngleR,
	/// `>>`
	AngleR2,
	/// `>>=`
	AngleR2Eq,
	/// `>>>`
	AngleR3,
	/// `>>>=`
	AngleR3Eq,
	/// `>=`
	AngleREq,
	/// `*`
	Asterisk,
	/// `**`
	Asterisk2,
	/// `**=`
	Asterisk2Eq,
	/// `*=`
	AsteriskEq,
	/// `@`
	At,
	/// `!`
	Bang,
	/// `!=`
	BangEq,
	/// `{`
	BraceL,
	/// `}`
	BraceR,
	/// `[`
	BracketL,
	/// `]`
	BracketR,
	/// `^`
	Caret,
	/// `^^`
	Caret2,
	/// `^^`
	Caret2Eq,
	/// `^=`
	CaretEq,
	/// `:`
	Colon,
	/// `::`
	Colon2,
	/// `,`
	Comma,
	/// `$`
	Dollar,
	/// `=`
	Eq,
	/// `==`
	Eq2,
	/// `~` a.k.a tilde.
	Grave,
	/// `~!=`
	GraveBangEq,
	/// `~==`
	GraveEq,
	/// `is` (a specialized operator).
	Is,
	/// `!is` (a specialized operator).
	IsNot,
	/// `-`
	Minus,
	/// `-=`
	MinusEq,
	/// `--`
	Minus2,
	/// `(`
	ParenL,
	/// `)`
	ParenR,
	/// `%`
	Percent,
	/// `.`
	Period,
	/// `..`
	Period2,
	/// `...`, a.k.a. ellipsis.
	Period3,
	/// `..=`
	Period2Eq,
	/// `|`
	Pipe,
	/// `|=`
	PipeEq,
	/// `%=`
	PercentEq,
	/// `||`
	Pipe2,
	/// `||=`
	Pipe2Eq,
	/// `+`
	Plus,
	/// `++`
	Plus2,
	/// `+=`
	PlusEq,
	/// `#`
	Pound,
	/// `?`
	Question,
	/// `;`
	Semicolon,
	/// `/`
	Slash,
	/// `/=`
	SlashEq,
	/// `=>`
	ThickArrow,
	/// `->`
	ThinArrow,

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
	KwReturn,
	KwStatic,
	KwStruct,
	KwSwitch,
	KwUsing,
	KwVirtual,
	KwWhile,

	// Miscellaneous ///////////////////////////////////////////////////////////
	/// C++/Rust form. Treated as though it were whitespace.
	/// This tag covers both single- and multi-line variations, but not docs.
	Comment,
	/// Rust form.
	DocComment,
	/// C-style; an ASCII letter or underscore, then any number of ASCII letters,
	/// ASCII digits, or underscores. Assigned only to tokens.
	/// Can be used in [`Syn::Name`] or [`Syn::Label`] nodes.
	Ident,
	/// Input that the lexer considered to be invalid.
	Unknown,
	/// An unbroken string of spaces, tabs, newlines, and/or carriage returns.
	Whitespace,
	/// The top-level node.
	Root, // Ensure this is always the last variant!
}

impl From<Syn> for rowan::SyntaxKind {
	fn from(value: Syn) -> Self {
		Self(value as u16)
	}
}

impl rowan::Language for Syn {
	type Kind = Self;

	fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
		assert!(raw.0 <= Self::Root as u16);
		unsafe { std::mem::transmute::<u16, Syn>(raw.0) }
	}

	fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
		kind.into()
	}
}

impl doomfront::LangExt for Syn {
	const SYN_WHITESPACE: Self::Kind = Self::Whitespace;
	type AstRoot = ast::Root;
}

impl doomfront::LangComment for Syn {
	const SYN_COMMENT: Self::Kind = Self::Comment;
}

impl Syn {
	/// Alternatively "is whitespace or comment".
	/// Doc comments do not count as trivial syntax.
	#[must_use]
	pub fn is_trivia(&self) -> bool {
		matches!(self, Syn::Comment | Syn::Whitespace)
	}
}
