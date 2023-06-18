//! AST nodes for representing classes, mixin classes, and structs.

use crate::simple_astnode;

use super::{Syn, SyntaxNode, SyntaxToken};

// ClassDef ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ClassDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ClassDef(pub(super) SyntaxNode);

simple_astnode!(Syn, ClassDef, Syn::ClassDef);

impl ClassDef {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.unwrap()
	}

	/// The returned token is always tagged [`Syn::Ident`].
	pub fn parent_class(&self) -> Option<SyntaxToken> {
		let spec = self
			.0
			.children()
			.find_map(|node| Some(node).filter(|node| node.kind() == Syn::InheritSpec));

		let Some(spec) = spec else { return None; };
		let ret = spec.last_token().unwrap();
		debug_assert_eq!(ret.kind(), Syn::Ident);
		Some(ret)
	}
}

// ClassExtend /////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ClassExtend`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ClassExtend(pub(super) SyntaxNode);

simple_astnode!(Syn, ClassExtend, Syn::ClassExtend);

impl ClassExtend {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.unwrap()
	}
}

// MixinClassDef ///////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::MixinClassDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MixinClassDef(pub(super) SyntaxNode);

simple_astnode!(Syn, MixinClassDef, Syn::MixinClassDef);

impl MixinClassDef {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.unwrap()
	}
}

// StructDef ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::StructDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StructDef(pub(super) SyntaxNode);

simple_astnode!(Syn, StructDef, Syn::StructDef);

impl StructDef {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.unwrap()
	}
}

// StructExtend ////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::StructExtend`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StructExtend(pub(super) SyntaxNode);

simple_astnode!(Syn, StructExtend, Syn::StructExtend);

impl StructExtend {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.unwrap()
	}
}

// FieldDecl ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::FieldDecl`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FieldDecl(SyntaxNode);

simple_astnode!(Syn, FieldDecl, Syn::FieldDecl);

// FunctionDecl ////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::FunctionDecl`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FunctionDecl(pub(super) SyntaxNode);

simple_astnode!(Syn, FunctionDecl, Syn::FunctionDecl);

impl FunctionDecl {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.unwrap()
	}
}
