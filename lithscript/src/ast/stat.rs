//! AST nodes for representing statements.

use doomfront::{rowan::ast::AstNode, simple_astnode, AstError, AstResult};

use crate::{SyntaxElem, SyntaxToken};

use super::{Ident, Syn, SyntaxNode};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Statement {
	Break(BreakStat),
	Continue(ContinueStat),
	Expr(ExprStat),
	Import(ImportStat),
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
			Syn::BreakStat | Syn::ContinueStat | Syn::ExprStat | Syn::ImportStat | Syn::ReturnStat
		)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::BreakStat => Some(Self::Break(BreakStat(node))),
			Syn::ContinueStat => Some(Self::Continue(ContinueStat(node))),
			Syn::ExprStat => Some(Self::Expr(ExprStat(node))),
			Syn::ImportStat => Some(Self::Import(ImportStat(node))),
			Syn::ReturnStat => Some(Self::Return(ReturnStat(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Break(inner) => inner.syntax(),
			Self::Continue(inner) => inner.syntax(),
			Self::Expr(inner) => inner.syntax(),
			Self::Import(inner) => inner.syntax(),
			Self::Return(inner) => inner.syntax(),
		}
	}
}

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

// ImportStat //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ImportStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ImportStat(pub(super) SyntaxNode);

simple_astnode!(Syn, ImportStat, Syn::ImportStat);

impl ImportStat {
	/// The returned token is always tagged [`Syn::StringLit`].
	pub fn module(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syn::StringLit)
			})
			.ok_or(AstError::Missing)
	}

	#[must_use]
	pub fn list(&self) -> Option<ImportList> {
		self.0
			.last_child()
			.filter(|node| node.kind() == Syn::ImportList)
			.map(ImportList)
	}

	#[must_use]
	pub fn single(&self) -> Option<ImportEntry> {
		self.0
			.last_child()
			.filter(|node| node.kind() == Syn::ImportEntry)
			.map(ImportEntry)
	}

	/// If this is a `'*' => ident` import, return a [`Syn::Ident`] token
	/// for that trailing identifier.
	#[must_use]
	pub fn all_alias(&self) -> Option<Ident> {
		let Some(node) = self.0.last_child() else { return None; };
		let Syn::ImportEntry = node.kind() else { return None; };

		if node
			.first_token()
			.is_some_and(|token| token.kind() == Syn::Asterisk)
		{
			let ret = node.last_token().unwrap();
			debug_assert_eq!(ret.kind(), Syn::Ident);
			Some(Ident(ret))
		} else {
			None
		}
	}
}

/// Wraps a node tagged [`Syn::ImportList`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ImportList(SyntaxNode);

simple_astnode!(Syn, ImportList, Syn::ImportList);

impl ImportList {
	pub fn entries(&self) -> impl Iterator<Item = ImportEntry> {
		self.0.children().filter_map(ImportEntry::cast)
	}
}

/// Wraps a node tagged [`Syn::ImportEntry`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ImportEntry(SyntaxNode);

simple_astnode!(Syn, ImportEntry, Syn::ImportEntry);

impl ImportEntry {
	#[must_use]
	pub fn name(&self) -> Ident {
		let ret = self.0.first_child_or_token().unwrap().into_token().unwrap();
		debug_assert_eq!(ret.kind(), Syn::Ident);
		Ident(ret)
	}

	#[must_use]
	pub fn rename(&self) -> Option<Ident> {
		if !self.0.children_with_tokens().any(|elem| {
			elem.as_token()
				.is_some_and(|token| token.kind() == Syn::ThickArrow)
		}) {
			return None;
		}

		match self.0.last_child_or_token() {
			Some(SyntaxElem::Token(token)) => match token.kind() {
				Syn::Ident => Some(Ident(token)),
				_ => None,
			},
			_ => None,
		}
	}
}

// ReturnStat //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ReturnStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ReturnStat(SyntaxNode);

simple_astnode!(Syn, ReturnStat, Syn::ReturnStat);
