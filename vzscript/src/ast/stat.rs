//! AST nodes for representing statements.

use doomfront::{
	rowan::{ast::AstNode, Language},
	simple_astnode,
};

use super::{Syn, SyntaxNode};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Statement {
	Bind(BindStat),
	Break(BreakStat),
	Continue(ContinueStat),
	Expr(ExprStat),
	Return(ReturnStat),
}

impl AstNode for Statement {
	type Language = Syn;

	fn can_cast(kind: <Self::Language as Language>::Kind) -> bool
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
			Statement::Bind(inner) => inner.syntax(),
			Statement::Break(inner) => inner.syntax(),
			Statement::Continue(inner) => inner.syntax(),
			Statement::Expr(inner) => inner.syntax(),
			Statement::Return(inner) => inner.syntax(),
		}
	}
}

// Bind ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::BindStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BindStat(SyntaxNode);

simple_astnode!(Syn, BindStat, Syn::BindStat);

// Break //////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::BreakStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BreakStat(SyntaxNode);

simple_astnode!(Syn, BreakStat, Syn::BreakStat);

// Continue ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ContinueStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContinueStat(SyntaxNode);

simple_astnode!(Syn, ContinueStat, Syn::ContinueStat);

// Expr ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ExprStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExprStat(SyntaxNode);

simple_astnode!(Syn, ExprStat, Syn::ExprStat);

// Return /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ReturnStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReturnStat(SyntaxNode);

simple_astnode!(Syn, ReturnStat, Syn::ReturnStat);
