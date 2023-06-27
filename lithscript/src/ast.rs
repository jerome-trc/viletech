//! Structures for describing a LithScript abstract syntax tree.

mod expr;
mod item;
mod lit;

use doomfront::rowan::{self, ast::AstNode};

use doomfront::simple_astnode;

use super::{Syn, SyntaxNode, SyntaxToken};

pub use self::{expr::*, item::*, lit::*};

/// Wraps a token tagged [`Syn::Ident`].
/// Exists for the convenience of automatically handling raw identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Ident(pub(self) SyntaxToken);

impl Ident {
	#[must_use]
	pub fn text(&self) -> &str {
		let text = self.0.text();

		if !text.starts_with("r#") {
			text
		} else {
			&text[2..]
		}
	}

	#[must_use]
	pub fn is_raw(&self) -> bool {
		self.0.text().starts_with("r#")
	}
}

/// A top-level element in a source file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum TopLevel {
	/// Only "inner" annotations are allowed in this position.
	Annotation(Annotation),
	FuncDecl(FuncDecl),
}

impl AstNode for TopLevel {
	type Language = Syn;

	fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		matches!(kind, Syn::Annotation | Syn::FuncDecl)
	}

	fn cast(node: rowan::SyntaxNode<Self::Language>) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::Annotation => Some(Self::Annotation(Annotation(node))),
			Syn::FuncDecl => Some(Self::FuncDecl(FuncDecl(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &rowan::SyntaxNode<Self::Language> {
		match self {
			Self::Annotation(inner) => inner.syntax(),
			Self::FuncDecl(inner) => inner.syntax(),
		}
	}
}

impl TopLevel {
	#[must_use]
	pub fn into_func_decl(self) -> Option<FuncDecl> {
		match self {
			Self::FuncDecl(inner) => Some(inner),
			_ => None,
		}
	}
}

/// A top-level element in a REPL submission.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
			Self::Expr(inner) => inner.syntax(),
		}
	}
}

/// Wraps a node tagged [`Syn::Annotation`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Annotation(SyntaxNode);

simple_astnode!(Syn, Annotation, Syn::Annotation);

impl Annotation {
	/// Returns `true` if this annotation uses the syntax `#![]` instead of `#[]`.
	#[must_use]
	pub fn is_inner(&self) -> bool {
		self.0.children_with_tokens().nth(1).unwrap().kind() == Syn::Bang
	}
}
