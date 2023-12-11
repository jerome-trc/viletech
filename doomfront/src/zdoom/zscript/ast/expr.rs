//! AST nodes for representing expressions.

use rowan::ast::AstNode;

use crate::{simple_astnode, zdoom::ast::LitToken, AstError, AstResult};

use super::super::{Syntax, SyntaxNode, SyntaxToken};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Expr {
	Binary(BinExpr),
	Call(CallExpr),
	ClassCast(ClassCastExpr),
	Group(GroupExpr),
	Ident(IdentExpr),
	Index(IndexExpr),
	Literal(Literal),
	Member(MemberExpr),
	Postfix(PostfixExpr),
	Prefix(PrefixExpr),
	Super(SuperExpr),
	Ternary(TernaryExpr),
	Vector(VectorExpr),
}

impl AstNode for Expr {
	type Language = Syntax;

	fn can_cast(kind: Syntax) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syntax::BinExpr
				| Syntax::CallExpr
				| Syntax::ClassCastExpr
				| Syntax::GroupExpr
				| Syntax::IdentExpr
				| Syntax::IndexExpr
				| Syntax::Literal
				| Syntax::MemberExpr
				| Syntax::PostfixExpr
				| Syntax::PrefixExpr
				| Syntax::SuperExpr
				| Syntax::TernaryExpr
				| Syntax::VectorExpr
		)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syntax::BinExpr => Some(Self::Binary(BinExpr(node))),
			Syntax::CallExpr => Some(Self::Call(CallExpr(node))),
			Syntax::ClassCastExpr => Some(Self::ClassCast(ClassCastExpr(node))),
			Syntax::GroupExpr => Some(Self::Group(GroupExpr(node))),
			Syntax::IdentExpr => Some(Self::Ident(IdentExpr(node))),
			Syntax::IndexExpr => Some(Self::Index(IndexExpr(node))),
			Syntax::Literal => Some(Self::Literal(Literal(node))),
			Syntax::MemberExpr => Some(Self::Member(MemberExpr(node))),
			Syntax::PostfixExpr => Some(Self::Postfix(PostfixExpr(node))),
			Syntax::PrefixExpr => Some(Self::Prefix(PrefixExpr(node))),
			Syntax::SuperExpr => Some(Self::Super(SuperExpr(node))),
			Syntax::TernaryExpr => Some(Self::Ternary(TernaryExpr(node))),
			Syntax::VectorExpr => Some(Self::Vector(VectorExpr(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Binary(inner) => inner.syntax(),
			Self::Call(inner) => inner.syntax(),
			Self::ClassCast(inner) => inner.syntax(),
			Self::Group(inner) => inner.syntax(),
			Self::Ident(inner) => inner.syntax(),
			Self::Index(inner) => inner.syntax(),
			Self::Literal(inner) => inner.syntax(),
			Self::Member(inner) => inner.syntax(),
			Self::Postfix(inner) => inner.syntax(),
			Self::Prefix(inner) => inner.syntax(),
			Self::Super(inner) => inner.syntax(),
			Self::Ternary(inner) => inner.syntax(),
			Self::Vector(inner) => inner.syntax(),
		}
	}
}

/// A subset of [`Expr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum PrimaryExpr {
	ClassCast(ClassCastExpr),
	Call(CallExpr),
	Group(GroupExpr),
	Ident(IdentExpr),
	Index(IndexExpr),
	Literal(Literal),
	Member(MemberExpr),
	Postfix(PostfixExpr),
	Super(SuperExpr),
	Vector(VectorExpr),
}

impl AstNode for PrimaryExpr {
	type Language = Syntax;

