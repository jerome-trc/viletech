//! AST nodes for representing expressions.

use doomfront::{
	rowan::{self, ast::AstNode},
	simple_astnode,
};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
	Binary(ExprBinary),
	Name(Name),
	Literal(Literal),
	Prefix(ExprPrefix),
	Postfix(ExprPostfix),
	Ternary(ExprTernary),
	Type(ExprType),
}

impl AstNode for Expression {
	type Language = Syn;

	fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syn::ExprBinary
				| Syn::Name | Syn::Literal
				| Syn::ExprPostfix
				| Syn::ExprPrefix
				| Syn::ExprTernary
		)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::ExprBinary => Some(Self::Binary(ExprBinary(node))),
			Syn::Name => Some(Self::Name(Name(node))),
			Syn::Literal => Some(Self::Literal(Literal(node))),
			Syn::ExprPostfix => Some(Self::Postfix(ExprPostfix(node))),
			Syn::ExprPrefix => Some(Self::Prefix(ExprPrefix(node))),
			Syn::ExprTernary => Some(Self::Ternary(ExprTernary(node))),
			Syn::ExprType => Some(Self::Type(ExprType(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Binary(inner) => &inner.0,
			Self::Name(inner) => &inner.0,
			Self::Literal(inner) => &inner.0,
			Self::Prefix(inner) => &inner.0,
			Self::Postfix(inner) => &inner.0,
			Self::Ternary(inner) => &inner.0,
			Self::Type(inner) => &inner.0,
		}
	}
}

impl Expression {
	#[must_use]
	pub fn into_literal(self) -> Option<Literal> {
		match self {
			Self::Literal(lit) => Some(lit),
			_ => None,
		}
	}
}

/// Wrpas a [`Syn::ExprBinary`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct ExprBinary(SyntaxNode);

simple_astnode!(Syn, ExprBinary, Syn::ExprBinary);

/// Wraps a [`Syn::ExprPostfix`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct ExprPostfix(SyntaxNode);

simple_astnode!(Syn, ExprPostfix, Syn::ExprPostfix);

/// Wraps a [`Syn::ExprPrefix`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct ExprPrefix(SyntaxNode);

simple_astnode!(Syn, ExprPrefix, Syn::ExprPrefix);

/// Wraps a [`Syn::ExprTernary`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct ExprTernary(SyntaxNode);

simple_astnode!(Syn, ExprTernary, Syn::ExprTernary);

/// Wraps a [`Syn::ExprType`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct ExprType(SyntaxNode);

simple_astnode!(Syn, ExprType, Syn::ExprType);

impl ExprType {}
