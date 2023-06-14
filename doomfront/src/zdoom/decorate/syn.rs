//! Tags representing syntax nodes, from low-level primitives to high-level composites.

/// Tags representing syntax nodes, from low-level primitives to high-level composites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Syn {
	// Nodes: high-level composites ////////////////////////////////////////////
	/// `'actor' actorident? inheritspec? replacesclause? editornum? '{' innard* '}'`
	ActorDef,
	/// Optional part at the end of a [`Syn::StateDef`].
	ActionFunction,
	/// `'(' exprs? ')'`
	ArgList,
	/// `'const' 'int'|'float'|'fixed' ident '=' expr ';'`
	ConstDef,
	/// `'damagetype' '{' damagetypekvkp* '}'`
	DamageTypeDef,
	/// `ident (int | float | string)?`
	DamageTypeKvp,
	/// Wraps a [`Syn::IntLit`].
	EditorNumber,
	/// `'enum' '{' variant* '}' ';'`
	EnumDef,
	/// `ident ('=' expr)?`
	EnumVariant,
	/// A sequence of tokens that did not form a valid syntax element.
	Error,
	/// `(+ | -) identchain`
	FlagSetting,
	/// `'+' integer`
	GotoOffset,
	/// `(ident) | (ident ('.' ident)?)`
	IdentChain,
	/// [`Syn::PoundInclude`] followed by a [`Syn::StringLit`].
	IncludeDirective,
	/// `':' actorident`
	InheritSpec,
	PropertySettings,
	/// `'states' '{' (statedef|stateflow|statelabel)* '}'`
	StatesDef,
	/// For child nods under a [`Syn::StatesDef`].
	StateDef,
	StateFlow,
	/// `ident ':'`
	StateLabel,
	/// `'light' '(' string ')'`
	StateLight,
	/// `'offset' '(' expr ',' expr ')'`
	StateOffset,
	/// `'(' ident+ ')'`, where `ident` is "actor", "item", "overlay", or "weapon"
	/// (matched ASCII case-insensitively).
	StatesUsage,
	/// `'[' expr ']'`
	Subscript,
	/// `'replaces' actorident`
	ReplacesClause,
	/// `'[' ident ']'`, between a call identifier and argument list.
	RngSpec,
	/// The top-level node, representing the whole file.
	Root,
	/// `'var' ('int'|'float') ident ';'`
	UserVar,
	// Nodes: expressions //////////////////////////////////////////////////////
	BinExpr,
	CallExpr,
	GroupedExpr,
	IdentExpr,
	IndexExpr,
	Literal,
	PostfixExpr,
	PrefixExpr,
	TernaryExpr,
	// Tokens: literals ////////////////////////////////////////////////////////
	/// See [`crate::zdoom::lex::Token::FloatLit`].
	FloatLit,
	/// See [`crate::zdoom::lex::Token::IntLit`].
	IntLit,
	/// See [`crate::zdoom::lex::Token::NameLit`].
	NameLit,
	/// See [`crate::zdoom::lex::Token::StringLit`].
	StringLit,
	// Tokens: keywords ////////////////////////////////////////////////////////
	KwActor,
	KwBright,
	KwCanRaise,
	KwConst,
	KwDamageType,
	KwEnum,
	KwFail,
	KwFalse,
	KwFast,
	KwFixed,
	KwFloat,
	KwGoto,
	KwInt,
	KwLight,
	KwLoop,
	KwNoDelay,
	KwOffset,
	KwReplaces,
	KwSlow,
	KwStates,
	KwStop,
	KwSuper,
	KwTrue,
	KwVar,
	KwWait,
	// Tokens: glyphs //////////////////////////////////////////////////////////
	/// `&`
	Ampersand,
	/// `&&`
	Ampersand2,
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
	/// `>>>`
	AngleR3,
	/// `>>=`
	AngleR2Eq,
	/// `>>>=`
	AngleR3Eq,
	/// `>=`
	AngleREq,
	/// `*`
	Asterisk,
	/// `*=`
	AsteriskEq,
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
	/// `^=`
	CaretEq,
	/// `:`
	Colon,
	/// `::`
	Colon2,
	/// `,`
	Comma,
	/// `.`
	Dot,
	/// `=`
	Eq,
	/// `==`
	Eq2,
	/// `~`
	Tilde,
	/// `-`
	Minus,
	/// `--`
	Minus2,
	/// `-=`
	MinusEq,
	/// `(`
	ParenL,
	/// `)`
	ParenR,
	/// `%`
	Percent,
	/// `%=`
	PercentEq,
	/// `|`
	Pipe,
	/// `||`
	Pipe2,
	/// `|=`
	PipeEq,
	/// `+`
	Plus,
	/// `++`
	Plus2,
	/// `+=`
	PlusEq,
	/// `?`
	Question,
	/// `;`
	Semicolon,
	/// `/`
	Slash,
	/// `/=`
	SlashEq,
	// Tokens: miscellaneous ///////////////////////////////////////////////////
	/// The exact string `#include`, ASCII case-insensitive.
	PoundInclude,
	/// `"####"`, `"----"`,
	/// or a combination of exactly 4 ASCII digits, ASCII letters, and underscores.
	StateSprite,
	StateFrames,
	// Tokens: foundational ////////////////////////////////////////////////////
	/// Either single-line or multi-line.
	Comment,
	/// A C-style identifier.
	Ident,
	/// Spaces, newlines, carriage returns, or tabs.
	Whitespace,
	/// Lexer input rolled up under [`Syn::Error`].
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
