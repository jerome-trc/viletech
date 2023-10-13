//! Structures for representing Lithica abstract syntax trees.

mod expr;
mod item;
mod lit;

use doomfront::{
	rowan::{ast::AstNode, Direction},
	simple_astnode, AstError, AstResult,
};

use crate::{Syn, SyntaxNode, SyntaxToken};

pub use self::{expr::*, item::*, lit::*};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum TopLevel {
	Annotation(Annotation),
	Import(Import),
	Item(Item),
}

impl AstNode for TopLevel {
	type Language = Syn;

	fn can_cast(kind: Syn) -> bool
	where
		Self: Sized,
	{
		Item::can_cast(kind) || matches!(kind, Syn::Annotation | Syn::Import)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		if let Some(item) = Item::cast(node.clone()) {
			return Some(Self::Item(item));
		}

		if let Some(import) = Import::cast(node.clone()) {
			return Some(Self::Import(import));
		};

		if let Some(anno) = Annotation::cast(node) {
			return Some(Self::Annotation(anno));
		}

		None
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Annotation(inner) => inner.syntax(),
			Self::Import(inner) => inner.syntax(),
			Self::Item(inner) => inner.syntax(),
		}
	}
}

// Annotation //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::Annotation`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Annotation(SyntaxNode);

simple_astnode!(Syn, Annotation, Syn::Annotation);

impl Annotation {
	/// The returned token is always tagged [`Syn::Ident`].
	pub fn name(&self) -> AstResult<SyntaxToken> {
		let first_ident = self
			.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|t| t.kind() == Syn::Ident))
			.ok_or(AstError::Missing)?;

		let mut dot_seen = false;

		for elem in first_ident.siblings_with_tokens(Direction::Next) {
			match elem.kind() {
				Syn::Dot => dot_seen = true,
				Syn::Ident => {
					if dot_seen {
						return Ok(elem.into_token().unwrap());
					}
				}
				_ => continue,
			}
		}

		Err(AstError::Missing)
	}

	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn namespace(&self) -> Option<SyntaxToken> {
		let first_ident = self
			.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|t| t.kind() == Syn::Ident));

		let Some(first_ident) = first_ident else {
			return None;
		};

		if first_ident
			.siblings_with_tokens(Direction::Next)
			.any(|e| e.kind() == Syn::Dot)
		{
			return Some(first_ident);
		}

		None
	}

	#[must_use]
	pub fn arg_list(&self) -> Option<ArgList> {
		self.0.last_child().and_then(ArgList::cast)
	}
}

// ArgList, Argument ///////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ArgList`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ArgList(SyntaxNode);

simple_astnode!(Syn, ArgList, Syn::ArgList);

impl ArgList {
	pub fn iter(&self) -> impl Iterator<Item = Argument> {
		self.0.children().filter_map(Argument::cast)
	}
}

/// Wraps a node tagged [`Syn::Argument`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Argument(SyntaxNode);

simple_astnode!(Syn, Argument, Syn::Argument);

impl Argument {
	pub fn expr(&self) -> AstResult<Expr> {
		Expr::cast(self.0.last_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}

	#[must_use]
	pub fn name(&self) -> Option<Name> {
		let mut name = None;
		let mut colon_seen = false;

		for elem in self.0.children_with_tokens() {
			match elem.kind() {
				Syn::Colon => colon_seen = true,
				Syn::Ident | Syn::LitName => {
					name = Some(elem.into_token().unwrap());
					break;
				}
				_ => {}
			}
		}

		if colon_seen {
			name.map(Name)
		} else {
			None
		}
	}
}

// DocComment //////////////////////////////////////////////////////////////////

/// Wraps a [`Syn::DocComment`] token. Provides a convenience function for
/// stripping preceding slashes and surrounding whitespace.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocComment(SyntaxToken);

impl DocComment {
	#[must_use]
	pub fn text_trimmed(&self) -> &str {
		self.0.text().trim_matches('/').trim()
	}
}

impl std::ops::Deref for DocComment {
	type Target = SyntaxToken;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

// Import //////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Import {
	List {
		/// Tagged [`Syn::Import`].
		node: SyntaxNode,
		list: ImportList,
	},
	All {
		/// Tagged [`Syn::Import`].
		node: SyntaxNode,
		inner: ImportAll,
	},
}

impl AstNode for Import {
	type Language = Syn;

	fn can_cast(kind: Syn) -> bool
	where
		Self: Sized,
	{
		kind == Syn::Import
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		let child = node.last_child().unwrap();

		match child.kind() {
			Syn::ImportList => Some(Self::List {
				node,
				list: ImportList(child),
			}),
			Syn::ImportAll => Some(Self::All {
				node,
				inner: ImportAll(child),
			}),
			_ => unreachable!(),
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::List { node, .. } => node,
			Self::All { node, .. } => node,
		}
	}
}

impl Import {
	/// The returned token is always tagged [`Syn::LitString`].
	pub fn path(&self) -> AstResult<LitToken> {
		self.syntax()
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|t| t.kind() == Syn::LitString)
					.map(LitToken)
			})
			.ok_or(AstError::Missing)
	}
}

/// Wraps a node tagged [`Syn::ImportList`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ImportList(SyntaxNode);

simple_astnode!(Syn, ImportList, Syn::ImportList);

impl ImportList {
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
		self.0
			.first_token()
			.filter(|t| matches!(t.kind(), Syn::Ident | Syn::LitName))
			.map(Name)
			.ok_or(AstError::Missing)
	}

	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn rename(&self) -> Option<SyntaxToken> {
		let Some(ret) = self.0.last_token().filter(|t| t.kind() == Syn::Ident) else {
			return None;
		};

		let Some(arrow) = self
			.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|t| t.kind() == Syn::ThickArrow))
		else {
			return None;
		};

		(ret.index() > arrow.index()).then_some(ret)
	}
}

/// Wraps a node tagged [`Syn::ImportAll`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ImportAll(SyntaxNode);

simple_astnode!(Syn, ImportAll, Syn::ImportAll);

impl ImportAll {
	/// The returned token is always tagged [`Syn::Ident`].
	pub fn rename(&self) -> AstResult<SyntaxToken> {
		let ret = self.0.last_token().ok_or(AstError::Missing)?;

		(ret.kind() == Syn::Ident)
			.then_some(ret)
			.ok_or(AstError::Incorrect)
	}
}

// Name ////////////////////////////////////////////////////////////////////////

/// A convenience for positions which accept either [`Syn::Ident`] or [`Syn::LitName`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Name(pub(super) SyntaxToken);

impl Name {
	#[must_use]
	pub fn text(&self) -> &str {
		match self.0.kind() {
			Syn::Ident => self.0.text(),
			Syn::LitName => &self.0.text()[1..(self.0.text().len() - 1)],
			_ => unreachable!(),
		}
	}

	#[must_use]
	pub fn inner(&self) -> &SyntaxToken {
		&self.0
	}
}

// TypeSpec ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::TypeSpec`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeSpec(SyntaxNode);

simple_astnode!(Syn, TypeSpec, Syn::TypeSpec);

impl TypeSpec {
	pub fn expr(&self) -> AstResult<Expr> {
		Expr::cast(self.0.first_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}
}

// Common AST helper functions /////////////////////////////////////////////////

fn doc_comments(node: &SyntaxNode) -> impl Iterator<Item = DocComment> {
	node.children_with_tokens()
		.take_while(|elem| elem.kind().is_trivia() || elem.kind() == Syn::DocComment)
		.filter_map(|elem| {
			elem.into_token()
				.filter(|token| token.kind() == Syn::DocComment)
				.map(DocComment)
		})
}
