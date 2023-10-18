//! Abstract syntax tree nodes for representing patterns.

use doomfront::{rowan::ast::AstNode, simple_astnode, AstError, AstResult};

use crate::{Syn, SyntaxNode, SyntaxToken};

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
	type Language = Syn;

	fn can_cast(kind: Syn) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syn::PatGrouped | Syn::PatIdent | Syn::PatLit | Syn::PatSlice | Syn::PatWildcard
		)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::PatGrouped => Some(Self::Grouped(PatGrouped(node))),
			Syn::PatIdent => Some(Self::Ident(PatIdent(node))),
			Syn::PatLit => Some(Self::Literal(PatLit(node))),
			Syn::PatSlice => Some(Self::Slice(PatSlice(node))),
			Syn::PatWildcard => Some(Self::Wildcard(PatWildcard(node))),
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

/// Wraps a node tagged [`Syn::PatGrouped`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PatGrouped(SyntaxNode);

simple_astnode!(Syn, PatGrouped, Syn::PatGrouped);

impl PatGrouped {
	pub fn inner(&self) -> AstResult<Pattern> {
		Pattern::cast(self.0.first_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}
}

// Ident ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::PatIdent`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PatIdent(SyntaxNode);

simple_astnode!(Syn, PatIdent, Syn::PatIdent);

impl PatIdent {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn token(&self) -> SyntaxToken {
		let ret = self.0.first_token().unwrap();
		debug_assert_eq!(ret.kind(), Syn::Ident);
		ret
	}
}

// Literal /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::PatLit`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PatLit(SyntaxNode);

simple_astnode!(Syn, PatLit, Syn::PatLit);

impl PatLit {
	/// The returned token is always tagged [`Syn::Minus`].
	/// Returns `None` if this pattern's last token is not tagged
	/// [`Syn::LitFloat`] or [`Syn::LitInt`].
	#[must_use]
	pub fn minus(&self) -> Option<SyntaxToken> {
		if !self
			.0
			.last_token()
			.is_some_and(|t| matches!(t.kind(), Syn::LitInt | Syn::LitFloat))
		{
			return None;
		}

		self.0.first_token().filter(|t| t.kind() == Syn::Minus)
	}

	pub fn token(&self) -> AstResult<LitToken> {
		let ret = self.0.last_token().unwrap();

		match ret.kind() {
			Syn::LitFalse
			| Syn::LitFloat
			| Syn::LitInt
			| Syn::LitName
			| Syn::LitString
			| Syn::LitTrue => Ok(LitToken(ret)),
			_ => Err(AstError::Missing),
		}
	}
}

// Slice ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::PatSlice`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PatSlice(SyntaxNode);

simple_astnode!(Syn, PatSlice, Syn::PatSlice);

impl PatSlice {
	pub fn elements(&self) -> impl Iterator<Item = Pattern> {
		self.0.children().filter_map(Pattern::cast)
	}
}

// Wildcard ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::PatWildcard`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PatWildcard(SyntaxNode);

simple_astnode!(Syn, PatWildcard, Syn::PatWildcard);

impl PatWildcard {
	/// The returned token is always tagged [`Syn::Underscore`].
	#[must_use]
	pub fn token(&self) -> SyntaxToken {
		let ret = self.0.first_token().unwrap();
		debug_assert_eq!(ret.kind(), Syn::Underscore);
		ret
	}
}
