//! Tags representing syntax nodes, from low-level primitives to high-level composites.

use crate::{zdoom::Token, LangExt};

/// Tags representing syntax nodes, from low-level primitives to high-level composites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Syn {
	// Nodes: high-level composites ////////////////////////////////////////////
	/// A whole CVar definition.
	Definition,
	/// An `=` followed by a literal to optionally set a custom default value.
	DefaultDef,
	/// A sequence of tokens that either did not form a valid syntax element,
	/// or which contained one or more tokens considered invalid by the lexer.
	Error,
	/// The set of flags qualifying a definition, scope specifiers included.
	DefFlags,
	/// The top-level node, representing the whole file.
	Root,
	/// The type specifier is always followed by the identifier.
	TypeSpec,
	// Tokens: literals ////////////////////////////////////////////////////////
	/// The exact string `false`.
	FalseLit,
	/// See [`crate::zdoom::lex::Token::FloatLit`].
	FloatLit,
	/// See [`crate::zdoom::lex::Token::IntLit`].
	IntLit,
	/// See [`crate::zdoom::lex::Token::StringLit`].
	/// Also used for providing defaults to color CVars.
	StringLit,
	/// The exact string `true`.
	TrueLit,
	// Tokens: literals, irrelevant ////////////////////////////////////////////
	/// See [`crate::zdoom::lex::Token::NameLit`].
	NameLit,
	// Tokens: keywords ////////////////////////////////////////////////////////
	/// The type specifier `bool`.
	KwBool,
	/// The type specifier `color`.
	KwColor,
	/// The type specifier `float`.
	KwFloat,
	/// The type specifier `int`.
	KwInt,
	/// The type specifier `string`.
	KwString,

	/// The configuration flag `cheat`.
	KwCheat,
	/// The configuration flag `noarchive`.
	KwNoArchive,
	/// The scope specifier `nosave`.
	KwNoSave,
	/// The configuration flag `latch`.
	KwLatch,
	/// The scope specifier `server`.
	KwServer,
	/// The scope specifier `user`.
	KwUser,
	// Tokens: keywords, irrelevant ////////////////////////////////////////////
	KwAbstract,
	KwAction,
	KwAlignOf,
	KwArray,
	KwAuto,
	KwBreak,
	KwBright,
	KwByte,
	KwCanRaise,
	KwCase,
	KwChar,
	KwClass,
	KwClearScope,
	KwConst,
	KwContinue,
	KwCross,
	KwDefault,
	KwDeprecated,
	KwDo,
	KwDot,
	KwDouble,
	KwElse,
	KwEnum,
	KwExtend,
	KwFail,
	KwFast,
	KwFinal,
	KwFlagDef,
	KwForEach,
	KwFor,
	KwGoto,
	KwIf,
	KwInt16,
	KwInt8,
	KwInternal,
	KwIn,
	KwIs,
	KwLatent,
	KwLet,
	KwLight,
	KwLong,
	KwLoop,
	KwMap,
	KwMapIterator,
	KwMeta,
	KwMixin,
	KwName,
	KwNative,
	KwNoDelay,
	KwNone,
	KwNull,
	KwOffset,
	KwOut,
	KwOverride,
	KwPlay,
	KwPrivate,
	KwProperty,
	KwProtected,
	KwReadOnly,
	KwReturn,
	KwSByte,
	KwShort,
	KwSizeOf,
	KwSlow,
	KwSound,
	KwState,
	KwStates,
	KwStatic,
	KwStop,
	KwStruct,
	KwSuper,
	KwSwitch,
	KwReplaces,
	KwTransient,
	KwUi,
	KwUInt,
	KwUInt16,
	KwUInt8,
	KwULong,
	KwUntil,
	KwUShort,
	KwVar,
	KwVarArg,
	KwVector2,
	KwVector3,
	KwVersion,
	KwVirtual,
	KwVirtualScope,
	KwVoid,
	KwVolatile,
	KwWait,
	KwWhile,
	// Tokens: glyphs //////////////////////////////////////////////////////////
	/// `=`
	Eq,
	/// `;`
	Semicolon,
	// Tokens: glyphs, irrelevant //////////////////////////////////////////////
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
	/// `<>=`
	AngleLAngleREq,
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
	/// `\`
	Backslash,
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
	/// `.`
	Dot,
	/// `..`
	Dot2,
	/// `...`, a.k.a. ellipsis.
	Dot3,
	/// `==`
	Eq2,
	/// `~`
	Tilde,
	/// `~==`
	TildeEq2,
	/// `-`
	Minus,
	/// `-=`
	MinusEq,
	/// `--`
	Minus2,
	/// `----` (for actor state sprite definitions.)
	Minus4,
	/// `(`
	ParenL,
	/// `)`
	ParenR,
	/// `%`
	Percent,
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
	/// `####` (for state sprites in actor definitions.)
	Pound4,
	/// `?`
	Question,
	/// `/`
	Slash,
	/// `/=`
	SlashEq,
	/// `->`
	ThinArrow,
	// Tokens: miscellaenous ///////////////////////////////////////////////////
	RegionStart,
	RegionEnd,
	/// A name for a defined CVar.
	Ident,
	/// Either single-line (C++-style) or multi-line (C-style).
	Comment,
	/// Input that the lexer considered to be invalid.
	Unknown,
	/// Spaces, newlines, carriage returns, or tabs.
	Whitespace,
	// Tokens, miscellaneous, irrelevant ///////////////////////////////////////
	/// The string `#include`, ASCII case insensitive.
	PoundInclude,
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
			Token::KwClearScope => Self::KwClearScope,
			Token::KwColor => Self::KwColor,
			Token::KwConst => Self::KwConst,
			Token::KwContinue => Self::KwContinue,
			Token::KwCross => Self::KwCross,
			Token::KwDefault => Self::KwDefault,
			Token::KwDeprecated => Self::KwDeprecated,
			Token::KwDo => Self::KwDo,
			Token::KwDot => Self::KwDot,
			Token::KwDouble => Self::KwDouble,
			Token::KwElse => Self::KwElse,
			Token::KwEnum => Self::KwEnum,
			Token::KwExtend => Self::KwExtend,
			Token::KwFail => Self::KwFail,
			Token::KwFalse => Self::FalseLit,
			Token::KwFast => Self::KwFast,
			Token::KwFinal => Self::KwFinal,
			Token::KwFlagDef => Self::KwFlagDef,
			Token::KwFloat => Self::KwFloat,
			Token::KwFor => Self::KwFor,
			Token::KwForEach => Self::KwForEach,
			Token::KwGoto => Self::KwGoto,
			Token::KwIn => Self::KwIn,
			Token::KwInternal => Self::KwInternal,
			Token::KwIf => Self::KwIf,
			Token::KwInt => Self::KwInt,
			Token::KwInt16 => Self::KwInt16,
			Token::KwInt8 => Self::KwInt8,
			Token::KwIs => Self::KwIs,
			Token::KwLet => Self::KwLet,
			Token::KwLight => Self::KwLight,
			Token::KwLong => Self::KwLong,
			Token::KwLoop => Self::KwLoop,
			Token::KwMap => Self::KwMap,
			Token::KwMapIterator => Self::KwMapIterator,
			Token::KwMeta => Self::KwMeta,
			Token::KwMixin => Self::KwMixin,
			Token::KwName => Self::KwName,
			Token::KwNative => Self::KwNative,
			Token::KwNoDelay => Self::KwNoDelay,
			Token::KwNone => Self::KwNone,
			Token::KwNull => Self::KwNull,
			Token::KwOffset => Self::KwOffset,
			Token::KwOut => Self::KwOut,
			Token::KwOverride => Self::KwOverride,
			Token::KwPlay => Self::KwPlay,
			Token::KwPrivate => Self::KwPrivate,
			Token::KwProperty => Self::KwProperty,
			Token::KwProtected => Self::KwProtected,
			Token::KwReadOnly => Self::KwReadOnly,
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
			Token::KwTransient => Self::KwTransient,
			Token::KwTrue => Self::TrueLit,
			Token::KwUi => Self::KwUi,
			Token::KwUInt => Self::KwUInt,
			Token::KwUInt16 => Self::KwUInt16,
			Token::KwUInt8 => Self::KwUInt8,
			Token::KwULong => Self::KwULong,
			Token::KwUntil => Self::KwUntil,
			Token::KwUShort => Self::KwUShort,
			Token::KwVar => Self::KwVar,
			Token::KwVarArg => Self::KwVarArg,
			Token::KwVector2 => Self::KwVector2,
			Token::KwVector3 => Self::KwVector3,
			Token::KwVersion => Self::KwVersion,
			Token::KwVirtual => Self::KwVirtual,
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
			Token::Minus => Self::Minus,
			Token::Minus2 => Self::Minus2,
			Token::Minus4 => Self::Minus4,
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
			Token::Tilde => Self::Tilde,
			Token::TildeEq2 => Self::TildeEq2,
			// Miscellaneous ///////////////////////////////////////////////////
			Token::PoundInclude => Self::PoundInclude,
			Token::RegionStart => Self::RegionStart,
			Token::RegionEnd => Self::RegionEnd,
			Token::Ident => Self::Ident,
			Token::Whitespace => Self::Whitespace,
			Token::Comment => Self::Comment,
			Token::Unknown => Self::Unknown,
			Token::__Last | Token::__FirstKw | Token::__LastKw | Token::Eof | Token::DocComment => {
				unreachable!()
			}
		}
	}
}