	fn can_cast(kind: Syntax) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syntax::CallExpr
				| Syntax::ClassCastExpr
				| Syntax::GroupExpr
				| Syntax::IdentExpr
				| Syntax::IndexExpr
				| Syntax::Literal
				| Syntax::MemberExpr
				| Syntax::PostfixExpr
				| Syntax::SuperExpr
				| Syntax::VectorExpr
		)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syntax::CallExpr => Some(Self::Call(CallExpr(node))),
			Syntax::ClassCastExpr => Some(Self::ClassCast(ClassCastExpr(node))),
			Syntax::GroupExpr => Some(Self::Group(GroupExpr(node))),
			Syntax::IdentExpr => Some(Self::Ident(IdentExpr(node))),
			Syntax::IndexExpr => Some(Self::Index(IndexExpr(node))),
			Syntax::Literal => Some(Self::Literal(Literal(node))),
			Syntax::MemberExpr => Some(Self::Member(MemberExpr(node))),
			Syntax::PostfixExpr => Some(Self::Postfix(PostfixExpr(node))),
			Syntax::SuperExpr => Some(Self::Super(SuperExpr(node))),
			Syntax::VectorExpr => Some(Self::Vector(VectorExpr(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Call(inner) => inner.syntax(),
			Self::ClassCast(inner) => inner.syntax(),
			Self::Group(inner) => inner.syntax(),
			Self::Ident(inner) => inner.syntax(),
			Self::Index(inner) => inner.syntax(),
			Self::Literal(inner) => inner.syntax(),
			Self::Member(inner) => inner.syntax(),
			Self::Postfix(inner) => inner.syntax(),
			Self::Super(inner) => inner.syntax(),
			Self::Vector(inner) => inner.syntax(),
		}
	}
}

impl From<PrimaryExpr> for Expr {
	fn from(value: PrimaryExpr) -> Self {
		match value {
			PrimaryExpr::ClassCast(inner) => Self::ClassCast(inner),
			PrimaryExpr::Call(inner) => Self::Call(inner),
			PrimaryExpr::Group(inner) => Self::Group(inner),
			PrimaryExpr::Ident(inner) => Self::Ident(inner),
			PrimaryExpr::Index(inner) => Self::Index(inner),
			PrimaryExpr::Literal(inner) => Self::Literal(inner),
			PrimaryExpr::Member(inner) => Self::Member(inner),
			PrimaryExpr::Postfix(inner) => Self::Postfix(inner),
			PrimaryExpr::Super(inner) => Self::Super(inner),
			PrimaryExpr::Vector(inner) => Self::Vector(inner),
		}
	}
}

// BinExpr /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::BinExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct BinExpr(SyntaxNode);

simple_astnode!(Syntax, BinExpr, Syntax::BinExpr);

impl BinExpr {
	#[must_use]
	pub fn left(&self) -> Expr {
		Expr::cast(self.0.first_child().unwrap()).unwrap()
	}

	#[must_use]
	pub fn operator(&self) -> (SyntaxToken, BinOp) {
		let ret0 = self
			.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| !token.kind().is_trivia()))
			.unwrap();

		let ret1 = match ret0.kind() {
			Syntax::Ampersand => BinOp::Ampersand,
			Syntax::Ampersand2 => BinOp::Ampersand2,
			Syntax::AmpersandEq => BinOp::AmpersandEq,
			Syntax::AngleL => BinOp::AngleL,
			Syntax::AngleL2 => BinOp::AngleL2,
			Syntax::AngleL2Eq => BinOp::AngleL2Eq,
			Syntax::AngleLAngleREq => BinOp::AngleLAngleREq,
			Syntax::AngleLEq => BinOp::AngleLEq,
			Syntax::AngleR => BinOp::AngleR,
			Syntax::AngleR2 => BinOp::AngleR2,
			Syntax::AngleR2Eq => BinOp::AngleR2Eq,
			Syntax::AngleR3 => BinOp::AngleR3,
			Syntax::AngleR3Eq => BinOp::AngleR3Eq,
			Syntax::AngleREq => BinOp::AngleREq,
			Syntax::Asterisk => BinOp::Asterisk,
			Syntax::Asterisk2 => BinOp::Asterisk2,
			Syntax::AsteriskEq => BinOp::AsteriskEq,
			Syntax::Bang => BinOp::Bang,
			Syntax::BangEq => BinOp::BangEq,
			Syntax::Caret => BinOp::Caret,
			Syntax::CaretEq => BinOp::CaretEq,
			Syntax::Dot2 => BinOp::Dot2,
			Syntax::Eq => BinOp::Eq,
			Syntax::Eq2 => BinOp::Eq2,
			Syntax::KwAlignOf => BinOp::KwAlignOf,
			Syntax::KwCross => BinOp::KwCross,
			Syntax::KwDot => BinOp::KwDot,
			Syntax::KwIs => BinOp::KwIs,
			Syntax::KwSizeOf => BinOp::KwSizeOf,
			Syntax::Minus => BinOp::Minus,
			Syntax::Minus2 => BinOp::Minus2,
			Syntax::MinusEq => BinOp::MinusEq,
			Syntax::Percent => BinOp::Percent,
			Syntax::PercentEq => BinOp::PercentEq,
			Syntax::Pipe => BinOp::Pipe,
			Syntax::Pipe2 => BinOp::Pipe2,
			Syntax::PipeEq => BinOp::PipeEq,
			Syntax::Plus => BinOp::Plus,
			Syntax::Plus2 => BinOp::Plus2,
			Syntax::PlusEq => BinOp::PlusEq,
			Syntax::Slash => BinOp::Slash,
			Syntax::SlashEq => BinOp::SlashEq,
			Syntax::Tilde => BinOp::Tilde,
			Syntax::TildeEq2 => BinOp::TildeEq2,
			_ => unreachable!(),
		};

		(ret0, ret1)
	}

	pub fn right(&self) -> AstResult<Expr> {
		Expr::cast(self.0.children().nth(1).ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}
}

