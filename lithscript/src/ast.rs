//! Structures for describing a LithScript abstract syntax tree.

mod expr;
mod item;
mod lit;
mod stat;

use doomfront::rowan::{self, ast::AstNode};

use doomfront::{simple_astnode, AstError, AstResult};

use crate::SyntaxElem;

use super::{Syn, SyntaxNode, SyntaxToken};

pub use self::{expr::*, item::*, lit::*, stat::*};

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
	Import(Import),
}

impl AstNode for TopLevel {
	type Language = Syn;

	fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		matches!(kind, Syn::Annotation | Syn::FuncDecl | Syn::Import)
	}

	fn cast(node: rowan::SyntaxNode<Self::Language>) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::Annotation => Some(Self::Annotation(Annotation(node))),
			Syn::FuncDecl => Some(Self::FuncDecl(FuncDecl(node))),
			Syn::Import => Some(Self::Import(Import(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &rowan::SyntaxNode<Self::Language> {
		match self {
			Self::Annotation(inner) => inner.syntax(),
			Self::FuncDecl(inner) => inner.syntax(),
			Self::Import(inner) => inner.syntax(),
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

	#[must_use]
	pub fn into_import(self) -> Option<Import> {
		match self {
			Self::Import(inner) => Some(inner),
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

/// Wraps a node tagged [`Syn::Import`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Import(pub(super) SyntaxNode);

simple_astnode!(Syn, Import, Syn::Import);

impl Import {
	/// The returned token is always tagged [`Syn::StringLit`].
	pub fn file_path(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syn::StringLit)
			})
			.ok_or(AstError::Missing)
	}

	#[must_use]
	pub fn group(&self) -> Option<ImportGroup> {
		self.0
			.last_child()
			.filter(|node| node.kind() == Syn::ImportGroup)
			.map(ImportGroup)
	}

	#[must_use]
	pub fn single(&self) -> Option<ImportEntry> {
		self.0
			.last_child()
			.filter(|node| node.kind() == Syn::ImportEntry)
			.map(ImportEntry)
	}

	/// If this is a `'*' => ident` import, return a [`Syn::Ident`] token
	/// for that trailing identifier.
	#[must_use]
	pub fn all_alias(&self) -> Option<Ident> {
		let Some(node) = self.0.last_child() else { return None; };
		let Syn::ImportEntry = node.kind() else { return None; };

		if node
			.first_token()
			.is_some_and(|token| token.kind() == Syn::Asterisk)
		{
			let ret = node.last_token().unwrap();
			debug_assert_eq!(ret.kind(), Syn::Ident);
			Some(Ident(ret))
		} else {
			None
		}
	}
}

/// Wraps a node tagged [`Syn::ImportGroup`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ImportGroup(SyntaxNode);

simple_astnode!(Syn, ImportGroup, Syn::ImportGroup);

impl ImportGroup {
	pub fn entries(&self) -> impl Iterator<Item = ImportEntry> {
		self.0.children().filter_map(ImportEntry::cast)
	}
}

/// Wraps a node tagged [`Syn::ImportEntry`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ImportEntry(SyntaxNode);

simple_astnode!(Syn, ImportEntry, Syn::ImportEntry);

impl ImportEntry {
	#[must_use]
	pub fn name(&self) -> Ident {
		let ret = self.0.first_child_or_token().unwrap().into_token().unwrap();
		debug_assert_eq!(ret.kind(), Syn::Ident);
		Ident(ret)
	}

	#[must_use]
	pub fn rename(&self) -> Option<Ident> {
		if !self.0.children_with_tokens().any(|elem| {
			elem.as_token()
				.is_some_and(|token| token.kind() == Syn::ThickArrow)
		}) {
			return None;
		}

		match self.0.last_child_or_token() {
			Some(SyntaxElem::Token(token)) => match token.kind() {
				Syn::Ident => Some(Ident(token)),
				_ => None,
			},
			_ => None,
		}
	}
}
