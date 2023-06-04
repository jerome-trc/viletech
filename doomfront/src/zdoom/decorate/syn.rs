//! Tags representing syntax nodes, from low-level primitives to high-level composites.

/// Tags representing syntax nodes, from low-level primitives to high-level composites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize, serde::Deserialize))]
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
	IdentExpr,
	Literal,
	UnaryExpr,
	// Nodes: preprocessor /////////////////////////////////////////////////////
	/// [`Syn::PoundInclude`] followed by a [`Syn::StringLit`].
	IncludeDirective,
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
	/// `<=`
	AngleLEq,
	/// `>=`
	AngleREq,
	/// `{`
	BraceL,
	/// `}`
	BraceR,
	/// `[`
	BracketL,
	/// `]`
	BracketR,
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
	/// `-`
	Minus,
	/// `--`
	Minus2,
	/// `(`
	ParenL,
	/// `)`
	ParenR,
	/// `|`
	Pipe,
	/// `+`
	Plus,
	/// `++`
	Plus2,
	/// `;`
	Semicolon,
	/// `/`
	Slash,
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
