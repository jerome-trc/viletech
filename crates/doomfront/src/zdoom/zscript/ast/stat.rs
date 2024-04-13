//! AST nodes for representing statements.

use rowan::ast::AstNode;

use crate::{simple_astnode, AstError, AstResult};

use super::{CoreType, DocComment, Expr, LocalVar, Syntax, SyntaxNode, SyntaxToken, VarName};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Statement {
	Assign(AssignStat),
	Break(BreakStat),
	Case(CaseStat),
	Compound(CompoundStat),
	CondLoop(CondLoopStat),
	Continue(ContinueStat),
	DeclAssign(DeclAssignStat),
	Default(DefaultStat),
	Empty(EmptyStat),
	Expr(ExprStat),
	For(ForStat),
	ForEach(ForEachStat),
	If(IfStat),
	Local(LocalStat),
	Return(ReturnStat),
	StaticConst(StaticConstStat),
	Switch(SwitchStat),
}

impl AstNode for Statement {
	type Language = Syntax;

	fn can_cast(kind: Syntax) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syntax::AssignStat
				| Syntax::BreakStat
				| Syntax::CaseStat
				| Syntax::CompoundStat
				| Syntax::ContinueStat
				| Syntax::DeclAssignStat
				| Syntax::DefaultStat
				| Syntax::DoUntilStat
				| Syntax::DoWhileStat
				| Syntax::EmptyStat
				| Syntax::ExprStat
				| Syntax::ForStat
				| Syntax::ForEachStat
				| Syntax::IfStat | Syntax::LocalStat
				| Syntax::ReturnStat
				| Syntax::StaticConstStat
				| Syntax::SwitchStat
				| Syntax::UntilStat
				| Syntax::WhileStat
		)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syntax::AssignStat => Some(Self::Assign(AssignStat(node))),
			Syntax::BreakStat => Some(Self::Break(BreakStat(node))),
			Syntax::CaseStat => Some(Self::Case(CaseStat(node))),
			Syntax::CompoundStat => Some(Self::Compound(CompoundStat(node))),
			Syntax::ContinueStat => Some(Self::Continue(ContinueStat(node))),
			Syntax::DeclAssignStat => Some(Self::DeclAssign(DeclAssignStat(node))),
			Syntax::DefaultStat => Some(Self::Default(DefaultStat(node))),
			Syntax::EmptyStat => Some(Self::Empty(EmptyStat(node))),
			Syntax::ExprStat => Some(Self::Expr(ExprStat(node))),
			Syntax::ForStat => Some(Self::For(ForStat(node))),
			Syntax::ForEachStat => Some(Self::ForEach(ForEachStat(node))),
			Syntax::IfStat => Some(Self::If(IfStat(node))),
			Syntax::LocalStat => Some(Self::Local(LocalStat(node))),
			Syntax::ReturnStat => Some(Self::Return(ReturnStat(node))),
			Syntax::StaticConstStat => Some(Self::StaticConst(StaticConstStat(node))),
			Syntax::SwitchStat => Some(Self::Switch(SwitchStat(node))),
			Syntax::DoUntilStat | Syntax::DoWhileStat | Syntax::UntilStat | Syntax::WhileStat => {
				Some(Self::CondLoop(CondLoopStat(node)))
			}
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Assign(inner) => inner.syntax(),
			Self::Break(inner) => inner.syntax(),
			Self::Case(inner) => inner.syntax(),
			Self::Compound(inner) => inner.syntax(),
			Self::CondLoop(inner) => inner.syntax(),
			Self::Continue(inner) => inner.syntax(),
			Self::DeclAssign(inner) => inner.syntax(),
			Self::Default(inner) => inner.syntax(),
			Self::Empty(inner) => inner.syntax(),
			Self::Expr(inner) => inner.syntax(),
			Self::For(inner) => inner.syntax(),
			Self::ForEach(inner) => inner.syntax(),
			Self::If(inner) => inner.syntax(),
			Self::Local(inner) => inner.syntax(),
			Self::Return(inner) => inner.syntax(),
			Self::StaticConst(inner) => inner.syntax(),
			Self::Switch(inner) => inner.syntax(),
		}
	}
}