/// See [`BinExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum BinOp {
	Ampersand,
	Ampersand2,
	AmpersandEq,
	AngleL,
	AngleL2,
	AngleL2Eq,
	AngleLAngleREq,
	AngleLEq,
	AngleR,
	AngleR2,
	AngleR2Eq,
	AngleR3,
	AngleR3Eq,
	AngleREq,
	Asterisk,
	Asterisk2,
	AsteriskEq,
	Bang,
	BangEq,
	Caret,
	CaretEq,
	Dot2,
	Eq,
	Eq2,
	KwAlignOf,
	KwCross,
	KwDot,
	KwIs,
	KwSizeOf,
	Minus,
	Minus2,
	MinusEq,
	Percent,
	PercentEq,
	Pipe,
	Pipe2,
	PipeEq,
	Plus,
	Plus2,
	PlusEq,
	Slash,
	SlashEq,
	Tilde,
	TildeEq2,
}

// CallExpr ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::CallExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CallExpr(SyntaxNode);

simple_astnode!(Syntax, CallExpr, Syntax::CallExpr);

impl CallExpr {
	#[must_use]
	pub fn called(&self) -> PrimaryExpr {
		PrimaryExpr::cast(self.0.first_child().unwrap()).unwrap()
	}

	#[must_use]
	pub fn arg_list(&self) -> ArgList {
		let node = self.0.last_child().unwrap();
		debug_assert!(node.kind() == Syntax::ArgList);
		ArgList(node)
	}
}

/// Wraps a node tagged [`Syntax::ArgList`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ArgList(SyntaxNode);

simple_astnode!(Syntax, ArgList, Syntax::ArgList);

impl ArgList {
	pub fn args(&self) -> impl Iterator<Item = Argument> {
		self.0.children().filter_map(|node| match node.kind() {
			Syntax::Argument => Some(Argument(node)),
			_ => None,
		})
	}
}

/// Wraps a node tagged [`Syntax::Argument`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Argument(SyntaxNode);

simple_astnode!(Syntax, Argument, Syntax::Argument);

impl Argument {
	/// The returned token is always tagged [`Syntax::Ident`].
	#[must_use]
	pub fn name(&self) -> Option<SyntaxToken> {
		let Some(ret) = self
			.0
			.first_token()
			.filter(|token| token.kind() == Syntax::Ident)
		else {
			return None;
		};

		if let Expr::Ident(e_id) = self.expr() {
			if e_id.token().index() == ret.index() {
				return None;
			}
		}

		Some(ret)
	}

