//! AST nodes for representing expressions.

use doomfront::rowan::{self, ast::AstNode};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
	Literal(Literal),
	Name(Name),
}

impl AstNode for Expression {
	type Language = Syn;

	fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		matches!(kind, |Syn::Name| Syn::Literal)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::Literal => Some(Self::Literal(Literal(node))),
			Syn::Name => Some(Self::Name(Name(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Literal(inner) => &inner.0,
			Self::Name(inner) => &inner.0,
		}
	}
}

impl Expression {
	#[must_use]
	pub fn into_literal(self) -> Option<Literal> {
		match self {
			Self::Literal(lit) => Some(lit),
			_ => None,
		}
	}
}
