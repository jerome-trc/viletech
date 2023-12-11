//! Abstract syntax tree nodes for representing patterns.

use doomfront::{rowan::ast::AstNode, simple_astnode, AstError, AstResult};

use crate::{Syntax, SyntaxNode, SyntaxToken};

use super::LitToken;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Pattern {
	Grouped(PatGrouped),
	Ident(PatIdent),
	Literal(PatLit),
	Slice(PatSlice),
	Wildcard(PatWildcard),
}

impl AstNode for Pattern {
	type Language = Syntax;

	fn can_cast(kind: Syntax) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syntax::PatGrouped
				| Syntax::PatIdent
				| Syntax::PatLit | Syntax::PatSlice
				| Syntax::PatWildcard
		)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syntax::PatGrouped => Some(Self::Grouped(PatGrouped(node))),
			Syntax::PatIdent => Some(Self::Ident(PatIdent(node))),
			Syntax::PatLit => Some(Self::Literal(PatLit(node))),
			Syntax::PatSlice => Some(Self::Slice(PatSlice(node))),
			Syntax::PatWildcard => Some(Self::Wildcard(PatWildcard(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Grouped(inner) => inner.syntax(),
			Self::Ident(inner) => inner.syntax(),
			Self::Literal(inner) => inner.syntax(),
			Self::Slice(inner) => inner.syntax(),
			Self::Wildcard(inner) => inner.syntax(),
		}
	}
}

// Grouped /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::PatGrouped`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PatGrouped(SyntaxNode);

simple_astnode!(Syntax, PatGrouped, Syntax::PatGrouped);

impl PatGrouped {
	pub fn inner(&self) -> AstResult<Pattern> {
		Pattern::cast(self.0.first_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}
}

// Ident ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::PatIdent`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PatIdent(SyntaxNode);

simple_astnode!(Syntax, PatIdent, Syntax::PatIdent);

impl PatIdent {
	/// The returned token is always tagged [`Syntax::Ident`].
	#[must_use]
	pub fn token(&self) -> SyntaxToken {
		let ret = self.0.first_token().unwrap();
		debug_assert_eq!(ret.kind(), Syntax::Ident);
		ret
	}
}

// Literal /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::PatLit`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PatLit(SyntaxNode);

simple_astnode!(Syntax, PatLit, Syntax::PatLit);

impl PatLit {
	/// The returned token is always tagged [`Syntax::Minus`].
	/// Returns `None` if this pattern's last token is not tagged
	/// [`Syntax::LitFloat`] or [`Syntax::LitInt`].
	#[must_use]
	pub fn minus(&self) -> Option<SyntaxToken> {
		if !self
			.0
			.last_token()
			.is_some_and(|t| matches!(t.kind(), Syntax::LitInt | Syntax::LitFloat))
		{
			return None;
		}

		self.0.first_token().filter(|t| t.kind() == Syntax::Minus)
	}

	pub fn token(&self) -> AstResult<LitToken> {
		let ret = self.0.last_token().unwrap();

		match ret.kind() {
			Syntax::LitFalse
			| Syntax::LitFloat
			| Syntax::LitInt
			| Syntax::LitName
			| Syntax::LitString
			| Syntax::LitTrue => Ok(LitToken(ret)),
			_ => Err(AstError::Missing),
		}
	}
}

// Slice ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::PatSlice`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PatSlice(SyntaxNode);

simple_astnode!(Syntax, PatSlice, Syntax::PatSlice);

impl PatSlice {
	pub fn elements(&self) -> impl Iterator<Item = Pattern> {
		self.0.children().filter_map(Pattern::cast)
	}
}

// Wildcard ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::PatWildcard`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PatWildcard(SyntaxNode);

simple_astnode!(Syntax, PatWildcard, Syntax::PatWildcard);

impl PatWildcard {
	/// The returned token is always tagged [`Syntax::Underscore`].
	#[must_use]
	pub fn token(&self) -> SyntaxToken {
		let ret = self.0.first_token().unwrap();
		debug_assert_eq!(ret.kind(), Syntax::Underscore);
		ret
	}
}
