//! Tags representing syntax nodes, from low-level primitives to high-level composites.

use crate::{zdoom::Token, LangExt};

/// Tags representing syntax nodes, from low-level primitives to high-level composites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Syn {
	// Nodes: high-level composites ////////////////////////////////////////////
	/// Optional part at the end of a [`Syn::StateDef`].
	ActionFunction,
	/// `'action' statesusage?`
	ActionQual,
	/// `'(' exprs? ')'`
	ArgList,
	/// `(ident ':')? expr`
	Argument,
	/// `('[' expr? ']')+`
	ArrayLen,
	/// `'class' ident inheritspec? replacesclause? '{' innard* '}'`
	ClassDef,
	/// `'extend' 'class' ident '{' innard* '}'`
	ClassExtend,
	ClassQuals,
	/// `'static'? 'const' ident '=' expr ';'`
	ConstDef,
	/// `'default' '{' (propertysetting | flagsetting)* '}'`
	DefaultBlock,
	/// `'deprecated' '(' string (',' string)? ')'`
	DeprecationQual,
	/// `'enum' ident enumtypespec? '{' variant* '}'`
	EnumDef,
	/// `':' inttypename`
	EnumTypeSpec,
	/// `ident ('=' expr)?`
	EnumVariant,
	/// A sequence of tokens that did not form a valid syntax element.
	Error,
	FieldDecl,
	FlagDef,
	/// `('+' | '-') identchain`
	FlagSetting,
	/// The first part of a `for` loop's opening "header".
	ForLoopInit,
	/// The second part of a `for` loop's opening "header".
	ForLoopCond,
	/// The third part of a `for` loop's opening "header".
	ForLoopIter,
	FunctionDecl,
	/// `'goto' 'super::'? identchain ('+' integer)?`
	GotoOffset,
	/// `(ident) | ('.'? ident ('.' ident)*)`
	///
	/// Known in ZScript's Lemon grammar as a "dottable ID".
	IdentChain,
	/// The `#include` preprocessor directive and its string literal argument.
	IncludeDirective,
	/// `':' ident`
	InheritSpec,
	/// Will have one of the following tokens as a child:
	/// - [`Syn::KwFalse`]
	/// - [`Syn::KwFloat`]
	/// - [`Syn::IntLit`]
	/// - [`Syn::NameLit`]
	/// - [`Syn::NullLit`]
	/// - [`Syn::StringLit`]
	/// - [`Syn::KwTrue`]
	Literal,
	LocalVar,
	LocalVarInit,
	/// Precedes a [`Syn::FieldDecl`] or [`Syn::FunctionDecl`].
	MemberQuals,
	/// `'mixin' 'class' ident '{' innard* '}'`
	MixinClassDef,
	/// `typeref ident ('=' expr)?`
	Parameter,
	/// `'(' parameter* ')'`
	ParamList,
	/// `identchain expr* ';'`
	PropertySetting,
	PropertyDef,
	/// `'replaces' ident`
	ReplacesClause,
	/// `typeref (',' typeref)*`
	ReturnTypes,
	/// The top-level node, representing the whole file.
	Root,
	/// `'fail' | 'loop' | 'stop' | 'wait' ';'` or
	/// `'goto' (scope '::')? identchain ('+' integer)?`
	StateFlow,
	/// For child nodes under a [`Syn::StatesBlock`].
	StateDef,
	/// `identchain ':'`
	StateLabel,
	/// `'light' '(' (string | name) (',' (string | name))* ')'`
	StateLight,
	/// `'offset' '(' expr ',' expr ')'`
	StateOffset,
	StateQuals,
	/// `'states' statesusage? '{' innard* '}'`
	StatesBlock,
	/// `'(' ('actor' | 'item' | 'overlay' | 'weapon')+ ')'`
	StatesUsage,
	/// `'struct' ident '{' innard* '}'`
	StructDef,
	/// `'extend' 'struct' ident '{' innard* '}'`
	StructExtend,
	StructQuals,
	/// `'[' expr ']'`
	Subscript,
	/// `coretype arraylen?`
	TypeRef,
	/// ident arraylen*
	VarName,
	/// `'version' string`
	VersionDirective,
	/// `'version' '(' string ')'`
	VersionQual,
	// Nodes: core types ///////////////////////////////////////////////////////
	/// `typeref arraylen?`
	ArrayType,
	/// `'class' ('<' identchain '>')?`
	ClassType,
	/// `'array' '<' typeref arraylen? '>'`
	DynArrayType,
	/// `identchain`
	IdentChainType,
	/// `'let'`
	LetType,
	/// `'map' '<' typeref arraylen? ',' typeref arraylen? '>'`
	MapType,
	/// `'mapiterator' '<' typeref arraylen? ',' typeref arraylen? '>'`
	MapIterType,
	/// `'@' ident`
	NativeType,
	/// `'sbyte' | 'byte' | 'int8' | 'uint8' | 'short' | 'ushort' | 'int16' | 'uint16' |
	/// 'bool' | 'float' | 'double' | 'vector2' | 'vector3' | 'name' | 'sound' |
	/// 'state' | 'color'`
	PrimitiveType,
	/// `'readonly' '<' (ident | nativetype) '>'`
	ReadOnlyType,
	// Nodes: expressions //////////////////////////////////////////////////////
	BinExpr,
	/// `primaryexpr '(' arglist? ')'`
	CallExpr,
	/// `'(' 'class' '<' ident '>' ')' '(' namedexprlist? ')'`
	ClassCastExpr,
	/// `'(' expr ')'`
	GroupExpr,
	/// `ident`
	IdentExpr,
	/// `primaryexpr '[' expr ']'`
	IndexExpr,
	/// `primaryexpr '.' ident`
	MemberExpr,
	/// `primaryexpr operator`
	PostfixExpr,
	/// `operator primaryexpr`
	PrefixExpr,
	/// `'super'`
	SuperExpr,
	/// `expr '?' expr ':' expr`, as in C.
	TernaryExpr,
	/// Two parentheses surrounding two, three, or four comma-separated expressions.
	///
	/// Used to construct vectors and colors.
	VectorExpr,
	// Nodes: statements ///////////////////////////////////////////////////////
	/// `'[' exprlist ']' '=' expr ';'`
	AssignStat,
	/// `'break' ';'`
	BreakStat,
	/// `'case' expr ':'`
	CaseStat,
	/// `'{' statement* '}'`
	CompoundStat,
	/// `'continue' ';'`
	ContinueStat,
	/// `'let' (localstat | '[' identlist ']' '=' expr) ';'`
	DeclAssignStat,
	/// `'default' ':'`
	DefaultStat,
	/// `'do' statement 'until' '(' expr ')'`
	DoUntilStat,
	/// `'do' statement 'while' '(' expr ')'`
	DoWhileStat,
	/// `';'`
	EmptyStat,
	/// An expression followed by a semicolon.
	ExprStat,
	/// C-style, with a three-part (semicolon-delimited, parenthesis-enclosed) opener.
	ForStat,
	/// `'foreach' '(' varname ':' expr ')' statement`
	ForEachStat,
	/// `'if' '(' expr ')' '{' statement '}' ('else' statement)?`
	IfStat,
	/// `typeref (ident (arraylen | '{' exprlist '}' | '=' (expr | '{' exprlist '}')))+`
	LocalStat,
	/// `'mixin' ident ';'`
	MixinStat,
	/// `'return' exprlist ';'`
	ReturnStat,
	/// `'static' 'const' (ident '[' ']' | '[' ']' ident) '=' '{' exprlist '}' ';'`
	StaticConstStat,
	/// `'switch' '(' expr ')' statement`
	SwitchStat,
	/// `'until' '(' expr ')' statement`
	UntilStat,
	/// `'while' '(' expr ')' statement`
	WhileStat,
	// Tokens: literals ////////////////////////////////////////////////////////
	/// See [`crate::zdoom::lex::Token::FloatLit`].
	FloatLit,
	/// See [`crate::zdoom::lex::Token::IntLit`].
	IntLit,
	/// See [`crate::zdoom::lex::Token::NameLit`].
	NameLit,
	/// The exact string `null`.
	NullLit,
	/// See [`crate::zdoom::lex::Token::StringLit`].
	StringLit,
	// Tokens: keywords ////////////////////////////////////////////////////////
	#[doc(hidden)]
	__FirstKw,
	KwAbstract,
	KwAction,
	KwAlignOf,
	KwArray,
	KwBool,
	KwBreak,
	/// Only a keyword in [`Syn::StateDef`] elements.
	KwBright,
	KwByte,
	/// Only a keyword in [`Syn::StateDef`] items.
	KwCanRaise,
	KwCase,
	KwChar,
	KwClass,
	KwClearScope,
	KwColor,
	KwConst,
	KwContinue,
	KwCross,
	/// Context-sensitive. Only a keyword within a [`Syn::ClassDef`].
	KwDefault,
	KwDeprecated,
	KwDo,
	KwDot,
	KwDouble,
	KwElse,
	KwEnum,
	KwExtend,
	/// Only a keyword in [`Syn::StateFlow`] elements.
	KwFail,
	KwFalse,
	/// Only a keyword in [`Syn::StateDef`] items.
	KwFast,
	KwFinal,
	KwFlagDef,
	KwFloat,
	KwForEach,
	KwFor,
	KwGoto,
	KwIf,
	KwInt,
	KwInt16,
	KwInt8,
	KwInternal,
	KwIn,
	KwIs,
	KwLatent,
	KwLet,
	/// Only a keyword in [`Syn::StateLight`] elements.
	KwLight,
	KwLong,
	/// Only a keyword in [`Syn::StateFlow`] elements.
	KwLoop,
	KwMap,
	KwMapIterator,
	KwMeta,
	KwMixin,
	KwName,
	KwNative,
	/// Only a keyword in [`Syn::StateDef`] items.
	KwNoDelay,
	/// Only a keyword in [`Syn::StateOffset`] elements.
	KwNone,
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
	/// Only a keyword in [`Syn::StateDef`] items.
	KwSlow,
	KwSound,
	KwState,
	KwStates,
	KwStatic,
	/// Only a keyword in [`Syn::StateFlow`] elements.
	KwStop,
	KwString,
	KwStruct,
	KwSuper,
	KwSwitch,
	KwReplaces,
	KwTransient,
	KwTrue,
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
	/// Always child to a [`Syn::VersionQual`] node.
	KwVersion,
	KwVirtual,
	KwVirtualScope,
	KwVoid,
	/// Only a keyword in [`Syn::StateFlow`] elements.
	KwWait,
	KwWhile,
	// Tokens: keywords, irrelevant ////////////////////////////////////////////
	KwAuto,
	KwVolatile,
	#[doc(hidden)]
	__LastKw,
	// Tokens: glyphs, composite glyphs ////////////////////////////////////////
	#[doc(hidden)]
	__FirstGlyph,
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
	/// `=`
	Eq,
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
	/// `;`
	Semicolon,
	/// `/`
	Slash,
	/// `/=`
	SlashEq,
	// Tokens: glyphs, irrelevant //////////////////////////////////////////////
	/// `->`
	ThinArrow,
	#[doc(hidden)]
	__LastGlyph,
	// Tokens: miscellaenous ///////////////////////////////////////////////////
	/// The string `#include`, ASCII case insensitive.
	PoundInclude,
	RegionStart,
	RegionEnd,
	/// `####`, `----`,
	/// or a combination of exactly 4 ASCII digits, ASCII letters, and underscores.
	StateSprite,
	StateFrames,
	/// ZScript comments use C++ syntax and are treated like whitespace.
	Comment,
	/// Single-line comments preceded by `///`. Non-standard; used only by
	/// [zscdoc](https://gitlab.com/Gutawer/zscdoc).
	DocComment,
	/// A C-style identifier.
	Ident,
	/// Spaces, newlines, carriage returns, or tabs.
	Whitespace,
	/// Lexer input rolled up under [`Syn::Error`].
	Unknown,
	#[doc(hidden)]
	__Last,
}

impl Syn {
	#[must_use]
	pub fn is_trivia(self) -> bool {
		matches!(
			self,
			Self::Whitespace | Self::Comment | Self::RegionStart | Self::RegionEnd
		)
	}

	#[must_use]
	pub fn is_keyword(self) -> bool {
		let u = self as u16;
		u > (Self::__FirstKw as u16) && u < (Self::__LastKw as u16)
	}

	#[must_use]
	pub fn is_glyph(self) -> bool {
		let u = self as u16;
		u > (Self::__FirstGlyph as u16) && u < (Self::__LastGlyph as u16)
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
			Token::KwFalse => Self::KwFalse,
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
			Token::KwNull => Self::NullLit,
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
			Token::KwTrue => Self::KwTrue,
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
			Token::DocComment => Self::DocComment,
			Token::Comment => Self::Comment,
			Token::Unknown => Self::Unknown,
			Token::__Last | Token::__FirstKw | Token::__LastKw | Token::Eof => {
				unreachable!()
			}
		}
	}
}

impl LangExt for Syn {
	type Token = Token;
	const EOF: Self::Token = Token::Eof;
	const ERR_NODE: Self::Kind = Syn::Error;
}