	#[must_use]
	pub fn expr(&self) -> Expr {
		Expr::cast(self.0.last_child().unwrap()).unwrap()
	}
}

// ClassCastExpr ///////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ClassCastExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ClassCastExpr(SyntaxNode);

simple_astnode!(Syntax, ClassCastExpr, Syntax::ClassCastExpr);

impl ClassCastExpr {
	/// The returned token is always tagged [`Syntax::Ident`].
	pub fn class_name(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::Ident)
			})
			.ok_or(AstError::Missing)
	}

	#[must_use]
	pub fn arg_list(&self) -> ArgList {
		let node = self.0.last_child().unwrap();
		debug_assert!(node.kind() == Syntax::ArgList);
		ArgList(node)
	}
}

// GroupExpr ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::GroupExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GroupExpr(SyntaxNode);

simple_astnode!(Syntax, GroupExpr, Syntax::GroupExpr);

impl GroupExpr {
	#[must_use]
	pub fn inner(&self) -> Expr {
		Expr::cast(self.0.first_child().unwrap()).unwrap()
	}
}

// IdentExpr ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::IdentExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct IdentExpr(SyntaxNode);

simple_astnode!(Syntax, IdentExpr, Syntax::IdentExpr);

impl IdentExpr {
	/// The returned token is always tagged [`Syntax::Ident`].
	#[must_use]
	pub fn token(&self) -> SyntaxToken {
		let token = self.0.first_token().unwrap();
		debug_assert_eq!(token.kind(), Syntax::Ident);
		token
	}
}

// IndexExpr ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::IndexExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct IndexExpr(SyntaxNode);

simple_astnode!(Syntax, IndexExpr, Syntax::IndexExpr);

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

/// Wraps a node tagged [`Syntax::Literal`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Literal(SyntaxNode);

simple_astnode!(Syntax, Literal, Syntax::Literal);

impl Literal {
	/// Mind that this may not be the real whole literal if dealing with strings.
	/// See [`Self::strings`].
	#[must_use]
	pub fn token(&self) -> LitToken<Syntax> {
		LitToken::new(self.0.first_token().unwrap())
	}

	/// A ZScript string literal expression can be formed by writing multiple
	/// string literals adjacently.
	pub fn strings(&self) -> Option<impl Iterator<Item = LitToken<Syntax>>> {
		if self.0.first_token().unwrap().kind() == Syntax::StringLit {
			Some(self.0.children_with_tokens().filter_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::StringLit)
					.map(LitToken::new)
			}))
		} else {
			None
		}
	}
}

// MemberExpr //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::MemberExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MemberExpr(SyntaxNode);

simple_astnode!(Syntax, MemberExpr, Syntax::MemberExpr);

impl MemberExpr {
	#[must_use]
	pub fn left(&self) -> PrimaryExpr {
		PrimaryExpr::cast(self.0.first_child().unwrap()).unwrap()
	}

	/// The returned token is always tagged [`Syntax::Ident`].
	pub fn member_name(&self) -> AstResult<SyntaxToken> {
		self.0
			.last_token()
			.filter(|token| token.kind() == Syntax::Ident)
			.ok_or(AstError::Missing)
	}
}

// PostfixExpr /////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::PostfixExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PostfixExpr(SyntaxNode);

simple_astnode!(Syntax, PostfixExpr, Syntax::PostfixExpr);

impl PostfixExpr {
	#[must_use]
	pub fn operand(&self) -> Expr {
		Expr::cast(self.0.last_child().unwrap()).unwrap()
	}

	#[must_use]
	pub fn operator(&self) -> (SyntaxToken, PostfixOp) {
		let ret0 = self.0.last_token().unwrap();

		let ret1 = match ret0.kind() {
			Syntax::Minus2 => PostfixOp::Minus2,
			Syntax::Plus2 => PostfixOp::Plus2,
			_ => unreachable!(),
		};

		(ret0, ret1)
	}
}

/// See [`PostfixExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum PostfixOp {
	Minus2,
	Plus2,
}

