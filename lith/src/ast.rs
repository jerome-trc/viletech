//! Structures for representing Lithica abstract syntax trees.

mod expr;
mod item;
mod lit;
mod pat;
mod stmt;

use doomfront::{
	rowan::{ast::AstNode, Direction, TextRange},
	simple_astnode, AstError, AstResult,
};

use crate::{Syn, SyntaxNode, SyntaxToken};

pub use self::{expr::*, item::*, lit::*, pat::*, stmt::*};

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

/// Anything that can inhabit a [`FunctionBody`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum CoreElement {
	Annotation(Annotation),
	Import(Import),
	Item(Item),
	Statement(Statement),
}

impl AstNode for CoreElement {
	type Language = Syn;

	fn can_cast(kind: Syn) -> bool
	where
		Self: Sized,
	{
		Statement::can_cast(kind)
			|| Item::can_cast(kind)
			|| Annotation::can_cast(kind)
			|| Import::can_cast(kind)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		if let Some(statement) = Statement::cast(node.clone()) {
			return Some(Self::Statement(statement));
		}

		if let Some(item) = Item::cast(node.clone()) {
			return Some(Self::Item(item));
		}

		if let Some(anno) = Annotation::cast(node.clone()) {
			return Some(Self::Annotation(anno));
		}

		if let Some(import) = Import::cast(node.clone()) {
			return Some(Self::Import(import));
		}

		None
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Statement(inner) => inner.syntax(),
			Self::Item(inner) => inner.syntax(),
			Self::Annotation(inner) => inner.syntax(),
			Self::Import(inner) => inner.syntax(),
		}
	}
}

// Annotation //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::Annotation`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Annotation(pub(super) SyntaxNode);

simple_astnode!(Syn, Annotation, Syn::Annotation);

impl Annotation {
	pub fn name(&self) -> AstResult<AnnotationName> {
		let ident0 = self
			.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|t| t.kind() == Syn::Ident))
			.ok_or(AstError::Missing)?;

		let dot_opt = ident0
			.siblings_with_tokens(Direction::Next)
			.find_map(|elem| elem.into_token().filter(|t| t.kind() == Syn::Dot));

		let Some(dot) = dot_opt else {
			return Ok(AnnotationName::Unscoped(ident0));
		};

		if let Some(ident1) = dot
			.siblings_with_tokens(Direction::Next)
			.find_map(|elem| elem.into_token().filter(|t| t.kind() == Syn::Ident))
		{
			return Ok(AnnotationName::Scoped(ident0, ident1));
		}

		Ok(AnnotationName::Unscoped(ident0))
	}

	#[must_use]
	pub fn arg_list(&self) -> Option<ArgList> {
		self.0.last_child().and_then(ArgList::cast)
	}
}

/// All tokens herein are always tagged [`Syn::Ident`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum AnnotationName {
	Unscoped(SyntaxToken),
	Scoped(SyntaxToken, SyntaxToken),
}

impl AnnotationName {
	#[must_use]
	pub fn text(&self) -> (&str, Option<&str>) {
		match self {
			Self::Unscoped(ident) => (ident.text(), None),
			Self::Scoped(ident0, ident1) => (ident0.text(), Some(ident1.text())),
		}
	}

	#[must_use]
	pub fn text_range(&self) -> TextRange {
		match self {
			Self::Unscoped(ident) => ident.text_range(),
			Self::Scoped(ident0, ident1) => {
				TextRange::new(ident0.text_range().start(), ident1.text_range().end())
			}
		}
	}
}

// ArgList, Argument ///////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ArgList`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ArgList(SyntaxNode);

simple_astnode!(Syn, ArgList, Syn::ArgList);

impl ArgList {
	/// The returned token is always tagged [`Syn::Dot3`].
	#[must_use]
	pub fn dot3(&self) -> Option<SyntaxToken> {
		let Some(ret) = self
			.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|t| t.kind() == Syn::Dot3))
		else {
			return None;
		};

		if ret
			.next_sibling_or_token()
			.is_some_and(|elem| matches!(elem.kind(), Syn::Argument | Syn::Error))
		{
			return None;
		}

		Some(ret)
	}

	/// Shorthand for `self.dot3.is_some()`.
	#[must_use]
	pub fn acceding(&self) -> bool {
		self.dot3().is_some()
	}

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

// BlockLabel //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::BlockLabel`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct BlockLabel(SyntaxNode);

simple_astnode!(Syn, BlockLabel, Syn::BlockLabel);

impl BlockLabel {
	/// The returned token is always tagged [`Syn::Ident`].
	pub fn ident(&self) -> AstResult<SyntaxToken> {
		let Some(opener) = self.0.first_token() else {
			return Err(AstError::Incorrect);
		};

		let Some(closer) = self.0.last_token() else {
			return Err(AstError::Incorrect);
		};

		if opener.kind() != Syn::Colon2 || closer.kind() != Syn::Colon2 {
			return Err(AstError::Incorrect);
		}

		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|t| t.kind() == Syn::Ident))
			.ok_or(AstError::Missing)
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
		if node.kind() != Syn::Import {
			return None;
		}

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

	pub fn annotations(&self) -> impl Iterator<Item = Annotation> {
		self.syntax().children().filter_map(Annotation::cast)
	}

	pub fn docs(&self) -> impl Iterator<Item = DocComment> {
		doc_comments(self.syntax())
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
pub enum TypeSpec {
	AnyT(SyntaxNode),
	TypeT(SyntaxNode),
	Expr(SyntaxNode),
}

impl AstNode for TypeSpec {
	type Language = Syn;

	fn can_cast(kind: Syn) -> bool
	where
		Self: Sized,
	{
		kind == Syn::TypeSpec
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		if !Self::can_cast(node.kind()) {
			return None;
		}

		let token_x = node.last_token().unwrap();

		match token_x.kind() {
			Syn::KwAnyT => return Some(Self::AnyT(node)),
			Syn::KwTypeT => return Some(Self::TypeT(node)),
			_ => {}
		}

		(node.last_child().is_some_and(|n| Expr::can_cast(n.kind()))).then(|| Self::Expr(node))
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::AnyT(node) | Self::TypeT(node) | Self::Expr(node) => node,
		}
	}
}

impl TypeSpec {
	#[must_use]
	pub fn into_expr(&self) -> Option<Expr> {
		match self {
			Self::Expr(node) => node.first_child().and_then(Expr::cast),
			_ => None,
		}
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
