//! AST nodes for representing expressions.

use doomfront::{
	rowan::{ast::AstNode, Language},
	simple_astnode, AstError, AstResult,
};

use crate::{Syn, SyntaxNode, SyntaxToken};

use super::LitToken;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
	Array(ArrayExpr),
	Binary(BinExpr),
	Block(BlockExpr),
	Call(CallExpr),
	Class(ClassExpr),
	Construct(ConstructExpr),
	Enum(EnumExpr),
	Field(FieldExpr),
	For(ForExpr),
	GroupExpr(GroupExpr),
	Function(FunctionExpr),
	Ident(IdentExpr),
	Index(IndexExpr),
	Literal(Literal),
	Prefix(PrefixExpr),
	Struct(StructExpr),
	Switch(SwitchExpr),
	Union(UnionExpr),
	Variant(VariantExpr),
}

impl AstNode for Expr {
	type Language = Syn;

	fn can_cast(kind: <Self::Language as Language>::Kind) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syn::ArrayExpr
				| Syn::BinExpr | Syn::BlockExpr
				| Syn::CallExpr | Syn::ClassExpr
				| Syn::ConstructExpr
				| Syn::EnumExpr | Syn::FieldExpr
				| Syn::ForExpr | Syn::GroupExpr
				| Syn::FunctionExpr
				| Syn::IdentExpr | Syn::IndexExpr
				| Syn::Literal | Syn::PrefixExpr
				| Syn::StructExpr
				| Syn::SwitchExpr
				| Syn::UnionExpr | Syn::VariantExpr
		)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::ArrayExpr => Some(Self::Array(ArrayExpr(node))),
			Syn::BinExpr => Some(Self::Binary(BinExpr(node))),
			Syn::BlockExpr => Some(Self::Block(BlockExpr(node))),
			Syn::CallExpr => Some(Self::Call(CallExpr(node))),
			Syn::ClassExpr => Some(Self::Class(ClassExpr(node))),
			Syn::ConstructExpr => Some(Self::Construct(ConstructExpr(node))),
			Syn::EnumExpr => Some(Self::Enum(EnumExpr(node))),
			Syn::FieldExpr => Some(Self::Field(FieldExpr(node))),
			Syn::ForExpr => Some(Self::For(ForExpr(node))),
			Syn::GroupExpr => Some(Self::GroupExpr(GroupExpr(node))),
			Syn::FunctionExpr => Some(Self::Function(FunctionExpr(node))),
			Syn::IdentExpr => Some(Self::Ident(IdentExpr(node))),
			Syn::IndexExpr => Some(Self::Index(IndexExpr(node))),
			Syn::Literal => Some(Self::Literal(Literal(node))),
			Syn::PrefixExpr => Some(Self::Prefix(PrefixExpr(node))),
			Syn::StructExpr => Some(Self::Struct(StructExpr(node))),
			Syn::SwitchExpr => Some(Self::Switch(SwitchExpr(node))),
			Syn::UnionExpr => Some(Self::Union(UnionExpr(node))),
			Syn::VariantExpr => Some(Self::Variant(VariantExpr(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Expr::Array(inner) => inner.syntax(),
			Expr::Binary(inner) => inner.syntax(),
			Expr::Block(inner) => inner.syntax(),
			Expr::Call(inner) => inner.syntax(),
			Expr::Class(inner) => inner.syntax(),
			Expr::Construct(inner) => inner.syntax(),
			Expr::Enum(inner) => inner.syntax(),
			Expr::Field(inner) => inner.syntax(),
			Expr::For(inner) => inner.syntax(),
			Expr::GroupExpr(inner) => inner.syntax(),
			Expr::Function(inner) => inner.syntax(),
			Expr::Ident(inner) => inner.syntax(),
			Expr::Index(inner) => inner.syntax(),
			Expr::Literal(inner) => inner.syntax(),
			Expr::Prefix(inner) => inner.syntax(),
			Expr::Struct(inner) => inner.syntax(),
			Expr::Switch(inner) => inner.syntax(),
			Expr::Union(inner) => inner.syntax(),
			Expr::Variant(inner) => inner.syntax(),
		}
	}
}

// Array ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ArrayExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayExpr(SyntaxNode);

simple_astnode!(Syn, ArrayExpr, Syn::ArrayExpr);

// Binary //////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::BinExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BinExpr(SyntaxNode);

simple_astnode!(Syn, BinExpr, Syn::BinExpr);

impl BinExpr {
	#[must_use]
	pub fn left(&self) -> Expr {
		Expr::cast(self.0.first_child().unwrap()).unwrap()
	}

