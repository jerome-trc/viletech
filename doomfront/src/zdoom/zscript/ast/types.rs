//! AST nodes for representing types.

use rowan::ast::AstNode;

use crate::simple_astnode;

use super::{ArrayLen, IdentChain, Syn, SyntaxNode, SyntaxToken};

/// Wraps a node tagged [`Syn::TypeRef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TypeRef(pub(super) SyntaxNode);

simple_astnode!(Syn, TypeRef, Syn::TypeRef);

impl TypeRef {
	#[must_use]
	pub fn core(&self) -> CoreType {
		CoreType::cast(self.0.first_child().unwrap()).unwrap()
	}

	#[must_use]
	pub fn array_len(&self) -> Option<ArrayLen> {
		self.0
			.last_child()
			.filter(|node| node.kind() == Syn::ArrayLen)
			.map(ArrayLen)
	}
}

// CoreType ////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum CoreType {
	Class(ClassType),
	DynArray(DynArrayType),
	IdentChain(IdentChainType),
	Let(LetType),
	Map(MapType),
	MapIter(MapIterType),
	Native(NativeType),
	Readonly(ReadOnlyType),
}

impl AstNode for CoreType {
	type Language = Syn;

	fn can_cast(kind: Syn) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syn::ClassType
				| Syn::DynArrayType
				| Syn::IdentChainType
				| Syn::LetType | Syn::MapType
				| Syn::MapIterType
				| Syn::NativeType
				| Syn::ReadOnlyType
		)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::ClassType => Some(Self::Class(ClassType(node))),
			Syn::DynArrayType => Some(Self::DynArray(DynArrayType(node))),
			Syn::IdentChainType => Some(Self::IdentChain(IdentChainType(node))),
			Syn::LetType => Some(Self::Let(LetType(node))),
			Syn::MapType => Some(Self::Map(MapType(node))),
			Syn::MapIterType => Some(Self::MapIter(MapIterType(node))),
			Syn::NativeType => Some(Self::Native(NativeType(node))),
			Syn::ReadOnlyType => Some(Self::Readonly(ReadOnlyType(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Class(inner) => inner.syntax(),
			Self::DynArray(inner) => inner.syntax(),
			Self::IdentChain(inner) => inner.syntax(),
			Self::Let(inner) => inner.syntax(),
			Self::Map(inner) => inner.syntax(),
			Self::MapIter(inner) => inner.syntax(),
			Self::Native(inner) => inner.syntax(),
			Self::Readonly(inner) => inner.syntax(),
		}
	}
}

// ClassType ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ClassType`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ClassType(SyntaxNode);

simple_astnode!(Syn, ClassType, Syn::ClassType);

impl ClassType {
	#[must_use]
	pub fn restrictor(&self) -> Option<IdentChain> {
		self.0.first_child().map(IdentChain)
	}
}

// DynArrayType ////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::DynArrayType`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DynArrayType(SyntaxNode);

simple_astnode!(Syn, DynArrayType, Syn::DynArrayType);

impl DynArrayType {
	#[must_use]
	pub fn element_type(&self) -> TypeRef {
		let ret = self.0.last_child().unwrap();
		debug_assert_eq!(ret.kind(), Syn::TypeRef);
		TypeRef(ret)
	}
}

// IdentChainType //////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::IdentChainType`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct IdentChainType(SyntaxNode);

simple_astnode!(Syn, IdentChainType, Syn::IdentChainType);

impl IdentChainType {
	#[must_use]
	pub fn inner(&self) -> IdentChain {
		let ret = self.0.first_child().unwrap();
		debug_assert_eq!(ret.kind(), Syn::IdentChain);
		IdentChain(ret)
	}
}

// LetType /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::LetType`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LetType(SyntaxNode);

simple_astnode!(Syn, LetType, Syn::LetType);

// MapType /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::MapType`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MapType(SyntaxNode);

simple_astnode!(Syn, MapType, Syn::MapType);

impl MapType {
	#[must_use]
	pub fn key_type(&self) -> TypeRef {
		let ret = self.0.first_child().unwrap();
		debug_assert_eq!(ret.kind(), Syn::TypeRef);
		TypeRef(ret)
	}

	#[must_use]
	pub fn value_type(&self) -> TypeRef {
		let ret = self.0.last_child().unwrap();
		debug_assert_eq!(ret.kind(), Syn::TypeRef);
		TypeRef(ret)
	}
}

// MapIterType /////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::MapIterType`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MapIterType(SyntaxNode);

simple_astnode!(Syn, MapIterType, Syn::MapIterType);

impl MapIterType {
	#[must_use]
	pub fn key_type(&self) -> TypeRef {
		let ret = self.0.first_child().unwrap();
		debug_assert_eq!(ret.kind(), Syn::TypeRef);
		TypeRef(ret)
	}

	#[must_use]
	pub fn value_type(&self) -> TypeRef {
		let ret = self.0.last_child().unwrap();
		debug_assert_eq!(ret.kind(), Syn::TypeRef);
		TypeRef(ret)
	}
}

// NativeType //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::NativeType`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NativeType(SyntaxNode);

simple_astnode!(Syn, NativeType, Syn::NativeType);

impl NativeType {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn ident(&self) -> SyntaxToken {
		let ret = self.0.last_token().unwrap();
		debug_assert_eq!(ret.kind(), Syn::Ident);
		ret
	}
}

// PrimitiveType ///////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PrimitiveType(SyntaxNode);

simple_astnode!(Syn, PrimitiveType, Syn::PrimitiveType);

impl PrimitiveType {
	/// See [`Syn::PrimitiveType`] for the range of possible token tags.
	pub fn token(&self) -> SyntaxToken {
		self.0.first_token().unwrap()
	}
}

// ReadonlyType ////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ReadonlyType`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ReadOnlyType(SyntaxNode);

simple_astnode!(Syn, ReadOnlyType, Syn::ReadOnlyType);

impl ReadOnlyType {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn ident(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.unwrap()
	}

	/// i.e. if the inner identifier is preceded with a [`Syn::At`].
	#[must_use]
	pub fn is_native(&self) -> bool {
		self.0.children_with_tokens().any(|elem| {
			elem.into_token()
				.is_some_and(|token| token.kind() == Syn::At)
		})
	}
}
