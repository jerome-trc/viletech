//! Structures for representing a VZScript abstract syntax tree.

mod expr;
mod item;
mod lit;
mod stat;
mod structure;

use doomfront::rowan::{self, ast::AstNode};

use doomfront::{simple_astnode, AstError, AstResult};

use super::{Syn, SyntaxNode, SyntaxToken};

pub use self::{expr::*, item::*, structure::*};

/// A top-level element in a source file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum TopLevel {
	Annotation(Annotation),
	ClassDef(ClassDef),
	ClassExtend(ClassExtend),
	ConstDef(ConstDef),
	EnumDef(EnumDef),
	FuncDecl(FuncDecl),
	MixinDef(MixinDef),
	StructDef(StructDef),
	StructExtend(StructExtend),
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
				| Syn::ClassDef | Syn::ClassExtend
				| Syn::ConstDef | Syn::EnumDef
				| Syn::FuncDecl | Syn::MixinDef
				| Syn::StructDef | Syn::StructExtend
				| Syn::UnionDef
		)
	}

	fn cast(node: rowan::SyntaxNode<Self::Language>) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::Annotation => Some(Self::Annotation(Annotation(node))),
			Syn::ClassDef => Some(Self::ClassDef(ClassDef(node))),
			Syn::ClassExtend => Some(Self::ClassExtend(ClassExtend(node))),
			Syn::ConstDef => Some(Self::ConstDef(ConstDef(node))),
			Syn::EnumDef => Some(Self::EnumDef(EnumDef(node))),
			Syn::FuncDecl => Some(Self::FuncDecl(FuncDecl(node))),
			Syn::MixinDef => Some(Self::MixinDef(MixinDef(node))),
			Syn::StructDef => Some(Self::StructDef(StructDef(node))),
			Syn::StructExtend => Some(Self::StructExtend(StructExtend(node))),
			Syn::UnionDef => Some(Self::UnionDef(UnionDef(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &rowan::SyntaxNode<Self::Language> {
		match self {
			TopLevel::Annotation(inner) => inner.syntax(),
			TopLevel::ClassDef(inner) => inner.syntax(),
			TopLevel::ClassExtend(inner) => inner.syntax(),
			TopLevel::ConstDef(inner) => inner.syntax(),
			TopLevel::EnumDef(inner) => inner.syntax(),
			TopLevel::FuncDecl(inner) => inner.syntax(),
			TopLevel::MixinDef(inner) => inner.syntax(),
			TopLevel::StructDef(inner) => inner.syntax(),
			TopLevel::StructExtend(inner) => inner.syntax(),
			TopLevel::UnionDef(inner) => inner.syntax(),
		}
	}
}

/// Wraps a node tagged [`Syn::Annotation`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