	pub fn right(&self) -> AstResult<Expr> {
		Expr::cast(self.0.children().nth(1).ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}
}

// Block ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::BlockExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockExpr(SyntaxNode);

simple_astnode!(Syn, BlockExpr, Syn::BlockExpr);

// Call ////////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::CallExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CallExpr(SyntaxNode);

simple_astnode!(Syn, CallExpr, Syn::CallExpr);

impl CallExpr {}

// Class ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ClassExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClassExpr(SyntaxNode);

simple_astnode!(Syn, ClassExpr, Syn::ClassExpr);

// Construct ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ConstructExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConstructExpr(SyntaxNode);

simple_astnode!(Syn, ConstructExpr, Syn::ConstructExpr);

// Enum ////////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::EnumExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumExpr(SyntaxNode);

simple_astnode!(Syn, EnumExpr, Syn::EnumExpr);

// Field ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::FieldExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldExpr(SyntaxNode);

simple_astnode!(Syn, FieldExpr, Syn::FieldExpr);

impl FieldExpr {
	#[must_use]
	pub fn lhs(&self) -> Expr {
		Expr::cast(self.0.first_child().unwrap()).unwrap()
	}

	pub fn field_name(&self) -> AstResult<SyntaxToken> {
		self.0.last_token().ok_or(AstError::Missing)
	}
}

// For /////////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ForExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ForExpr(SyntaxNode);

simple_astnode!(Syn, ForExpr, Syn::ForExpr);

// Function ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::FunctionExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionExpr(SyntaxNode);

simple_astnode!(Syn, FunctionExpr, Syn::FunctionExpr);

// Group ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::GroupExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GroupExpr(SyntaxNode);

simple_astnode!(Syn, GroupExpr, Syn::GroupExpr);

impl GroupExpr {
	pub fn inner(&self) -> AstResult<Expr> {
		Expr::cast(self.0.first_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}
}

// Ident. //////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::IdentExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdentExpr(SyntaxNode);

simple_astnode!(Syn, IdentExpr, Syn::IdentExpr);

impl IdentExpr {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn token(&self) -> SyntaxToken {
		let ret = self.0.first_token().unwrap();
		debug_assert_eq!(ret.kind(), Syn::Ident);
		ret
	}
}

// Index ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::IndexExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndexExpr(SyntaxNode);

simple_astnode!(Syn, IndexExpr, Syn::IndexExpr);

impl IndexExpr {
	#[must_use]
	pub fn indexed(&self) -> Expr {
		Expr::cast(self.0.first_child().unwrap()).unwrap()
	}

	pub fn index(&self) -> AstResult<Expr> {
		match self.0.children().nth(1) {
			Some(node) => Expr::cast(node).ok_or(AstError::Incorrect),
			None => Err(AstError::Missing),
		}
	}
}

// Literal /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::Literal`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Literal(SyntaxNode);

simple_astnode!(Syn, Literal, Syn::Literal);

impl Literal {
	#[must_use]
	pub fn token(&self) -> LitToken {
		LitToken(self.0.first_token().unwrap())
	}
}

// Prefix //////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::PrefixExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrefixExpr(SyntaxNode);

simple_astnode!(Syn, PrefixExpr, Syn::PrefixExpr);

impl PrefixExpr {
	pub fn operand(&self) -> AstResult<Expr> {
		Expr::cast(self.0.last_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}

	#[must_use]
	pub fn operator(&self) -> (SyntaxToken, PrefixOp) {
		let ret0 = self.0.first_token().unwrap();

		let ret1 = match ret0.kind() {
			Syn::Bang => PrefixOp::Bang,
			Syn::Minus => PrefixOp::Minus,
			Syn::Tilde => PrefixOp::Tilde,
			_ => unreachable!(),
		};

		(ret0, ret1)
	}
}

/// See [`PrefixExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrefixOp {
	Bang,
	Minus,
	Tilde,
}

// Struct //////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::StructExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructExpr(SyntaxNode);

simple_astnode!(Syn, StructExpr, Syn::StructExpr);

// Switch //////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::SwitchExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SwitchExpr(SyntaxNode);

simple_astnode!(Syn, SwitchExpr, Syn::SwitchExpr);

// Union ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::UnionExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnionExpr(SyntaxNode);

simple_astnode!(Syn, UnionExpr, Syn::UnionExpr);

// Variant /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::VariantExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VariantExpr(SyntaxNode);

simple_astnode!(Syn, VariantExpr, Syn::VariantExpr);
