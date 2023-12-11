//! AST nodes for representing expressions.

use rowan::ast::AstNode;

use crate::simple_astnode;

use super::{
	super::{Syntax, SyntaxNode, SyntaxToken},
	lit::Literal,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Expr {
	Call(ExprCall),
	Ident(ExprIdent),
	Literal(Literal),
}

impl AstNode for Expr {
	type Language = Syntax;

	fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		matches!(kind, Syntax::CallExpr | Syntax::IdentExpr | Syntax::Literal)
	}

	fn cast(node: rowan::SyntaxNode<Self::Language>) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syntax::CallExpr => Some(Self::Call(ExprCall(node))),
			Syntax::IdentExpr => Some(Self::Ident(ExprIdent(node))),
			Syntax::Literal => Some(Self::Literal(Literal(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &rowan::SyntaxNode<Self::Language> {
		match self {
			Self::Call(inner) => inner.syntax(),
			Self::Ident(inner) => inner.syntax(),
			Self::Literal(inner) => inner.syntax(),
		}
	}
}

impl Expr {
	#[must_use]
	pub fn into_ident(self) -> Option<ExprIdent> {
		match self {
			Self::Ident(inner) => Some(inner),
			_ => None,
		}
	}

	#[must_use]
	pub fn into_literal(self) -> Option<Literal> {
		match self {
			Self::Literal(inner) => Some(inner),
			_ => None,
		}
	}
}

/// Wraps a node tagged [`Syntax::CallExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprCall(pub(super) SyntaxNode);

simple_astnode!(Syntax, ExprCall, Syntax::CallExpr);

impl ExprCall {
	/// The returned token is always tagged [`Syntax::Ident`].
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.syntax().first_token().unwrap()
	}
}

/// Wraps a node tagged [`Syntax::IdentExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprIdent(pub(super) SyntaxNode);

simple_astnode!(Syntax, ExprIdent, Syntax::IdentExpr);

impl ExprIdent {
	#[must_use]
	pub fn token(&self) -> SyntaxToken {
		self.syntax().first_token().unwrap()
	}
}
