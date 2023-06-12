//! Tags representing syntax nodes, from low-level primitives to high-level composites.

/// Tags representing syntax nodes, from low-level primitives to high-level composites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Syn {
	// Nodes: high-level composites ////////////////////////////////////////////
	/// Optional part at the end of a [`Syn::StateDef`].
	ActionFunction,
	/// `'(' exprs? ')'`
	ArgList,
	/// `('[' expr? ']')+`
	ArrayLen,
	/// `'class' 'ident' inheritspec? replacesclause? '{' innard* '}'`
	ClassDef,
	/// `'extend' 'class' ident '{' innard* '}'`
	ClassExtend,
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
	/// The first part of a for loop opener.
	ForInit,
	/// The secnod part of a for loop opener.
	ForCond,
	/// The third part of a for loop opener.
	ForIter,
	FunctionDecl,
	/// `'goto' 'super::'? identchain ('+' integer)?`
	GotoOffset,
	/// `(ident) | (ident ('.' ident)?)`
	///
	/// Known in ZScript's Lemon grammar as a "dottable ID".
	IdentChain,
	/// The `#include` preprocessor directive and its string literal argument.
	IncludeDirective,
	/// `':' ident`
	InheritSpec,
	/// Will have one of the following tokens as a child:
	/// - [`Syn::LitFalse`]
	/// - [`Syn::LitFloat`]
	/// - [`Syn::LitInt`]
	/// - [`Syn::LitName`]
	/// - [`Syn::LitNull`]
	/// - [`Syn::LitString`]
	/// - [`Syn::LitTrue`]
	Literal,
	LocalVar,
	LocalVarInit,
	/// `'mixin' 'class' ident '{' innard* '}'`
	MixinClassDef,
	ParamList,
	/// `identchain expr* ';'`
	PropertySetting,
	PropertyDef,
	/// `'replaces' ident`
	ReplacesClause,
	ReturnTypes,
	/// `'[' ident ']'`, between a call identifier and argument list.
	RngSpec,
	/// The top-level node, representing the whole file.
	Root,
	StateFlow,
	/// For child nodes under a [`Syn::StatesDef`].
	StateDef,
	/// `ident ':'`
	StateLabel,
	/// `'light' '(' string ')'`
	StateLight,
	/// `'offset' '(' expr ',' expr ')'`
	StateOffset,
	/// `'states' ident '{' innard* '}'`
	StatesDef,
	StatesUsage,
	/// `'struct' ident '{' innard* '}'`
	StructDef,
	/// `'extend' 'struct' ident '{' innard* '}'`
	StructExtend,
	/// `'[' expr ']'`
	Subscript,
	/// Can be [`Syn::KwLet`], [`Syn::IdentChain`], or `'readonly' '<' '@'? ident '>'`.
	TypeRef,
	/// The `version` preprocessor directive and its string literal argument.
	VersionDirective,
	/// `'version' '(' string ')'`
	VersionQual,
	// Nodes: expressions //////////////////////////////////////////////////////
	ArrayExpr,
	BinExpr,
	CallExpr,
	GroupExpr,
	IdentExpr,
	IndexExpr,
	PostfixExpr,
	PrefixExpr,
	SuperExpr,
	/// Two parentheses surrounding two, three, or four comma-separated expressions.
	///
	/// Used to construct vectors and colors.
	VectorExpr,
	// Nodes: statements ///////////////////////////////////////////////////////
	AssignStat,
	BreakStat,
	CompoundStat,
	ContinueStat,
	DeclAssignStat,
	DoUntilStat,
	DoWhileStat,
	/// A lone semicolon.
	EmptyStat,
	/// An expression followed by a semicolon.
	ExprStat,
	/// C-style, with a three-part (semicolon-delimited, parenthesis-enclosed) opener.
	ForStat,
	ForEachStat,
	/// For use in switch cases. May start with `'case' ident ':'` or `'default' ':'`.
	LabelledStat,
	LocalStat,
	MixinStat,
	ReturnStat,
	StaticConstStat,
	SwitchStat,
	UntilStat,
	WhileStat,
	// Tokens: literals ////////////////////////////////////////////////////////
	/// The exact string `false`.
	FalseLit,
	/// See [`crate::zdoom::lex::Token::FloatLit`].
	FloatLit,
	/// See [`crate::zdoom::lex::Token::IntLit`].
	IntLit,
	/// See [`crate::zdoom::lex::Token::NameLit`].
	NameLit,
	/// The exact string `null`.
	NullLit,
	/// The exact string `true`.
	TrueLit,
	/// See [`crate::zdoom::lex::Token::StringLit`].
	StringLit,
	// Tokens: keywords ////////////////////////////////////////////////////////
	KwAbstract,
	KwAction,
	KwAlignOf,
	KwArray,
	KwBreak,
	/// Only a keyword in [`Syn::StateDef`] elements.
	KwBright,
	/// Only a keyword in [`Syn::StateDef`] items.
	KwCanRaise,
	KwCase,
	KwClass,
	KwClearScope,
	KwConst,
	KwContinue,
	KwCross,
	/// Context-sensitive. Only a keyword within a [`Syn::ClassDef`].
	KwDefault,
	KwDeprecated,
	KwDo,
	KwDot,
	KwElse,
	KwEnum,
	KwExtend,
	/// Only a keyword in [`Syn::StateFlow`] elements.
	KwFail,
	/// Only a keyword in [`Syn::StateDef`] items.
	KwFast,
	KwFinal,
	KwFlagdef,
	KwForEach,
	KwFor,
	KwGoto,
	KwIf,
	KwInternal,
	KwIn,
	KwIs,
	KwLatent,
	KwLet,
	/// Only a keyword in [`Syn::StateLight`] elements.
	KwLight,
	/// Only a keyword in [`Syn::StateFlow`] elements.
	KwLoop,
	KwMap,
	KwMapIterator,
	KwMeta,
	KwMixin,
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
	KwReadonly,
	KwReturn,
	KwSizeof,
	/// Only a keyword in [`Syn::StateDef`] items.
	KwSlow,
	KwStates,
	KwStatic,
	/// Only a keyword in [`Syn::StateChange`] elements.
	KwStop,
	KwStruct,
	KwSuper,
	KwSwitch,
	KwReplaces,
	KwTransient,
	KwUi,
	KwUntil,
	KwVar,
	KwVarArg,
	/// Always child to a [`Syn::VersionQual`] node.
	KwVersion,
	KwVirtual,
	KwVirtualScope,
	/// Only a keyword in [`Syn::StateChange`] elements.
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
	/// `####`, `----`,
	/// or a combination of exactly 4 ASCII digits, ASCII letters, and underscores.
	StateSprite,
	StateFrames,
	// Tokens: foundational ////////////////////////////////////////////////////
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
