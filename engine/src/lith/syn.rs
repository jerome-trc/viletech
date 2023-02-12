//! Syntax tags.

use doomfront::rowan;

use super::ast;

/// Tags representing syntax nodes, from low-level primitives to high-level composites.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Syn {
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
	/// The exact string `null`.
	LitNull,
	/// The exact string `false`.
	LitFalse,
	/// The exact string `true`.
	LitTrue,
	/// Lith integer literals use similar syntax to that of Rust:
	/// <https://doc.rust-lang.org/stable/reference/tokens.html#integer-literals>
	LitInt,
	/// Lith floating-point literals use similar syntax to that of Rust:
	/// <https://doc.rust-lang.org/stable/reference/tokens.html#floating-point-literals>
	LitFloat,
	/// Lith character literals use similar syntax to that of Rust:
	/// <https://doc.rust-lang.org/stable/reference/tokens.html#character-literals>
	LitChar,
	/// Lith string literals use similar syntax to that of Rust:
	/// <https://doc.rust-lang.org/stable/reference/tokens.html#string-literals>
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
	KwReturn,
	KwStatic,
	KwStruct,
	KwSwitch,
	KwUsing,
	KwVirtual,
	KwWhile,

	// Higher-level composites /////////////////////////////////////////////////
	/// For both function calls and annotations. Wraps zero or more [`Syn::Argument`]s.
	/// Like in C# and ZScript, arguments can be passed by parameter name.
	ArgList,
	/// `expr` or `label: expr`. Given to function calls and annotations.
	Argument,
	/// `#![resolver(args)]`. `!` and arguments are optional.
	Annotation,
	/// `{` then `}`, optionally with statements in between.
	Block,
	/// A group of declaration qualifier keywords separated by whitespace.
	/// Part of a [`Syn::FunctionDecl`].
	DeclQualifiers,
	Expression,
	/// e.g. `expr + expr` or `expr?.expr`.
	ExprBinary,
	ExprCall,
	/// `expr[expr]`; array element access.
	ExprIndex,
	/// e.g. `expr++` or `expr?`
	ExprPostfix,
	/// e.g. `++expr`
	ExprPrefix,
	/// `expr = expr ? expr : expr`
	ExprTernary,
	/// `@[typeref]`.
	ExprType,
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
	/// `let ident = <expr>`. May include a `const` after `let` and/or a type
	/// specifier after the identifier, in the form `: typeref`.
	StatBinding,
	/// `break;`
	StatBreak,
	/// `continue;`
	StatContinue,
	/// e.g. `;`
	StatEmpty,
	/// e.g. `666;`.
	StatExpr,
	/// `if {}`
	StatIf,
	/// `do {} until (expr)`
	StatLoopDoUntil,
	/// `do {} while (expr)`
	StatLoopDoWhile,
	/// `for (ident : expr) {}`
	StatLoopFor,
	/// `loop {}`
	StatLoopInfinite,
	/// `while (expr) {}`
	StatLoopWhile,
	/// `return;`
	StatReturn,
	/// Same syntax as C.
	StatSwitch,
	/// An identifier, `Super`, or `Self`.
	ResolverPart,
	/// `name::name` and so on.
	Resolver,
	/// Part of a function declaration, after qualifiers.
	/// One or more written types separated by commas.
	ReturnTypes,
	/// A type reference may be a [`Syn::Resolver`], an array descriptor, a
	/// tuple descriptor, or `_` to make the compiler attempt inferrence.
	TypeRef,
	/// "Type specifier". A [`Syn::Colon`] followed by a [`Syn::TypeRef`].
	TypeSpec,
	/// `using ident = type`
	TypeAlias,

	// Miscellaneous ///////////////////////////////////////////////////////////
	/// C-style; an ASCII letter or underscore, then any number of ASCII letters,
	/// ASCII digits, or underscores. Assigned only to tokens.
	/// Can be used in [`Syn::Ident`] or [`Syn::Label`] nodes.
	Ident,
	/// C++/Rust form. Treated as though it were whitespace.
	/// This tag covers both single- and multi-line variations, but not docs.
	Comment,
	/// Rust form.
	DocComment,
	Whitespace,
	/// Input that the lexer considered to be invalid.
	Unknown,
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