// AssignStat //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::AssignStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AssignStat(SyntaxNode);

simple_astnode!(Syntax, AssignStat, Syntax::AssignStat);

impl AssignStat {
	pub fn assigned(&self) -> impl Iterator<Item = Expr> {
		self.0
			.children_with_tokens()
			.take_while(|elem| elem.kind() != Syntax::Eq)
			.filter_map(|elem| elem.into_node().map(|node| Expr::cast(node).unwrap()))
	}

	pub fn assignee(&self) -> AstResult<Expr> {
		let Some(node) = self.0.last_child() else {
			return Err(AstError::Missing);
		};
		Expr::cast(node).ok_or(AstError::Incorrect)
	}
}

// BreakStat ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::BreakStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct BreakStat(SyntaxNode);

simple_astnode!(Syntax, BreakStat, Syntax::BreakStat);

// CaseStat ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::CaseStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CaseStat(SyntaxNode);

simple_astnode!(Syntax, CaseStat, Syntax::CaseStat);

impl CaseStat {
	pub fn expr(&self) -> AstResult<Expr> {
		let Some(node) = self.0.first_child() else {
			return Err(AstError::Missing);
		};
		Expr::cast(node).ok_or(AstError::Incorrect)
	}
}

// CompoundStat ////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::CompoundStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CompoundStat(SyntaxNode);

simple_astnode!(Syntax, CompoundStat, Syntax::CompoundStat);

impl CompoundStat {
	pub fn innards(&self) -> impl Iterator<Item = Statement> {
		self.0.children().filter_map(Statement::cast)
	}
}

// CondLoopStat ////////////////////////////////////////////////////////////////

/// Wraps a node tagged with one of the following:
/// - [`Syntax::DoUntilStat`]
/// - [`Syntax::DoWhileStat`]
/// - [`Syntax::UntilStat`]
/// - [`Syntax::WhileStat`]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CondLoopStat(SyntaxNode);

impl AstNode for CondLoopStat {
	type Language = Syntax;

	fn can_cast(kind: Syntax) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syntax::DoUntilStat | Syntax::DoWhileStat | Syntax::UntilStat | Syntax::WhileStat
		)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		if Self::can_cast(node.kind()) {
			Some(Self(node))
		} else {
			None
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}
}

impl CondLoopStat {
	#[must_use]
	pub fn is_do_loop(&self) -> bool {
		matches!(self.0.kind(), Syntax::DoUntilStat | Syntax::DoWhileStat)
	}

	#[must_use]
	pub fn is_while_loop(&self) -> bool {
		matches!(self.0.kind(), Syntax::WhileStat | Syntax::DoWhileStat)
	}

	#[must_use]
	pub fn is_until_loop(&self) -> bool {
		matches!(self.0.kind(), Syntax::UntilStat | Syntax::DoUntilStat)
	}

	pub fn condition(&self) -> AstResult<Expr> {
		match self.0.kind() {
			Syntax::DoUntilStat | Syntax::DoWhileStat => {
				let Some(node) = self.0.last_child() else {
					return Err(AstError::Missing);
				};
				Expr::cast(node).ok_or(AstError::Incorrect)
			}
			Syntax::WhileStat | Syntax::UntilStat => {
				let Some(node) = self.0.first_child() else {
					return Err(AstError::Missing);
				};
				Expr::cast(node).ok_or(AstError::Incorrect)
			}
			_ => unreachable!(),
		}
	}

