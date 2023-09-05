//! Structures for representing a VZScript abstract syntax tree.

mod expr;
mod item;
mod lit;
mod stat;
mod structure;

use doomfront::rowan::{self, ast::AstNode};

use doomfront::{simple_astnode, AstError, AstResult};

use super::{Syn, SyntaxNode, SyntaxToken};

pub use self::{expr::*, item::*, lit::*, stat::*, structure::*};

/// A top-level element in a source file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TopLevel {
	Annotation(Annotation),
	ClassDef(ClassDef),
	ConstDef(ConstDef),
	EnumDef(EnumDef),
	FuncDecl(FuncDecl),
	StructDef(StructDef),
	UnionDef(UnionDef),
}

impl AstNode for TopLevel {
	type Language = Syn;

	fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syn::Annotation
				| Syn::ClassDef | Syn::ConstDef
				| Syn::EnumDef | Syn::FuncDecl
				| Syn::StructDef | Syn::UnionDef
		)
	}

	fn cast(node: rowan::SyntaxNode<Self::Language>) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::Annotation => Some(Self::Annotation(Annotation(node))),
			Syn::ClassDef => Some(Self::ClassDef(ClassDef(node))),
			Syn::ConstDef => Some(Self::ConstDef(ConstDef(node))),
			Syn::EnumDef => Some(Self::EnumDef(EnumDef(node))),
			Syn::FuncDecl => Some(Self::FuncDecl(FuncDecl(node))),
			Syn::StructDef => Some(Self::StructDef(StructDef(node))),
			Syn::UnionDef => Some(Self::UnionDef(UnionDef(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &rowan::SyntaxNode<Self::Language> {
		match self {
			TopLevel::Annotation(inner) => inner.syntax(),
			TopLevel::ClassDef(inner) => inner.syntax(),
			TopLevel::ConstDef(inner) => inner.syntax(),
			TopLevel::EnumDef(inner) => inner.syntax(),
			TopLevel::FuncDecl(inner) => inner.syntax(),
			TopLevel::StructDef(inner) => inner.syntax(),
			TopLevel::UnionDef(inner) => inner.syntax(),
		}
	}
}

// Annotation //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::Annotation`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Annotation(SyntaxNode);

simple_astnode!(Syn, Annotation, Syn::Annotation);

impl Annotation {
	/// The returned token is always tagged [`Syn::Ident`].
	pub fn name(&self) -> AstResult<SyntaxToken> {
		self.0
			.first_token()
			.unwrap()
			.next_token()
			.ok_or(AstError::Missing)
	}
}

// Argument and ArgList ////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ArgList`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArgList(SyntaxNode);

simple_astnode!(Syn, ArgList, Syn::ArgList);

impl ArgList {
	pub fn iter(&self) -> impl Iterator<Item = Argument> {
		self.0.children().filter_map(Argument::cast)
	}
}

/// Wraps a node tagged [`Syn::Argument`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
				Syn::Ident | Syn::NameLit => {
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

// Attribute ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::Attribute`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Attribute(SyntaxNode);

simple_astnode!(Syn, Attribute, Syn::Attribute);

impl Attribute {
	/// The returned token is always tagged [`Syn::Ident`].
	pub fn name(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.ok_or(AstError::Missing)
	}

	#[must_use]
	pub fn args(&self) -> Option<ArgList> {
		match self.0.last_child() {
			Some(node) => ArgList::cast(node),
			None => None,
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

// Name, NameChain /////////////////////////////////////////////////////////////

/// Convenience structure wrapping a [`SyntaxToken`] tagged with either
/// [`Syn::Ident`] or [`Syn::NameLit`], for positions where either is valid.
pub struct Name(SyntaxToken);

impl Name {
	#[must_use]
	pub fn inner_text(&self) -> &str {
		match self.0.kind() {
			Syn::Ident => self.0.text(),
			Syn::NameLit => &self.0.text()[1..(self.0.text().len() - 1)],
			_ => unreachable!(),
		}
	}
}

impl std::ops::Deref for Name {
	type Target = SyntaxToken;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

/// Wraps a node tagged [`Syn::NameChain`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NameChain(SyntaxNode);

simple_astnode!(Syn, NameChain, Syn::NameChain);

impl NameChain {
	pub fn parts(&self) -> impl Iterator<Item = Name> {
		self.0.children_with_tokens().filter_map(|elem| {
			if matches!(elem.kind(), Syn::Ident | Syn::NameLit) {
				Some(Name(elem.into_token().unwrap()))
			} else {
				None
			}
		})
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

pub(self) fn doc_comments(node: &SyntaxNode) -> impl Iterator<Item = DocComment> {
	node.children_with_tokens()
		.take_while(|elem| elem.kind().is_trivia() || elem.kind() == Syn::DocComment)
		.filter_map(|elem| {
			elem.into_token()
				.filter(|token| token.kind() == Syn::DocComment)
				.map(DocComment)
		})
}
