//! Structures for representing Lithica abstract syntax trees.

mod expr;
mod item;
mod lit;

use doomfront::{rowan::ast::AstNode, simple_astnode, AstError, AstResult};

use crate::{Syn, SyntaxNode, SyntaxToken};

pub use self::{expr::*, item::*, lit::*};

// DocComment //////////////////////////////////////////////////////////////////

/// Wraps a [`Syn::DocComment`] token. Provides a convenience function for
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

/// A convenience for positions which accept either [`Syn::Ident`] or [`Syn::LitName`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Name(pub(super) SyntaxToken);

impl Name {
	#[must_use]
	pub fn text(&self) -> &str {
		match self.0.kind() {
			Syn::Ident => self.0.text(),
			Syn::LitName => &self.0.text()[1..(self.0.text().len() - 1)],
			_ => unreachable!(),
		}
	}

	#[must_use]
	pub fn inner(&self) -> &SyntaxToken {
		&self.0
	}
}

// TypeSpec ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::TypeSpec`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeSpec(SyntaxNode);

simple_astnode!(Syn, TypeSpec, Syn::TypeSpec);

impl TypeSpec {
	pub fn expr(&self) -> AstResult<Expr> {
		Expr::cast(self.0.first_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}
}

// Common AST helper functions /////////////////////////////////////////////////

fn doc_comments(node: &SyntaxNode) -> impl Iterator<Item = DocComment> {
	node.children_with_tokens()
		.take_while(|elem| elem.kind().is_trivia() || elem.kind() == Syn::DocComment)
		.filter_map(|elem| {
			elem.into_token()
				.filter(|token| token.kind() == Syn::DocComment)
				.map(DocComment)
		})
}