// PrefixExpr //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::PrefixExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PrefixExpr(SyntaxNode);

simple_astnode!(Syntax, PrefixExpr, Syntax::PrefixExpr);

impl PrefixExpr {
	#[must_use]
	pub fn operand(&self) -> Expr {
		Expr::cast(self.0.last_child().unwrap()).unwrap()
	}

	#[must_use]
	pub fn operator(&self) -> (SyntaxToken, PrefixOp) {
		let ret0 = self.0.first_token().unwrap();

		let ret1 = match ret0.kind() {
			Syntax::Bang => PrefixOp::Bang,
			Syntax::Minus => PrefixOp::Minus,
			Syntax::Minus2 => PrefixOp::Minus2,
			Syntax::Plus => PrefixOp::Plus,
			Syntax::Plus2 => PrefixOp::Plus2,
			Syntax::Tilde => PrefixOp::Tilde,
			_ => unreachable!(),
		};

		(ret0, ret1)
	}
}

/// See [`PrefixExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum PrefixOp {
	Bang,
	Minus,
	Minus2,
	Plus,
	Plus2,
	Tilde,
}

// SuperExpr ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::SuperExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SuperExpr(SyntaxNode);

simple_astnode!(Syntax, SuperExpr, Syntax::SuperExpr);

impl SuperExpr {
	/// The returned token is always tagged [`Syntax::KwSuper`].
	#[must_use]
	pub fn token(&self) -> SyntaxToken {
		let token = self.0.first_token().unwrap();
		debug_assert_eq!(token.kind(), Syntax::KwSuper);
		token
	}
}

// TernaryExpr /////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::TernaryExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TernaryExpr(SyntaxNode);

simple_astnode!(Syntax, TernaryExpr, Syntax::TernaryExpr);

impl TernaryExpr {
	#[must_use]
	pub fn condition(&self) -> Expr {
		Expr::cast(self.0.first_child().unwrap()).unwrap()
	}

	/// The returned token is always tagged [`Syntax::Question`].
	#[must_use]
	pub fn question_mark(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::Question)
			})
			.unwrap()
	}

	pub fn if_expr(&self) -> AstResult<Expr> {
		let Some(node) = self.0.children().nth(1) else {
			return Err(AstError::Missing);
		};
		Expr::cast(node).ok_or(AstError::Incorrect)
	}

	/// The returned token is always tagged [`Syntax::Colon`].
	pub fn colon(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::Colon)
			})
			.ok_or(AstError::Missing)
	}

	pub fn else_expr(&self) -> AstResult<Expr> {
		let Some(node) = self.0.children().nth(2) else {
			return Err(AstError::Missing);
		};
		Expr::cast(node).ok_or(AstError::Incorrect)
	}
}

// VectorExpr //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::VectorExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct VectorExpr(SyntaxNode);

simple_astnode!(Syntax, VectorExpr, Syntax::VectorExpr);

impl VectorExpr {
	/// The first element. Alternatively `a`, for the alpha component in a color.
	#[must_use]
	pub fn x(&self) -> Expr {
		Expr::cast(self.0.first_child().unwrap()).unwrap()
	}

	/// The second element. Alternatively `r`, for the red component in a color.
	#[must_use]
	pub fn y(&self) -> Expr {
		self.0
			.children()
			.nth(1)
			.map(|node| Expr::cast(node).unwrap())
			.unwrap()
	}

	/// The third element. Alternatively `g`, for the green component in a color.
	#[must_use]
	pub fn z(&self) -> Option<Expr> {
		self.0
			.children()
			.nth(2)
			.map(|node| Expr::cast(node).unwrap())
	}

	/// The fourth element. Alternatively `b`, for the blue component in a color.
	#[must_use]
	pub fn w(&self) -> Option<Expr> {
		self.0
			.children()
			.nth(3)
			.map(|node| Expr::cast(node).unwrap())
	}

	pub fn elements(&self) -> impl Iterator<Item = Expr> {
		self.0.children().map(|node| Expr::cast(node).unwrap())
	}
}