	pub fn statement(&self) -> AstResult<Statement> {
		match self.0.kind() {
			Syntax::DoUntilStat | Syntax::DoWhileStat => {
				let Some(node) = self.0.first_child() else {
					return Err(AstError::Missing);
				};

				Statement::cast(node).ok_or(AstError::Incorrect)
			}
			Syntax::WhileStat | Syntax::UntilStat => {
				let Some(node) = self.0.last_child() else {
					return Err(AstError::Missing);
				};

				Statement::cast(node).ok_or(AstError::Incorrect)
			}
			_ => unreachable!(),
		}
	}
}

// ContinueStat ////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ContinueStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ContinueStat(SyntaxNode);

simple_astnode!(Syntax, ContinueStat, Syntax::ContinueStat);

// DeclAssignStat //////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::DeclAssignStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DeclAssignStat(SyntaxNode);

simple_astnode!(Syntax, DeclAssignStat, Syntax::DeclAssignStat);

impl DeclAssignStat {
	/// Yielded tokens are always tagged [`Syntax::Ident`].
	pub fn idents(&self) -> impl Iterator<Item = SyntaxToken> {
		self.0
			.children_with_tokens()
			.take_while(|elem| elem.kind() != Syntax::Eq)
			.filter_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::Ident)
			})
	}

	pub fn expr(&self) -> AstResult<Expr> {
		let Some(node) = self.0.last_child() else {
			return Err(AstError::Missing);
		};
		Expr::cast(node).ok_or(AstError::Incorrect)
	}
}

// DefaultStat /////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::DefaultStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DefaultStat(SyntaxNode);

simple_astnode!(Syntax, DefaultStat, Syntax::DefaultStat);

// EmptyStat ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::EmptyStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EmptyStat(SyntaxNode);

simple_astnode!(Syntax, EmptyStat, Syntax::EmptyStat);

impl EmptyStat {
	/// The returned token is always tagged [`Syntax::Semicolon`].
	#[must_use]
	pub fn semicolon(&self) -> SyntaxToken {
		self.0.first_token().unwrap()
	} // Yes, this is useful. A good linter should warn against these.
}

// ExprStat ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ExprStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprStat(SyntaxNode);

simple_astnode!(Syntax, ExprStat, Syntax::ExprStat);

impl ExprStat {
	#[must_use]
	pub fn expr(&self) -> Expr {
		Expr::cast(self.0.first_child().unwrap()).unwrap()
	}
}

// ForStat /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ForStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ForStat(SyntaxNode);

simple_astnode!(Syntax, ForStat, Syntax::ForStat);

impl ForStat {
	pub fn init(&self) -> AstResult<ForLoopInit> {
		let Some(node) = self.0.first_child() else {
			return Err(AstError::Missing);
		};
		ForLoopInit::cast(node).ok_or(AstError::Incorrect)
	}

	pub fn condition(&self) -> AstResult<ForLoopCond> {
		self.0
			.children()
			.find_map(ForLoopCond::cast)
			.ok_or(AstError::Missing)
	}

	pub fn iter(&self) -> AstResult<ForLoopIter> {
		self.0
			.children()
			.find_map(ForLoopIter::cast)
			.ok_or(AstError::Missing)
	}
}

/// Wraps a node tagged [`Syntax::ForLoopInit`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ForLoopInit(SyntaxNode);

simple_astnode!(Syntax, ForLoopInit, Syntax::ForLoopInit);

/// Wraps a node tagged [`Syntax::ForLoopCond`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ForLoopCond(SyntaxNode);

simple_astnode!(Syntax, ForLoopCond, Syntax::ForLoopCond);

impl ForLoopCond {
	#[must_use]
	pub fn expr(&self) -> Option<Expr> {
		self.0.first_child().map(|node| Expr::cast(node).unwrap())
	}
}

/// Wraps a node tagged [`Syntax::ForLoopIter`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ForLoopIter(SyntaxNode);

simple_astnode!(Syntax, ForLoopIter, Syntax::ForLoopIter);

