//! Tags representing syntax nodes, from low-level primitives to high-level composites.

use crate::{LangExt, zdoom::Token};

/// Tags representing syntax nodes, from low-level primitives to high-level composites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Syn {
	// Nodes: high-level composites ////////////////////////////////////////////
	/// `'automap' '{' kvp* '}'`
	AutomapDef,
	/// [`Syn::KwClearEpisodes`] outside of a block.
	ClearEpisodes,
	/// `'cluster' '{' kvp* '}'`
	ClusterDef,
	/// `'conversationids' '{' kvp* '}'`
	ConversationDef,
	DamageTypeDef,
	/// `'defaultmap' '{' kvp* '}'`
	DefaultMapDef,
	/// `'doomednums' '{' kvp* '}'`
	EdNumsDef,
	/// `'episode' ident '{' kvp* '}'`
	EpisodeDef,
	/// A sequence of tokens that did not form a valid syntax element.
	Error,
	/// `'gameinfo' '{' kvp* '}'`
	GameInfoDef,
	/// [`Syn::KwInclude`] followed by a [`Syn::StringLit`].
	IncludeDirective,
	/// `'intermission' '{' kvp* '}'`
	IntermissionDef,
	/// `ident ('=' value (, value)*)?`
	/// Where `value` is a literal or identifier.
	KeyValuePair,
	/// `'map' ident ('lookup' string)? '{' kvp* '}'`
	MapDef,
	/// The top-level node, representing the whole file.
	Root,
	/// `'skill' '{' kvp* '}'`
	SkillDef,
	/// `'spawnnums' '{' kvp* '}'`
	SpawnNumDefs,
	// Tokens: literals ////////////////////////////////////////////////////////
	/// See [`crate::zdoom::lex::Token::StringLit`].
	StringLit,
	/// See [`crate::zdoom::lex::Token::IntLit`].
	IntLit,
	/// See [`crate::zdoom::lex::Token::FloatLit`].
	FloatLit,
	// Tokens: keywords ////////////////////////////////////////////////////////
	KwAutomap,
	KwClearEpisodes,
	KwCluster,
	KwConversationIds,
	KwDamageType,
	KwDefaultMap,
	KwDoomEdNums,
	KwEpisode,
	KwFalse,
	KwGameInfo,
	KwLookup,
	KwInclude,
	KwIntermission,
	KwMap,
	KwSkill,
	KwSpawnNums,
	KwTrue,
	// Tokens: glyphs //////////////////////////////////////////////////////////
	/// `{`
	BraceL,
	/// `}`
	BraceR,
	/// `,`
	Comma,
	/// `=`
	Eq,
	// Tokens: miscellaneous ///////////////////////////////////////////////////
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

impl LangExt for Syn {
	type Token = Token;
	const EOF: Self::Token = Token::Eof;
	const ERR_NODE: Self::Kind = Syn::Error;
}
