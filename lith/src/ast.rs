//! Structures for representing Lithica abstract syntax trees.

mod expr;
mod lit;
mod pat;
mod stmt;

use doomfront::{simple_astnode, AstError, AstResult};

use crate::{Syntax, SyntaxNode, SyntaxToken};

pub use self::{expr::*, lit::*, pat::*, stmt::*};

// BlockLabel //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::BlockLabel`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct BlockLabel(SyntaxNode);

simple_astnode!(Syntax, BlockLabel, Syntax::BlockLabel);

impl BlockLabel {
	/// The returned token is always tagged [`Syntax::Ident`].
	pub fn ident(&self) -> AstResult<SyntaxToken> {
		let Some(opener) = self.0.first_token() else {
			return Err(AstError::Incorrect);
		};

		let Some(closer) = self.0.last_token() else {
			return Err(AstError::Incorrect);
		};

		if opener.kind() != Syntax::Colon2 || closer.kind() != Syntax::Colon2 {
			return Err(AstError::Incorrect);
		}

		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|t| t.kind() == Syntax::Ident))
			.ok_or(AstError::Missing)
	}
}

// DocComment //////////////////////////////////////////////////////////////////

/// Wraps a [`Syntax::DocComment`] token. Provides a convenience function for
/// stripping preceding slashes and surrounding whitespace.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocComment(SyntaxToken);

impl DocComment {
	#[must_use]
	pub fn text_trimmed(&self) -> &str {
		self.0.text().trim_matches('/').trim()
	}
}

impl std::ops::Deref for DocComment {
	type Target = SyntaxToken;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

// Name ////////////////////////////////////////////////////////////////////////

/// A convenience for positions which accept either [`Syntax::Ident`] or [`Syntax::LitName`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Name(pub(super) SyntaxToken);

impl Name {
	#[must_use]
	pub fn text(&self) -> &str {
		match self.0.kind() {
			Syntax::Ident => self.0.text(),
			Syntax::LitName => &self.0.text()[1..(self.0.text().len() - 1)],
			_ => unreachable!(),
		}
	}

	#[must_use]
	pub fn inner(&self) -> &SyntaxToken {
		&self.0
	}
}

// TypeSpec ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::TypeSpec`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TypeSpec(SyntaxNode);

simple_astnode!(Syntax, TypeSpec, Syntax::TypeSpec);

impl TypeSpec {
	#[must_use]
	pub fn expr(&self) -> ExprType {
		let ret = self.0.first_child().unwrap();
		debug_assert_eq!(ret.kind(), Syntax::ExprType);
		ExprType(ret)
	}
}
