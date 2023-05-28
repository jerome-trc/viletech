//! Tags representing syntax nodes, from low-level primitives to high-level composites.

/// Tags representing syntax nodes, from low-level primitives to high-level composites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize, serde::Deserialize))]
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
	Flags,
	/// The top-level node, representing the whole file.
	Root,
	/// The type specifier is always followed by the identifier.
	TypeSpec,
	// Tokens: literals ////////////////////////////////////////////////////////
	/// The boolean literal `false`.
	LitFalse,
	/// A (G)ZDoom (i.e. C/C++-style) floating-point literal.
	LitFloat,
	/// A (G)ZDoom (i.e. C/C++-style) integer literal.
	LitInt,
	/// Delimited by double quotes. Also used for defining default color values.
	LitString,
	/// The boolean literal `true`.
	LitTrue,
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
	// Tokens: glyphs //////////////////////////////////////////////////////////
	/// The `=` character.
	Eq,
	/// The `;` character, used as a terminator.
	Semicolon,
	// Tokens: miscellaenous ///////////////////////////////////////////////////
	/// A name for a defined CVar.
	Ident,
	/// Either single-line (C++-style) or multi-line (C-style).
	Comment,
	/// Input that the lexer considered to be invalid.
	Unknown,
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