impl ForLoopIter {
	pub fn exprs(&self) -> impl Iterator<Item = Expr> {
		self.0.children().map(|node| Expr::cast(node).unwrap())
	}
}

// ForEachStat /////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ForEachStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ForEachStat(SyntaxNode);

simple_astnode!(Syntax, ForEachStat, Syntax::ForEachStat);

impl ForEachStat {
	pub fn variable(&self) -> AstResult<VarName> {
		let Some(node) = self.0.first_child() else {
			return Err(AstError::Missing);
		};
		VarName::cast(node).ok_or(AstError::Incorrect)
	}

	pub fn collection(&self) -> AstResult<Expr> {
		self.0
			.children()
			.find_map(Expr::cast)
			.ok_or(AstError::Missing)
	}

	pub fn statement(&self) -> AstResult<Statement> {
		let Some(node) = self.0.last_child() else {
			return Err(AstError::Missing);
		};
		Statement::cast(node).ok_or(AstError::Incorrect)
	}
}

// IfStat //////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::IfStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct IfStat(SyntaxNode);

simple_astnode!(Syntax, IfStat, Syntax::IfStat);

impl IfStat {
	pub fn condition(&self) -> AstResult<Expr> {
		Expr::cast(self.0.last_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}

	pub fn statement(&self) -> AstResult<Statement> {
		Statement::cast(self.0.first_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}
}

// LocalStat ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::LocalStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LocalStat(SyntaxNode);

simple_astnode!(Syntax, LocalStat, Syntax::LocalStat);

impl LocalStat {
	#[must_use]
	pub fn var(&self) -> LocalVar {
		let ret = self.0.first_child().unwrap();
		debug_assert_eq!(ret.kind(), Syntax::LocalVar);
		LocalVar(ret)
	}
}

// ReturnStat //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ReturnStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ReturnStat(SyntaxNode);

simple_astnode!(Syntax, ReturnStat, Syntax::ReturnStat);

impl ReturnStat {
	pub fn exprs(&self) -> impl Iterator<Item = Expr> {
		self.0.children().filter_map(Expr::cast)
	}
}

// StaticConstStat /////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::StaticConstStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StaticConstStat(pub(super) SyntaxNode);

simple_astnode!(Syntax, StaticConstStat, Syntax::StaticConstStat);

impl StaticConstStat {
	pub fn docs(&self) -> impl Iterator<Item = DocComment> {
		super::doc_comments(&self.0)
	}

	/// `0` is always tagged [`Syntax::KwStatic`]; `1` is always tagged [`Syntax::KwConst`].
	#[must_use]
	pub fn keywords(&self) -> (SyntaxToken, SyntaxToken) {
		let ret0 = self
			.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::KwStatic)
			})
			.unwrap();

		let ret1 = self
			.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::KwConst)
			})
			.unwrap();

		(ret0, ret1)
	}

	/// The returned token is always tagged [`Syntax::Ident`].
	pub fn name(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::Ident)
			})
			.ok_or(AstError::Missing)
	}

	pub fn type_spec(&self) -> AstResult<CoreType> {
		CoreType::cast(self.0.first_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}

	pub fn values(&self) -> impl Iterator<Item = Expr> {
		self.0.children().filter_map(Expr::cast)
	}
}

// SwitchStat //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::SwitchStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SwitchStat(SyntaxNode);

simple_astnode!(Syntax, SwitchStat, Syntax::SwitchStat);

impl SwitchStat {
	pub fn expr(&self) -> AstResult<Expr> {
		let Some(node) = self.0.first_child() else {
			return Err(AstError::Missing);
		};
		Expr::cast(node).ok_or(AstError::Incorrect)
	}

	pub fn statement(&self) -> AstResult<Statement> {
		let Some(node) = self.0.last_child() else {
			return Err(AstError::Missing);
		};
		Statement::cast(node).ok_or(AstError::Incorrect)
	}
}
