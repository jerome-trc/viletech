//! Tags representing syntax nodes, from low-level primitives to high-level composites.

use crate::{zdoom::Token, LangExt};

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
	/// [`Syn::KwInclude`] followed by a [`Syn::StringLit`].
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
	GroupExpr,
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
	// Tokens: keywords, relevant //////////////////////////////////////////////
	KwActor,
	KwAction,
	KwBreak,
	KwBright,
	KwCanRaise,
	KwConst,
	KwContinue,
	KwDamageType,
	KwDo,
	KwElse,
	KwEnum,
	KwFail,
	KwFalse,
	KwFast,
	KwFixed,
	KwFloat,
	KwFor,
	KwGoto,
	KwIf,
	KwInt,
	KwLight,
	KwLoop,
	KwNoDelay,
	KwOffset,
	KwReplaces,
	KwReturn,
	KwSlow,
	KwStates,
	KwStop,
	KwSuper,
	KwTrue,
	KwVar,
	KwWait,
	// Tokens: keywords, irrelevant ////////////////////////////////////////////
	KwAbstract,
	KwAlignOf,
	KwArray,
	KwAuto,
	KwBool,
	KwByte,
	KwCase,
	KwChar,
	KwClass,
	KwColor,
	KwCross,
	KwDefault,
	KwDot,
	KwDouble,
	KwIn,
	/// The exact string `#include`, ASCII case-insensitive.
	KwInclude,
	KwInt16,
	KwInt8,
	KwIs,
	KwLong,
	KwMap,
	KwMapIterator,
	KwMixin,
	KwNative,
	KwNone,
	KwNull,
	KwProperty,
	KwSByte,
	KwShort,
	KwSizeOf,
	KwSound,
	KwState,
	KwStatic,
	KwString,
	KwStruct,
	KwSwitch,
	KwUntil,
	KwUInt,
	KwUInt16,
	KwUInt8,
	KwULong,
	KwUShort,
	KwVector2,
	KwVector3,
	KwVersion,
	KwVirtualScope,
	KwVoid,
	KwVolatile,
	KwWhile,
	// Tokens: glyphs, relevant ////////////////////////////////////////////////
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
	// Tokens: glyphs, irrelevant //////////////////////////////////////////////
	/// `<>=`
	AngleLAngleREq,
	/// `**`
	Asterisk2,
	/// `@`
	At,
	/// `\`
	Backslash,
	/// `$`
	Dollar,
	/// `..`
	Dot2,
	/// `...`
	Dot3,
	/// `#`
	Pound,
	/// `####`
	Pound4,
	/// `~==`
	TildeEq2,
	/// `->`
	ThinArrow,
	// Tokens: miscellaneous ///////////////////////////////////////////////////
	RegionStart,
	RegionEnd,
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

impl LangExt for Syn {
	type Token = Token;
	const EOF: Self::Token = Token::Eof;
	const ERR_NODE: Self::Kind = Syn::Error;
}

