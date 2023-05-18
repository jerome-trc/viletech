//! AST nodes for representing expressions.

use doomfront::rowan::{self, ast::AstNode};

use super::{lit::*, *};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum Expr {
	Literal(Literal),
	Resolver(Resolver),
}

impl AstNode for Expr {
	type Language = Syn;

	fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		matches!(kind, Syn::Literal | Syn::Resolver)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::Literal => Some(Self::Literal(Literal(node))),
			Syn::Name => Some(Self::Resolver(Resolver(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Literal(inner) => &inner.0,
			Self::Resolver(inner) => &inner.0,
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
