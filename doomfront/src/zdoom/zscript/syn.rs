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
	/// `'class' 'ident' inheritspec? replacesclause? '{' innard* '}'`
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
	/// `':' ident`
	EnumTypeSpec,
	/// `ident ('=' expr)?`
	EnumVariant,
	/// A sequence of tokens that did not form a valid syntax element.
	Error,
	FieldDecl,
	FlagDef,
	/// `('+' | '-') ident`
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
	/// - [`Syn::FalseLit`]
	/// - [`Syn::KwFloat`]
	/// - [`Syn::IntLit`]
	/// - [`Syn::NameLit`]
	/// - [`Syn::NullLit`]
	/// - [`Syn::StringLit`]
	/// - [`Syn::TrueLit`]
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
	/// `'[' ident ']'`, between a call identifier and argument list.
	RngSpec,
	/// The top-level node, representing the whole file.
	Root,
	/// `'fail' | 'loop' | 'stop' | 'wait' ';'` or
	/// `'goto' (scope '::')? identchain ('+' integer)?`
	StateFlow,
	/// For child nodes under a [`Syn::StatesBlock`].
	StateDef,
	/// `ident ':'`
	StateLabel,
	/// `'light' '(' (string | name) (',' (string | name))* ')'`
	StateLight,
	/// `'offset' '(' expr ',' expr ')'`
	StateOffset,
	StateQuals,
	/// `'states' ident '{' innard* '}'`
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
	/// The exact string `false`.
	KwFalse,
	/// See [`crate::zdoom::lex::Token::FloatLit`].
	FloatLit,
	/// See [`crate::zdoom::lex::Token::IntLit`].
	IntLit,
	/// See [`crate::zdoom::lex::Token::NameLit`].
	NameLit,
	/// The exact string `null`.
	NullLit,
	/// The exact string `true`.
	KwTrue,
	/// See [`crate::zdoom::lex::Token::StringLit`].
	StringLit,
	// Tokens: keywords ////////////////////////////////////////////////////////
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
	KwSizeof,
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
	// Tokens: glyphs, composite glyphs ////////////////////////////////////////
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
	/// `;`
	Semicolon,
	/// `/`
	Slash,
	/// `/=`
	SlashEq,
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
		const MAPPING: &[Syn] = &[
			Syn::FloatLit,
			Syn::IntLit,
			Syn::NameLit,
			Syn::StringLit,
			// Keywords ////////////////////////////////////////////////////////
			Syn::Unknown, // __FirstKw
			Syn::KwAbstract,
			Syn::KwAction,
			Syn::KwAlignOf,
			Syn::KwArray,
			Syn::Ident, // KwAuto
			Syn::KwBool,
			Syn::KwBreak,
			Syn::KwBright,
			Syn::KwByte,
			Syn::KwCanRaise,
			Syn::KwCase,
			Syn::Ident,
			Syn::KwClearScope,
			Syn::KwClass,
			Syn::KwColor,
			Syn::KwConst,
			Syn::KwContinue,
			Syn::KwCross,
			Syn::KwDefault,
			Syn::KwDeprecated,
			Syn::KwDo,
			Syn::KwDot,
			Syn::KwDouble,
			Syn::KwElse,
			Syn::KwEnum,
			Syn::KwExtend,
			Syn::KwFail,
			Syn::KwFalse, // KwFalse
			Syn::KwFast,
			Syn::KwFinal,
			Syn::KwFlagDef,
			Syn::KwFloat,
			Syn::KwFor,
			Syn::KwForEach,
			Syn::KwGoto,
			Syn::KwIn,
			Syn::KwIf,
			Syn::KwInt,
			Syn::KwInt16,
			Syn::KwInt8,
			Syn::KwInternal,
			Syn::KwIs,
			Syn::KwLet,
			Syn::KwLight,
			Syn::KwLong,
			Syn::KwLoop,
			Syn::KwMap,
			Syn::KwMapIterator,
			Syn::KwMeta,
			Syn::KwMixin,
			Syn::KwName, // KwName
			Syn::KwNative,
			Syn::KwNoDelay,
			Syn::Ident,   // KwNone
			Syn::NullLit, // KwNull
			Syn::KwOffset,
			Syn::KwOut,
			Syn::KwOverride,
			Syn::KwPlay,
			Syn::KwPrivate,
			Syn::KwProperty,
			Syn::KwProtected,
			Syn::KwReadOnly,
			Syn::KwReplaces,
			Syn::KwReturn,
			Syn::Ident,
			Syn::Ident,
			Syn::KwSizeof,
			Syn::KwSlow,
			Syn::KwSound,
			Syn::KwState,
			Syn::KwStates,
			Syn::KwStatic,
			Syn::KwStop,
			Syn::KwString,
			Syn::KwStruct,
			Syn::KwSuper,
			Syn::KwSwitch,
			Syn::KwTransient,
			Syn::KwTrue, // KwTrue
			Syn::KwUi,
			Syn::KwUInt,
			Syn::KwUInt16,
			Syn::KwUInt8,
			Syn::KwULong,
			Syn::KwUntil,
			Syn::KwUShort,
			Syn::KwVar,
			Syn::KwVarArg,
			Syn::KwVector2,
			Syn::KwVector3,
			Syn::KwVersion,
			Syn::KwVirtual,
			Syn::KwVirtualScope,
			Syn::KwVoid,
			Syn::Ident, // KwVolatile
			Syn::KwWait,
			Syn::KwWhile,
			Syn::Unknown, // __LastKw
			// Glyphs //////////////////////////////////////////////////////////
			Syn::Ampersand,
			Syn::Ampersand2,
			Syn::AmpersandEq,
			Syn::AngleL,
			Syn::AngleL2,
			Syn::AngleLEq,
			Syn::AngleL2Eq,
			Syn::AngleR,
			Syn::AngleREq,
			Syn::AngleR2,
			Syn::AngleR3,
			Syn::AngleR2Eq,
			Syn::AngleR3Eq,
			Syn::AngleLAngleREq,
			Syn::Asterisk,
			Syn::Asterisk2,
			Syn::AsteriskEq,
			Syn::At,
			Syn::Backslash,
			Syn::Bang,
			Syn::BangEq,
			Syn::BraceL,
			Syn::BraceR,
			Syn::BracketL,
			Syn::BracketR,
			Syn::Caret,
			Syn::CaretEq,
			Syn::Colon,
			Syn::Colon2,
			Syn::Comma,
			Syn::Dollar,
			Syn::Dot,
			Syn::Dot2,
			Syn::Dot3,
			Syn::Eq,
			Syn::Eq2,
			Syn::Tilde,
			Syn::TildeEq2,
			Syn::Minus,
			Syn::Minus2,
			Syn::Minus4,
			Syn::MinusEq,
			Syn::ParenL,
			Syn::ParenR,
			Syn::Percent,
			Syn::PercentEq,
			Syn::Pipe,
			Syn::Pipe2,
			Syn::PipeEq,
			Syn::Plus,
			Syn::Plus2,
			Syn::PlusEq,
			Syn::Pound,
			Syn::Pound4,
			Syn::Question,
			Syn::Semicolon,
			Syn::Slash,
			Syn::SlashEq,
			Syn::Unknown, // ThinArrow
			// Miscellaneous ///////////////////////////////////////////////////
			Syn::PoundInclude,
			Syn::RegionStart,
			Syn::RegionEnd,
			Syn::Ident,
			Syn::Whitespace,
			Syn::DocComment,
			Syn::Comment,
			Syn::Unknown,
			Syn::Unknown, // EOF; effectively unreachable.
		];

		const _STATIC_ASSERT: () = {
			if MAPPING.len() != Token::__Last as usize {
				panic!();
			}
		};

		MAPPING[value as usize]
	}
}

impl LangExt for Syn {
	type Token = Token;
	const EOF: Self::Token = Token::Eof;
	const ERR_NODE: Self::Kind = Syn::Error;
}
