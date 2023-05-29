//! AST nodes for representing expressions.

use rowan::ast::AstNode;

use crate::simple_astnode;

use super::{
	super::{Syn, SyntaxNode, SyntaxToken},
	lit::Literal,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize))]
pub enum Expr {
	Call(ExprCall),
	Ident(ExprIdent),
	Literal(Literal),
}

impl AstNode for Expr {
	type Language = Syn;

	fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		matches!(kind, Syn::CallExpr | Syn::IdentExpr | Syn::Literal)
	}

	fn cast(node: rowan::SyntaxNode<Self::Language>) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::CallExpr => Some(Self::Call(ExprCall(node))),
			Syn::IdentExpr => Some(Self::Ident(ExprIdent(node))),
			Syn::Literal => Some(Self::Literal(Literal(node))),
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

/// Wraps a node tagged [`Syn::ExprCall`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize))]
pub struct ExprCall(pub(super) SyntaxNode);

simple_astnode!(Syn, ExprCall, Syn::CallExpr);

impl ExprCall {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.syntax().first_token().unwrap()
	}
}

/// Wraps a node tagged [`Syn::ExprIdent`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize))]
pub struct ExprIdent(pub(super) SyntaxNode);

simple_astnode!(Syn, ExprIdent, Syn::IdentExpr);

impl ExprIdent {
	#[must_use]
	pub fn token(&self) -> SyntaxToken {
		self.syntax().first_token().unwrap()
	}
}
