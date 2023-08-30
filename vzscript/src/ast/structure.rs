//! Structural items; classes, mixins, structs, unions.

use doomfront::{simple_astnode, AstError, AstResult};

use crate::{Syn, SyntaxNode, SyntaxToken};

/// Wraps a node tagged [`Syn::ClassDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ClassDef(pub(super) SyntaxNode);

simple_astnode!(Syn, ClassDef, Syn::ClassDef);

/// Wraps a node tagged [`Syn::ClassExtend`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ClassExtend(pub(super) SyntaxNode);

simple_astnode!(Syn, ClassExtend, Syn::ClassExtend);

// Struct /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::StructDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StructDef(pub(super) SyntaxNode);

simple_astnode!(Syn, StructDef, Syn::StructDef);

/// Wraps a node tagged [`Syn::StructExtend`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StructExtend(pub(super) SyntaxNode);

simple_astnode!(Syn, StructExtend, Syn::StructExtend);

// Mixin //////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::MixinDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MixinDef(pub(super) SyntaxNode);

simple_astnode!(Syn, MixinDef, Syn::MixinDef);

impl MixinDef {
	/// The returned token is always tagged [`Syn::Ident`].
	pub fn name(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.ok_or(AstError::Missing)
	}
}

// Union //////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::UnionDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct UnionDef(pub(super) SyntaxNode);

simple_astnode!(Syn, UnionDef, Syn::UnionDef);
