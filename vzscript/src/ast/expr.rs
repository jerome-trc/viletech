//! AST nodes for representing expressions.

use doomfront::{
	rowan::{self, ast::AstNode},
	simple_astnode,
};

use crate::{Syn, SyntaxNode};

use super::Literal;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize))]
pub enum Expr {
	Binary(BinExpr),
	Grouped(GroupedExpr),
	Literal(Literal),
}

impl AstNode for Expr {
	type Language = Syn;

	fn can_cast(kind: <Self::Language as doomfront::rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		matches!(kind, Syn::Literal | Syn::BinExpr | Syn::GroupedExpr)
	}

	fn cast(node: doomfront::rowan::SyntaxNode<Self::Language>) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::Literal => Some(Self::Literal(Literal(node))),
			Syn::BinExpr => Some(Self::Binary(BinExpr(node))),
			Syn::GroupedExpr => Some(Self::Grouped(GroupedExpr(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &doomfront::rowan::SyntaxNode<Self::Language> {
		match self {
			Self::Binary(node) => node.syntax(),
			Self::Grouped(node) => node.syntax(),
			Self::Literal(node) => node.syntax(),
		}
	}
}

impl Expr {
	#[must_use]
	pub fn into_literal(self) -> Option<Literal> {
		match self {
			Self::Literal(lit) => Some(lit),
			_ => None,
		}
	}
}

/// Wraps a node tagged [`Syn::BinExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize))]
pub struct BinExpr(pub(super) SyntaxNode);

simple_astnode!(Syn, BinExpr, Syn::BinExpr);

impl BinExpr {
	#[must_use]
	pub fn left(&self) -> Expr {
		Expr::cast(self.0.first_child().unwrap()).unwrap()
	}

	#[must_use]
	pub fn right(&self) -> Expr {
		Expr::cast(self.0.last_child().unwrap()).unwrap()
	}
}

/// Wraps a node tagged [`Syn::GroupedExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize))]
pub struct GroupedExpr(pub(super) SyntaxNode);

simple_astnode!(Syn, GroupedExpr, Syn::GroupedExpr);

impl GroupedExpr {
	#[must_use]
	pub fn inner(&self) -> Expr {
		Expr::cast(self.0.first_child().unwrap()).unwrap()
	}
}
