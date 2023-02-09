//! AST nodes for representing expressions.

use doomfront::{
	rowan::{self, ast::AstNode, SyntaxNode},
	simple_astnode,
};

use super::Syn;

/// Wraps a [`Syn::ExprType`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct ExprType(SyntaxNode<Syn>);

simple_astnode!(Syn, ExprType, Syn::ExprType);
