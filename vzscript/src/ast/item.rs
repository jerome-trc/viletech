//! Non-structural items; enums, functions, type aliases, symbolic constants.

use doomfront::{rowan::ast::AstNode, simple_astnode, AstError, AstResult};

use crate::{Syn, SyntaxNode, SyntaxToken};

// ConstDef ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ConstDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConstDef(pub(super) SyntaxNode);

simple_astnode!(Syn, ConstDef, Syn::ConstDef);

// EnumDef ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::EnumDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EnumDef(pub(super) SyntaxNode);

simple_astnode!(Syn, EnumDef, Syn::EnumDef);

/// Wraps a node tagged [`Syn::EnumVariant`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EnumVariant(SyntaxNode);

simple_astnode!(Syn, EnumVariant, Syn::EnumVariant);

// FuncDecl ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::FuncDecl`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FuncDecl(pub(super) SyntaxNode);

simple_astnode!(Syn, FuncDecl, Syn::FuncDecl);

impl FuncDecl {
	/// The returned token is always tagged [`Syn::Ident`].
	pub fn name(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.ok_or(AstError::Missing)
	}

	pub fn params(&self) -> AstResult<ParamList> {
		self.0
			.children()
			.find_map(ParamList::cast)
			.ok_or(AstError::Missing)
	}
}

/// Wraps a node tagged [`Syn::ParamList`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ParamList(pub(super) SyntaxNode);

simple_astnode!(Syn, ParamList, Syn::ParamList);
