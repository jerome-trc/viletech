//! Tags representing syntax nodes, from low-level primitives to high-level composites.

/// Tags representing syntax nodes, from low-level primitives to high-level composites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Syn {
	// Nodes: high-level composites ////////////////////////////////////////////
	/// A sequence of tokens that did not form a valid syntax element.
	Error,
	/// `ident '=' string ';'`
	KeyValuePair,
	/// `'[' locale 'default'? ']'`
	LocaleTag,
	/// The top-level node, representing the whole file.
	Root,
	// Tokens //////////////////////////////////////////////////////////////////
	/// See [`crate::zdoom::lex::Token::StringLit`].
	StringLit,

	KwDefault,

	/// `[`
	BracketL,
	/// `]`
	BracketR,
	/// `=`
	Eq,
	/// `;`
	Semicolon,
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
