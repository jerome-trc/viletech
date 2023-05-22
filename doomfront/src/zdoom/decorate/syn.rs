//! Tags representing syntax nodes, from low-level primitives to high-level composites.

/// Tags representing syntax nodes, from low-level primitives to high-level composites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize, serde::Deserialize))]
pub enum Syn {
	// Nodes: high-level composites ////////////////////////////////////////////
	ActorDef,
	ActionFunction,
	ArgList,
	Argument,
	ConstDef,
	EditorNumber,
	EnumDef,
	EnumVariant,
	/// `(+ | -) ident`
	FlagSetting,
	GotoOffset,
	/// `(ident) | (ident ('.' ident)?)`
	IdentChain,
	InheritSpec,
	Literal,
	PropertySettings,
	StatesDef,
	StateDef,
	StateChange,
	StateLabel,
	StateLight,
	StateOffset,
	StatesUsage,
	ReplacesClause,
	RngSpec,
	Root,
	// Nodes: expressions //////////////////////////////////////////////////////
	ExprCall,
	ExprIdent,
	// Nodes: preprocessor /////////////////////////////////////////////////////
	/// [`Syn::PreprocInclude`] followed by a [`Syn::LitString`].
	IncludeDirective,
	// Tokens: preprocessor ////////////////////////////////////////////////////
	/// The exact string `#include`, case-insensitive.
	PreprocInclude,
	// Tokens: literals ////////////////////////////////////////////////////////
	LitFalse,
	LitFloat,
	LitInt,
	/// Like [`Syn::LitString`] but delimited by single-quotes (`'`) instead of double-quotes.
	LitName,
	LitTrue,
	LitString,
	// Tokens: keywords ////////////////////////////////////////////////////////
	KwActor,
	KwBright,
	KwCanRaise,
	KwConst,
	KwEnum,
	KwFail,
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
	KwWait,
	// Tokens: glyphs //////////////////////////////////////////////////////////
	BraceL,
	BraceR,
	BracketL,
	BracketR,
	Colon,
	Colon2,
	Comma,
	Eq,
	Minus,
	ParenL,
	ParenR,
	Period,
	Plus,
	Semicolon,
	// Tokens: miscellaneous ///////////////////////////////////////////////////
	/// `"####"`, `"----"`,
	/// or a combination of exactly 4 ASCII digits, ASCII letters, and underscores.
	StateSprite,
	StateFrames,
	/// The exact string `actor`, ASCII case insensitive.
	StatesUsageActor,
	/// The exact string `item`, ASCII case insensitive.
	StatesUsageItem,
	/// The exact string `overlay`, ASCII case insensitive.
	StatesUsageOverlay,
	/// The exact string `weapon`, ASCII case insensitive.
	StatesUsageWeapon,
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
