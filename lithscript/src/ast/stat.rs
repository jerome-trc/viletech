//! AST nodes for representing statements.

use doomfront::{rowan::ast::AstNode, simple_astnode};

use super::{Syn, SyntaxNode};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Statement {
	Bind(BindStat),
	Break(BreakStat),
	Continue(ContinueStat),
	Expr(ExprStat),
	Return(ReturnStat),
}

impl AstNode for Statement {
	type Language = Syn;

	fn can_cast(kind: Syn) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syn::BindStat | Syn::BreakStat | Syn::ContinueStat | Syn::ExprStat | Syn::ReturnStat
		)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::BindStat => Some(Self::Bind(BindStat(node))),
			Syn::BreakStat => Some(Self::Break(BreakStat(node))),
			Syn::ContinueStat => Some(Self::Continue(ContinueStat(node))),
			Syn::ExprStat => Some(Self::Expr(ExprStat(node))),
			Syn::ReturnStat => Some(Self::Return(ReturnStat(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Bind(inner) => inner.syntax(),
			Self::Break(inner) => inner.syntax(),
			Self::Continue(inner) => inner.syntax(),
			Self::Expr(inner) => inner.syntax(),
			Self::Return(inner) => inner.syntax(),
		}
	}
}

// BindStat ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::BindStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct BindStat(SyntaxNode);

simple_astnode!(Syn, BindStat, Syn::BindStat);

// BreakStat ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::BreakStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct BreakStat(SyntaxNode);

simple_astnode!(Syn, BreakStat, Syn::BreakStat);

// ContinueStat ////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ContinueStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ContinueStat(SyntaxNode);

simple_astnode!(Syn, ContinueStat, Syn::ContinueStat);

// ExprStat ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ExprStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprStat(SyntaxNode);

simple_astnode!(Syn, ExprStat, Syn::ExprStat);

// ReturnStat //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ReturnStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ReturnStat(SyntaxNode);

simple_astnode!(Syn, ReturnStat, Syn::ReturnStat);