impl From<crate::zdoom::Token> for Syn {
	fn from(value: crate::zdoom::Token) -> Self {
		match value {
			Token::FloatLit => Self::FloatLit,
			Token::IntLit => Self::IntLit,
			Token::NameLit => Self::NameLit,
			Token::StringLit => Self::StringLit,
			// Keywords ////////////////////////////////////////////////////////
			Token::KwAbstract => Self::KwAbstract,
			Token::KwAction => Self::KwAction,
			Token::KwAlignOf => Self::KwAlignOf,
			Token::KwArray => Self::KwArray,
			Token::KwAuto => Self::KwAuto,
			Token::KwBool => Self::KwBool,
			Token::KwBreak => Self::KwBreak,
			Token::KwBright => Self::KwBright,
			Token::KwByte => Self::KwByte,
			Token::KwCanRaise => Self::KwCanRaise,
			Token::KwCase => Self::KwCase,
			Token::KwChar => Self::KwChar,
			Token::KwClass => Self::KwClass,
			Token::KwColor => Self::KwColor,
			Token::KwConst => Self::KwConst,
			Token::KwContinue => Self::KwContinue,
			Token::KwCross => Self::KwCross,
			Token::KwDefault => Self::KwDefault,
			Token::KwDo => Self::KwDo,
			Token::KwDot => Self::KwDot,
			Token::KwDouble => Self::KwDouble,
			Token::KwElse => Self::KwElse,
			Token::KwEnum => Self::KwEnum,
			Token::KwFail => Self::KwFail,
			Token::KwFalse => Self::KwFalse,
			Token::KwFast => Self::KwFast,
			Token::KwFloat => Self::KwFloat,
			Token::KwFor => Self::KwFor,
			Token::KwGoto => Self::KwGoto,
			Token::KwIn => Self::KwIn,
			Token::KwInclude => Self::KwInclude,
			Token::KwIf => Self::KwIf,
			Token::KwInt => Self::KwInt,
			Token::KwInt16 => Self::KwInt16,
			Token::KwInt8 => Self::KwInt8,
			Token::KwIs => Self::KwIs,
			Token::KwLight => Self::KwLight,
			Token::KwLong => Self::KwLong,
			Token::KwLoop => Self::KwLoop,
			Token::KwMap => Self::KwMap,
			Token::KwMapIterator => Self::KwMapIterator,
			Token::KwMixin => Self::KwMixin,
			Token::KwNative => Self::KwNative,
			Token::KwNoDelay => Self::KwNoDelay,
			Token::KwNone => Self::KwNone,
			Token::KwNull => Self::KwNull,
			Token::KwOffset => Self::KwOffset,
			Token::KwProperty => Self::KwProperty,
			Token::KwReplaces => Self::KwReplaces,
			Token::KwReturn => Self::KwReturn,
			Token::KwSByte => Self::KwSByte,
			Token::KwShort => Self::KwShort,
			Token::KwSizeOf => Self::KwSizeOf,
			Token::KwSlow => Self::KwSlow,
			Token::KwSound => Self::KwSound,
			Token::KwState => Self::KwState,
			Token::KwStates => Self::KwStates,
			Token::KwStatic => Self::KwStatic,
			Token::KwStop => Self::KwStop,
			Token::KwString => Self::KwString,
			Token::KwStruct => Self::KwStruct,
			Token::KwSuper => Self::KwSuper,
			Token::KwSwitch => Self::KwSwitch,
			Token::KwTrue => Self::KwTrue,
			Token::KwUInt => Self::KwUInt,
			Token::KwUInt16 => Self::KwUInt16,
			Token::KwUInt8 => Self::KwUInt8,
			Token::KwULong => Self::KwULong,
			Token::KwUntil => Self::KwUntil,
			Token::KwUShort => Self::KwUShort,
			Token::KwVar => Self::KwVar,
			Token::KwVector2 => Self::KwVector2,
			Token::KwVector3 => Self::KwVector3,
			Token::KwVersion => Self::KwVersion,
			Token::KwVirtualScope => Self::KwVirtualScope,
			Token::KwVoid => Self::KwVoid,
			Token::KwVolatile => Self::KwVolatile,
			Token::KwWait => Self::KwWait,
			Token::KwWhile => Self::KwWhile,
			// Glyphs //////////////////////////////////////////////////////////
			Token::Ampersand => Self::Ampersand,
			Token::Ampersand2 => Self::Ampersand2,
			Token::AmpersandEq => Self::AmpersandEq,
			Token::AngleL => Self::AngleL,
			Token::AngleL2 => Self::AngleL2,
			Token::AngleLEq => Self::AngleLEq,
			Token::AngleL2Eq => Self::AngleL2Eq,
			Token::AngleR => Self::AngleR,
			Token::AngleREq => Self::AngleREq,
			Token::AngleR2 => Self::AngleR2,
			Token::AngleR3 => Self::AngleR3,
			Token::AngleR2Eq => Self::AngleR2Eq,
			Token::AngleR3Eq => Self::AngleR3Eq,
			Token::AngleLAngleREq => Self::AngleLAngleREq,
			Token::Asterisk => Self::Asterisk,
			Token::Asterisk2 => Self::Asterisk2,
			Token::AsteriskEq => Self::AsteriskEq,
			Token::At => Self::At,
			Token::Backslash => Self::Backslash,
			Token::Bang => Self::Bang,
			Token::BangEq => Self::BangEq,
			Token::BraceL => Self::BraceL,
			Token::BraceR => Self::BraceR,
			Token::BracketL => Self::BracketL,
			Token::BracketR => Self::BracketR,
			Token::Caret => Self::Caret,
			Token::CaretEq => Self::CaretEq,
			Token::Colon => Self::Colon,
			Token::Colon2 => Self::Colon2,
			Token::Comma => Self::Comma,
			Token::Dollar => Self::Dollar,
			Token::Dot => Self::Dot,
			Token::Dot2 => Self::Dot2,
			Token::Dot3 => Self::Dot3,
			Token::Eq => Self::Eq,
			Token::Eq2 => Self::Eq2,
			Token::Tilde => Self::Tilde,
			Token::TildeEq2 => Self::TildeEq2,
			Token::Minus => Self::Minus,
			Token::Minus2 => Self::Minus2,
			Token::MinusEq => Self::MinusEq,
			Token::ParenL => Self::ParenL,
			Token::ParenR => Self::ParenR,
			Token::Percent => Self::Percent,
			Token::PercentEq => Self::PercentEq,
			Token::Pipe => Self::Pipe,
			Token::Pipe2 => Self::Pipe2,
			Token::PipeEq => Self::PipeEq,
			Token::Plus => Self::Plus,
			Token::Plus2 => Self::Plus2,
			Token::PlusEq => Self::PlusEq,
			Token::Pound => Self::Pound,
			Token::Pound4 => Self::Pound4,
			Token::Question => Self::Question,
			Token::Semicolon => Self::Semicolon,
			Token::Slash => Self::Slash,
			Token::SlashEq => Self::SlashEq,
			Token::ThinArrow => Self::ThinArrow,
			// Miscellaneous ///////////////////////////////////////////////////
			Token::RegionStart => Self::RegionStart,
			Token::RegionEnd => Self::RegionEnd,
			Token::Ident
			| Token::KwClearScope
			| Token::KwDeprecated
			| Token::KwExtend
			| Token::KwFinal
			| Token::KwFlagDef
			| Token::KwForEach
			| Token::KwInternal
			| Token::KwLet
			| Token::KwMeta
			| Token::KwName
			| Token::KwOut
			| Token::KwOverride
			| Token::KwPlay
			| Token::KwPrivate
			| Token::KwProtected
			| Token::KwReadOnly
			| Token::KwTransient
			| Token::KwUi
			| Token::KwVarArg
			| Token::KwVirtual => Self::Ident,
			Token::Whitespace => Self::Whitespace,
			Token::Comment => Self::Comment,
			Token::Unknown | Token::Eof => Self::Unknown,
			Token::__Last | Token::__FirstKw | Token::__LastKw | Token::DocComment => {
				unreachable!()
			}
		}
	}
}
