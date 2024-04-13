//! AST nodes for representing types.

use rowan::ast::AstNode;

use crate::{simple_astnode, AstError, AstResult};

use super::{ArrayLen, IdentChain, Syntax, SyntaxNode, SyntaxToken};

/// Wraps a node tagged [`Syntax::TypeRef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TypeRef(pub(super) SyntaxNode);

simple_astnode!(Syntax, TypeRef, Syntax::TypeRef);

impl TypeRef {
	#[must_use]
	pub fn core(&self) -> CoreType {
		CoreType::cast(self.0.first_child().unwrap()).unwrap()
	}

	pub fn array_lengths(&self) -> impl Iterator<Item = ArrayLen> {
		self.0.children().filter_map(ArrayLen::cast)
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
	Primitive(PrimitiveType),
	Readonly(ReadOnlyType),
}

impl AstNode for CoreType {
	type Language = Syntax;

	fn can_cast(kind: Syntax) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syntax::ClassType
				| Syntax::DynArrayType
				| Syntax::IdentChainType
				| Syntax::LetType
				| Syntax::MapType
				| Syntax::MapIterType
				| Syntax::NativeType
				| Syntax::PrimitiveType
				| Syntax::ReadOnlyType
		)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syntax::ClassType => Some(Self::Class(ClassType(node))),
			Syntax::DynArrayType => Some(Self::DynArray(DynArrayType(node))),
			Syntax::IdentChainType => Some(Self::IdentChain(IdentChainType(node))),
			Syntax::LetType => Some(Self::Let(LetType(node))),
			Syntax::MapType => Some(Self::Map(MapType(node))),
			Syntax::MapIterType => Some(Self::MapIter(MapIterType(node))),
			Syntax::NativeType => Some(Self::Native(NativeType(node))),
			Syntax::PrimitiveType => Some(Self::Primitive(PrimitiveType(node))),
			Syntax::ReadOnlyType => Some(Self::Readonly(ReadOnlyType(node))),
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
			Self::Primitive(inner) => inner.syntax(),
			Self::Readonly(inner) => inner.syntax(),
		}
	}
}

// ClassType ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ClassType`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ClassType(SyntaxNode);

simple_astnode!(Syntax, ClassType, Syntax::ClassType);

impl ClassType {
	#[must_use]
	pub fn restrictor(&self) -> Option<IdentChain> {
		self.0.first_child().map(IdentChain)
	}
}

// DynArrayType ////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::DynArrayType`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DynArrayType(SyntaxNode);

simple_astnode!(Syntax, DynArrayType, Syntax::DynArrayType);

impl DynArrayType {
	pub fn element_type(&self) -> AstResult<TypeRef> {
		let Some(node) = self.0.last_child() else {
			return Err(AstError::Missing);
		};
		TypeRef::cast(node).ok_or(AstError::Incorrect)
	}
}

// IdentChainType //////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::IdentChainType`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct IdentChainType(SyntaxNode);

simple_astnode!(Syntax, IdentChainType, Syntax::IdentChainType);

impl IdentChainType {
	#[must_use]
	pub fn inner(&self) -> IdentChain {
		let ret = self.0.first_child().unwrap();
		debug_assert_eq!(ret.kind(), Syntax::IdentChain);
		IdentChain(ret)
	}
}

// LetType /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::LetType`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LetType(SyntaxNode);

simple_astnode!(Syntax, LetType, Syntax::LetType);

impl LetType {
	/// This is the only content of one of these nodes;
	/// the returned token is always tagged [`Syntax::KwLet`].
	#[must_use]
	pub fn keyword(&self) -> SyntaxToken {
		let ret = self.0.first_token().unwrap();
		debug_assert_eq!(ret.kind(), Syntax::KwLet);
		ret
	}
}

// MapType /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::MapType`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MapType(SyntaxNode);

simple_astnode!(Syntax, MapType, Syntax::MapType);

impl MapType {
	pub fn key_type(&self) -> AstResult<TypeRef> {
		let Some(node) = self.0.first_child() else {
			return Err(AstError::Missing);
		};
		TypeRef::cast(node).ok_or(AstError::Incorrect)
	}

	pub fn value_type(&self) -> AstResult<TypeRef> {
		let Some(node) = self.0.last_child() else {
			return Err(AstError::Missing);
		};
		TypeRef::cast(node).ok_or(AstError::Incorrect)
	}
}

// MapIterType /////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::MapIterType`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MapIterType(SyntaxNode);

simple_astnode!(Syntax, MapIterType, Syntax::MapIterType);

impl MapIterType {
	pub fn key_type(&self) -> AstResult<TypeRef> {
		let Some(node) = self.0.first_child() else {
			return Err(AstError::Missing);
		};
		TypeRef::cast(node).ok_or(AstError::Incorrect)
	}

	pub fn value_type(&self) -> AstResult<TypeRef> {
		let Some(node) = self.0.last_child() else {
			return Err(AstError::Missing);
		};
		TypeRef::cast(node).ok_or(AstError::Incorrect)
	}
}

// NativeType //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::NativeType`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NativeType(SyntaxNode);

simple_astnode!(Syntax, NativeType, Syntax::NativeType);

impl NativeType {
	/// The returned token is always tagged [`Syntax::Ident`].
	pub fn ident(&self) -> AstResult<SyntaxToken> {
		let ret = self.0.last_token().ok_or(AstError::Missing)?;

		match ret.kind() {
			Syntax::Ident => Ok(ret),
			_ => Err(AstError::Incorrect),
		}
	}
}

// PrimitiveType ///////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::PrimitiveType`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PrimitiveType(SyntaxNode);

simple_astnode!(Syntax, PrimitiveType, Syntax::PrimitiveType);

impl PrimitiveType {
	/// See [`Syntax::PrimitiveType`] for the range of possible token tags.
	pub fn token(&self) -> SyntaxToken {
		self.0.first_token().unwrap()
	}
}

// ReadonlyType ////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ReadOnlyType`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ReadOnlyType(SyntaxNode);

simple_astnode!(Syntax, ReadOnlyType, Syntax::ReadOnlyType);

impl ReadOnlyType {
	/// The returned token is always tagged [`Syntax::Ident`].
	pub fn ident(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::Ident)
			})
			.ok_or(AstError::Missing)
	}

	/// i.e. if the inner identifier is preceded with a [`Syntax::At`].
	#[must_use]
	pub fn is_native(&self) -> bool {
		self.0.children_with_tokens().any(|elem| {
			elem.into_token()
				.is_some_and(|token| token.kind() == Syntax::At)
		})
	}
}
