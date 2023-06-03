//! Structures for describing a VZScript abstract syntax tree.

pub mod expr;
pub mod lit;

use doomfront::rowan::{self, ast::AstNode};

use doomfront::simple_astnode;

use super::{Syn, SyntaxNode, SyntaxToken};

pub use self::{expr::*, lit::*};

/// A top-level element in a source file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize))]
pub enum FileRoot {
	/// Only "inner" annotations are allowed in this position.
	Annotation(Annotation),
}

impl AstNode for FileRoot {
	type Language = Syn;

	fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		matches!(kind, Syn::Annotation)
	}

	fn cast(node: rowan::SyntaxNode<Self::Language>) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::Annotation => Some(Self::Annotation(Annotation(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &rowan::SyntaxNode<Self::Language> {
		match self {
			Self::Annotation(anno) => &anno.0,
		}
	}
}

/// A top-level element in a REPL submission.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize))]
pub enum ReplRoot {
	Expr(Expr),
}

impl AstNode for ReplRoot {
	type Language = Syn;

	fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		Expr::can_cast(kind)
	}

	fn cast(node: rowan::SyntaxNode<Self::Language>) -> Option<Self>
	where
		Self: Sized,
	{
		Expr::cast(node).map(Self::Expr)
	}

	fn syntax(&self) -> &rowan::SyntaxNode<Self::Language> {
		match self {
			Self::Expr(expr) => expr.syntax(),
		}
	}
}

/// Wraps a node tagged [`Syn::Annotation`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize))]
pub struct Annotation(SyntaxNode);

simple_astnode!(Syn, Annotation, Syn::Annotation);

impl Annotation {
	/// Returns `true` if this annotation uses the syntax `#![]` instead of `#[]`.
	#[must_use]
	pub fn is_inner(&self) -> bool {
		self.0.children_with_tokens().nth(1).unwrap().kind() == Syn::Bang
	}
}
