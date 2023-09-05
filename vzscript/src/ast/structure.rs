//! Structural items; classes, mixins, structs, unions.

use doomfront::simple_astnode;

use crate::{Syn, SyntaxNode};

/// Wraps a node tagged [`Syn::ClassDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClassDef(pub(super) SyntaxNode);

simple_astnode!(Syn, ClassDef, Syn::ClassDef);

// Struct /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::StructDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructDef(pub(super) SyntaxNode);

simple_astnode!(Syn, StructDef, Syn::StructDef);

// Union //////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::UnionDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnionDef(pub(super) SyntaxNode);

simple_astnode!(Syn, UnionDef, Syn::UnionDef);
