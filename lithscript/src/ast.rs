//! Structures for describing a LithScript abstract syntax tree.

mod expr;
mod item;
mod lit;
mod stat;

use doomfront::rowan::{self, ast::AstNode};

use doomfront::{simple_astnode, AstError, AstResult};

use super::{Syn, SyntaxNode, SyntaxToken};

pub use self::{expr::*, item::*, lit::*, stat::*};

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

/// A convenience for positions which accept either [`Syn::Ident`] or [`Syn::NameLit`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Name(SyntaxToken);

impl Name {
	#[must_use]
	pub fn text(&self) -> &str {
		match self.0.kind() {
			Syn::Ident => self.0.text(),
			Syn::NameLit => &self.0.text()[1..(self.0.text().len() - 1)],
			_ => unreachable!(),
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
	pub fn all_alias(&self) -> Option<SyntaxToken> {
		let Some(node) = self.0.last_child() else { return None; };
		let Syn::ImportEntry = node.kind() else { return None; };

		if node
			.first_token()
			.is_some_and(|token| token.kind() == Syn::Asterisk)
		{
			node.last_token().filter(|token| token.kind() == Syn::Ident)
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
	pub fn name(&self) -> AstResult<Name> {
		let token = self.0.first_token().ok_or(AstError::Missing)?;

		match token.kind() {
			Syn::Ident | Syn::NameLit => Ok(Name(token)),
			_ => Err(AstError::Incorrect),
		}
	}

	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn rename(&self) -> Option<SyntaxToken> {
		self.0
			.children_with_tokens()
			.filter_map(|elem| elem.into_token())
			.skip_while(|token| token.kind() != Syn::ThickArrow)
			.find(|token| token.kind() == Syn::Ident)
	}
}
