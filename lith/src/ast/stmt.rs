//! Abstract syntax tree nodes for representing statements.

use doomfront::{rowan::ast::AstNode, simple_astnode, AstError, AstResult};

use crate::{Syntax, SyntaxNode, SyntaxToken};

use super::{BlockLabel, Expr, Pattern, TypeSpec};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Statement {
	Bind(StmtBind),
	Break(StmtBreak),
	Continue(StmtContinue),
	Expr(StmtExpr),
	Return(StmtReturn),
}

impl AstNode for Statement {
	type Language = Syntax;

	fn can_cast(kind: Syntax) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syntax::StmtBind
				| Syntax::StmtBreak
				| Syntax::StmtContinue
				| Syntax::StmtExpr
				| Syntax::StmtReturn
		)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syntax::StmtBind => Some(Self::Bind(StmtBind(node))),
			Syntax::StmtBreak => Some(Self::Break(StmtBreak(node))),
			Syntax::StmtContinue => Some(Self::Continue(StmtContinue(node))),
			Syntax::StmtExpr => Some(Self::Expr(StmtExpr(node))),
			Syntax::StmtReturn => Some(Self::Return(StmtReturn(node))),
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

// Bind ////////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::StmtBind`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StmtBind(SyntaxNode);

simple_astnode!(Syntax, StmtBind, Syntax::StmtBind);

impl StmtBind {
	#[must_use]
	pub fn keyword(&self) -> BindKeyword {
		let token = self.0.first_token().unwrap();

		match token.kind() {
			Syntax::KwLet => BindKeyword::Let(token),
			Syntax::KwVar => BindKeyword::Var(token),
			_ => unreachable!(),
		}
	}

	#[must_use]
	pub fn const_kw(&self) -> Option<SyntaxToken> {
		self.0
			.children_with_tokens()
			.take_while(|elem| !Pattern::can_cast(elem.kind()) && elem.kind() != Syntax::Error)
			.find_map(|elem| elem.into_token().filter(|t| t.kind() == Syntax::KwConst))
	}

	#[must_use]
	pub fn is_const(&self) -> bool {
		self.const_kw().is_some()
	}

	pub fn pattern(&self) -> AstResult<Pattern> {
		self.0
			.children()
			.find_map(Pattern::cast)
			.ok_or(AstError::Missing)
	}

	#[must_use]
	pub fn type_spec(&self) -> Option<TypeSpec> {
		self.0.children().find_map(TypeSpec::cast)
	}

	#[must_use]
	pub fn init(&self) -> Option<Expr> {
		self.0.children().find_map(Expr::cast)
	}
}

/// See [`StmtBind::keyword`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum BindKeyword {
	Let(SyntaxToken),
	Var(SyntaxToken),
}

// Break ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::StmtBreak`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StmtBreak(SyntaxNode);

simple_astnode!(Syntax, StmtBreak, Syntax::StmtBreak);

impl StmtBreak {
	#[must_use]
	pub fn expr(&self) -> Option<Expr> {
		self.0.children().find_map(Expr::cast)
	}

	#[must_use]
	pub fn block_label(&self) -> Option<BlockLabel> {
		self.0.children().find_map(BlockLabel::cast)
	}
}

// Continue ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::StmtContinue`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StmtContinue(SyntaxNode);

simple_astnode!(Syntax, StmtContinue, Syntax::StmtContinue);

impl StmtContinue {
	/// The returned token is always tagged [`Syntax::KwContinue`].
	#[must_use]
	pub fn keyword(&self) -> SyntaxToken {
		let ret = self.0.first_token().unwrap();
		debug_assert_eq!(ret.kind(), Syntax::KwContinue);
		ret
	}

	#[must_use]
	pub fn label(&self) -> Option<BlockLabel> {
		self.0.last_child().and_then(BlockLabel::cast)
	}
}

// Expression //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::StmtExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StmtExpr(SyntaxNode);

simple_astnode!(Syntax, StmtExpr, Syntax::StmtExpr);

impl StmtExpr {
	pub fn expr(&self) -> AstResult<Expr> {
		Expr::cast(self.0.first_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}
}

// Return //////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::StmtReturn`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StmtReturn(SyntaxNode);

simple_astnode!(Syntax, StmtReturn, Syntax::StmtReturn);

impl StmtReturn {
	pub fn expr(&self) -> AstResult<Expr> {
		Expr::cast(self.0.last_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}
}
