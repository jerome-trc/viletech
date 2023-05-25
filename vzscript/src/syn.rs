//! Tags representing syntax nodes, from low-level primitives to high-level composites.

use doomfront::rowan;

/// Tags representing syntax nodes, from low-level primitives to high-level composites.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Syn {
	// Nodes: high-level composites ////////////////////////////////////////////
	/// A top-level node representing a whole file.
	FileRoot,
	/// A top-level node representing a whole REPL submission.
	ReplRoot,
	// Tokens: miscellaenous ///////////////////////////////////////////////////
	/// Input considered invalid by the lexer.
	Error,
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
