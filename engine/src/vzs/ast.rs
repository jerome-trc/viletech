//! Abstract syntax tree nodes.

mod expr;
mod lit;

use doomfront::{
	rowan::{self, ast::AstNode},
	simple_astnode,
};
use serde::Serialize;

use super::{Syn, SyntaxNode, SyntaxToken};

pub use self::{expr::*, lit::*};

/// A top-level element in a source file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Annotation(SyntaxNode);

simple_astnode!(Syn, Annotation, Syn::Annotation);

impl Annotation {
	/// Returns `true` if this annotation uses the syntax `#![]` instead of `#[]`.
	#[must_use]
	pub fn is_inner(&self) -> bool {
		self.0.children_with_tokens().nth(1).unwrap().kind() == Syn::Bang
	}

	#[must_use]
	pub fn resolver(&self) -> Resolver {
		self.0.children().find_map(Resolver::cast).unwrap()
	}

	#[must_use]
	pub fn args(&self) -> Option<ArgList> {
		self.0.children().find_map(ArgList::cast)
	}
}

/// Wraps a node tagged [`Syn::Argument`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Argument(SyntaxNode);

simple_astnode!(Syn, Argument, Syn::Argument);

impl Argument {
	/// Returns a token tagged [`Syn::Ident`].
	#[must_use]
	pub fn label(&self) -> Option<SyntaxToken> {
		self.0
			.first_child_or_token()
			.filter(|elem| elem.kind() == Syn::Ident)
			.map(|elem| elem.into_token().unwrap())
	}

	#[must_use]
	pub fn expr(&self) -> Expr {
		Expr::cast(self.0.last_child().unwrap()).unwrap()
	}
}

/// Wraps a node tagged [`Syn::ArgList`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct ArgList(SyntaxNode);

simple_astnode!(Syn, ArgList, Syn::ArgList);

impl ArgList {
	pub fn iter(&self) -> impl Iterator<Item = Argument> {
		self.0.children().filter_map(Argument::cast)
	}
}

/// Wraps a node tagged [`Syn::Name`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Name(SyntaxNode);

simple_astnode!(Syn, Name, Syn::Name);

impl Name {
	/// Shorthand for
	/// `self.syntax().first_child_or_token().unwrap().into_token().unwrap()`.
	#[must_use]
	pub fn token(&self) -> SyntaxToken {
		self.0.first_child_or_token().unwrap().into_token().unwrap()
	}
}

/// Wraps a [`Syn::Resolver`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Resolver(SyntaxNode);

simple_astnode!(Syn, Resolver, Syn::Resolver);

impl Resolver {
	/// Every token returns is tagged [`Syn::Ident`].
	pub fn parts(&self) -> impl Iterator<Item = SyntaxToken> {
		self.0.children_with_tokens().filter_map(|n_or_t| {
			if n_or_t.kind() == Syn::Ident {
				n_or_t.into_token()
			} else {
				None
			}
		})
	}
}
