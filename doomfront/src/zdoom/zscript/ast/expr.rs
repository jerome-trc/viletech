//! AST nodes for representing expressions.

use rowan::ast::AstNode;

use crate::{simple_astnode, zdoom::ast::LitToken, AstError, AstResult};

use super::super::{Syn, SyntaxNode, SyntaxToken};

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
	Postfix(PostfixExpr),
	Prefix(PrefixExpr),
	Super(SuperExpr),
	Ternary(TernaryExpr),
	Vector(VectorExpr),
}

impl AstNode for Expr {
	type Language = Syn;

	fn can_cast(kind: Syn) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syn::BinExpr
				| Syn::CallExpr | Syn::ClassCastExpr
				| Syn::GroupExpr | Syn::IdentExpr
				| Syn::IndexExpr | Syn::Literal
				| Syn::PostfixExpr
				| Syn::PrefixExpr
				| Syn::SuperExpr | Syn::TernaryExpr
				| Syn::VectorExpr
		)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::BinExpr => Some(Self::Binary(BinExpr(node))),
			Syn::CallExpr => Some(Self::Call(CallExpr(node))),
			Syn::ClassCastExpr => Some(Self::ClassCast(ClassCastExpr(node))),
			Syn::GroupExpr => Some(Self::Group(GroupExpr(node))),
			Syn::IdentExpr => Some(Self::Ident(IdentExpr(node))),
			Syn::IndexExpr => Some(Self::Index(IndexExpr(node))),
			Syn::Literal => Some(Self::Literal(Literal(node))),
			Syn::PostfixExpr => Some(Self::Postfix(PostfixExpr(node))),
			Syn::PrefixExpr => Some(Self::Prefix(PrefixExpr(node))),
			Syn::SuperExpr => Some(Self::Super(SuperExpr(node))),
			Syn::TernaryExpr => Some(Self::Ternary(TernaryExpr(node))),
			Syn::VectorExpr => Some(Self::Vector(VectorExpr(node))),
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
			Self::Postfix(inner) => inner.syntax(),
			Self::Prefix(inner) => inner.syntax(),
			Self::Super(inner) => inner.syntax(),
			Self::Ternary(inner) => inner.syntax(),
			Self::Vector(inner) => inner.syntax(),
		}
	}
}

impl Expr {
	#[must_use]
	pub fn into_bin_expr(self) -> Option<BinExpr> {
		match self {
			Self::Binary(inner) => Some(inner),
			_ => None,
		}
	}

	#[must_use]
	pub fn into_call_expr(self) -> Option<CallExpr> {
		match self {
			Self::Call(inner) => Some(inner),
			_ => None,
		}
	}

	#[must_use]
	pub fn into_class_cast_expr(self) -> Option<ClassCastExpr> {
		match self {
			Self::ClassCast(inner) => Some(inner),
			_ => None,
		}
	}

	#[must_use]
	pub fn into_group_expr(self) -> Option<GroupExpr> {
		match self {
			Self::Group(inner) => Some(inner),
			_ => None,
		}
	}

	#[must_use]
	pub fn into_ident_expr(self) -> Option<IdentExpr> {
		match self {
			Self::Ident(inner) => Some(inner),
			_ => None,
		}
	}

	#[must_use]
	pub fn into_index_expr(self) -> Option<IndexExpr> {
		match self {
			Self::Index(inner) => Some(inner),
			_ => None,
		}
	}

	#[must_use]
	pub fn into_lit_expr(self) -> Option<Literal> {
		match self {
			Self::Literal(inner) => Some(inner),
			_ => None,
		}
	}

	#[must_use]
	pub fn into_postfix_expr(self) -> Option<PostfixExpr> {
		match self {
			Self::Postfix(inner) => Some(inner),
			_ => None,
		}
	}

	#[must_use]
	pub fn into_prefix_expr(self) -> Option<PrefixExpr> {
		match self {
			Self::Prefix(inner) => Some(inner),
			_ => None,
		}
	}

	#[must_use]
	pub fn into_super_expr(self) -> Option<SuperExpr> {
		match self {
			Self::Super(inner) => Some(inner),
			_ => None,
		}
	}

	#[must_use]
	pub fn into_ternary_expr(self) -> Option<TernaryExpr> {
		match self {
			Self::Ternary(inner) => Some(inner),
			_ => None,
		}
	}

	#[must_use]
	pub fn into_vector_expr(self) -> Option<VectorExpr> {
		match self {
			Self::Vector(inner) => Some(inner),
			_ => None,
		}
	}
}

// BinExpr /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::BinExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct BinExpr(SyntaxNode);

simple_astnode!(Syn, BinExpr, Syn::BinExpr);

impl BinExpr {
	#[must_use]
	pub fn lhs(&self) -> Expr {
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
			Syn::Ampersand => BinOp::Ampersand,
			Syn::Ampersand2 => BinOp::Ampersand2,
			Syn::AmpersandEq => BinOp::AmpersandEq,
			Syn::AngleL => BinOp::AngleL,
			Syn::AngleL2 => BinOp::AngleL2,
			Syn::AngleL2Eq => BinOp::AngleL2Eq,
			Syn::AngleLAngleREq => BinOp::AngleLAngleREq,
			Syn::AngleLEq => BinOp::AngleLEq,
			Syn::AngleR => BinOp::AngleR,
			Syn::AngleR2 => BinOp::AngleR2,
			Syn::AngleR2Eq => BinOp::AngleR2Eq,
			Syn::AngleR3 => BinOp::AngleR3,
			Syn::AngleR3Eq => BinOp::AngleR3Eq,
			Syn::AngleREq => BinOp::AngleREq,
			Syn::Asterisk => BinOp::Asterisk,
			Syn::Asterisk2 => BinOp::Asterisk2,
			Syn::AsteriskEq => BinOp::AsteriskEq,
			Syn::Bang => BinOp::Bang,
			Syn::BangEq => BinOp::BangEq,
			Syn::Caret => BinOp::Caret,
			Syn::CaretEq => BinOp::CaretEq,
			Syn::Dot => BinOp::Dot,
			Syn::Dot2 => BinOp::Dot2,
			Syn::Eq => BinOp::Eq,
			Syn::Eq2 => BinOp::Eq2,
			Syn::KwAlignOf => BinOp::KwAlignOf,
			Syn::KwCross => BinOp::KwCross,
			Syn::KwDot => BinOp::KwDot,
			Syn::KwIs => BinOp::KwIs,
			Syn::KwSizeOf => BinOp::KwSizeOf,
			Syn::Minus => BinOp::Minus,
			Syn::Minus2 => BinOp::Minus2,
			Syn::MinusEq => BinOp::MinusEq,
			Syn::Percent => BinOp::Percent,
			Syn::PercentEq => BinOp::PercentEq,
			Syn::Pipe => BinOp::Pipe,
			Syn::Pipe2 => BinOp::Pipe2,
			Syn::PipeEq => BinOp::PipeEq,
			Syn::Plus => BinOp::Plus,
			Syn::Plus2 => BinOp::Plus2,
			Syn::PlusEq => BinOp::PlusEq,
			Syn::Question => BinOp::Question,
			Syn::Slash => BinOp::Slash,
			Syn::SlashEq => BinOp::SlashEq,
			Syn::Tilde => BinOp::Tilde,
			Syn::TildeEq2 => BinOp::TildeEq2,
			_ => unreachable!(),
		};

		(ret0, ret1)
	}

	#[must_use]
	pub fn rhs(&self) -> Expr {
		Expr::cast(self.0.last_child().unwrap()).unwrap()
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
	Dot,
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
	Question,
	Slash,
	SlashEq,
	Tilde,
	TildeEq2,
}

// CallExpr ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::CallExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CallExpr(SyntaxNode);

simple_astnode!(Syn, CallExpr, Syn::CallExpr);

impl CallExpr {
	#[must_use]
	pub fn called(&self) -> Expr {
		Expr::cast(self.0.first_child().unwrap()).unwrap()
	}

	#[must_use]
	pub fn arg_list(&self) -> ArgList {
		let node = self.0.last_child().unwrap();
		debug_assert!(node.kind() == Syn::ArgList);
		ArgList(node)
	}
}

/// Wraps a node tagged [`Syn::ArgList`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ArgList(SyntaxNode);

simple_astnode!(Syn, ArgList, Syn::ArgList);

impl ArgList {
	pub fn args(&self) -> impl Iterator<Item = Argument> {
		self.0.children().filter_map(|node| match node.kind() {
			Syn::Argument => Some(Argument(node)),
			_ => None,
		})
	}
}

/// Wraps a node tagged [`Syn::Argument`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Argument(SyntaxNode);

simple_astnode!(Syn, Argument, Syn::Argument);

impl Argument {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		let token = self.0.first_token().unwrap();
		debug_assert!(token.kind() == Syn::Ident);
		token
	}

	#[must_use]
	pub fn expr(&self) -> Expr {
		Expr::cast(self.0.last_child().unwrap()).unwrap()
	}
}

// ClassCastExpr ///////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ClassCastExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ClassCastExpr(SyntaxNode);

simple_astnode!(Syn, ClassCastExpr, Syn::ClassCastExpr);

impl ClassCastExpr {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn class_name(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.unwrap()
	}

	#[must_use]
	pub fn arg_list(&self) -> ArgList {
		let node = self.0.last_child().unwrap();
		debug_assert!(node.kind() == Syn::ArgList);
		ArgList(node)
	}
}

// GroupExpr ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::GroupExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GroupExpr(SyntaxNode);

simple_astnode!(Syn, GroupExpr, Syn::GroupExpr);

impl GroupExpr {
	#[must_use]
	pub fn inner(&self) -> Expr {
		Expr::cast(self.0.first_child().unwrap()).unwrap()
	}
}

// IdentExpr ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::IdentExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct IdentExpr(SyntaxNode);

simple_astnode!(Syn, IdentExpr, Syn::IdentExpr);

impl IdentExpr {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn token(&self) -> SyntaxToken {
		let token = self.0.first_token().unwrap();
		debug_assert_eq!(token.kind(), Syn::Ident);
		token
	}
}

// IndexExpr ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::IndexExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct IndexExpr(SyntaxNode);

simple_astnode!(Syn, IndexExpr, Syn::IndexExpr);

impl IndexExpr {
	#[must_use]
	pub fn indexed(&self) -> Expr {
		Expr::cast(self.0.first_child().unwrap()).unwrap()
	}

	#[must_use]
	pub fn index(&self) -> Expr {
		Expr::cast(self.0.last_child().unwrap()).unwrap()
	}
}

// Literal /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::Literal`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Literal(SyntaxNode);

simple_astnode!(Syn, Literal, Syn::Literal);

impl Literal {
	/// Mind that this may not be the real whole literal if dealing with strings.
	/// See [`Self::strings`].
	#[must_use]
	pub fn token(&self) -> LitToken<Syn> {
		LitToken::new(self.0.first_token().unwrap())
	}

	/// A ZScript string literal expression can be formed by writing multiple
	/// string literals adjacently.
	pub fn strings(&self) -> Option<impl Iterator<Item = LitToken<Syn>>> {
		if self.0.first_token().unwrap().kind() == Syn::StringLit {
			Some(self.0.children_with_tokens().filter_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syn::StringLit)
					.map(LitToken::new)
			}))
		} else {
			None
		}
	}
}

// MemberExpr //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::MemberExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MemberExpr(SyntaxNode);

simple_astnode!(Syn, MemberExpr, Syn::MemberExpr);

impl MemberExpr {
	#[must_use]
	pub fn lhs(&self) -> PrimaryExpr {
		PrimaryExpr::cast(self.0.first_child().unwrap()).unwrap()
	}

	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn member_name(&self) -> AstResult<SyntaxToken> {
		self.0.last_token().filter(|token| token.kind() == Syn::Ident).ok_or(AstError::Missing)
	}
}

// PostfixExpr /////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::PostfixExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PostfixExpr(SyntaxNode);

simple_astnode!(Syn, PostfixExpr, Syn::PostfixExpr);

impl PostfixExpr {
	#[must_use]
	pub fn operand(&self) -> Expr {
		Expr::cast(self.0.last_child().unwrap()).unwrap()
	}

	#[must_use]
	pub fn operator(&self) -> (SyntaxToken, PostfixOp) {
		let ret0 = self.0.last_token().unwrap();

		let ret1 = match ret0.kind() {
			Syn::Minus2 => PostfixOp::Minus2,
			Syn::Plus2 => PostfixOp::Plus2,
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

/// Wraps a node tagged [`Syn::PrefixExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PrefixExpr(SyntaxNode);

simple_astnode!(Syn, PrefixExpr, Syn::PrefixExpr);

impl PrefixExpr {
	#[must_use]
	pub fn operand(&self) -> Expr {
		Expr::cast(self.0.last_child().unwrap()).unwrap()
	}

	#[must_use]
	pub fn operator(&self) -> (SyntaxToken, PrefixOp) {
		let ret0 = self.0.first_token().unwrap();

		let ret1 = match ret0.kind() {
			Syn::Bang => PrefixOp::Bang,
			Syn::Minus => PrefixOp::Minus,
			Syn::Minus2 => PrefixOp::Minus2,
			Syn::Plus => PrefixOp::Plus,
			Syn::Plus2 => PrefixOp::Plus2,
			Syn::Tilde => PrefixOp::Tilde,
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

/// Wraps a node tagged [`Syn::SuperExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SuperExpr(SyntaxNode);

simple_astnode!(Syn, SuperExpr, Syn::SuperExpr);

impl SuperExpr {
	/// The returned token is always tagged [`Syn::KwSuper`].
	#[must_use]
	pub fn token(&self) -> SyntaxToken {
		let token = self.0.first_token().unwrap();
		debug_assert_eq!(token.kind(), Syn::KwSuper);
		token
	}
}

// TernaryExpr /////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::TernaryExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TernaryExpr(SyntaxNode);

simple_astnode!(Syn, TernaryExpr, Syn::TernaryExpr);

impl TernaryExpr {
	#[must_use]
	pub fn condition(&self) -> Expr {
		Expr::cast(self.0.first_child().unwrap()).unwrap()
	}

	pub fn if_expr(&self) -> AstResult<Expr> {
		let Some(node) = self.0.children().nth(1) else { return Err(AstError::Missing); };
		Expr::cast(node).ok_or(AstError::Incorrect)
	}

	pub fn else_expr(&self) -> AstResult<Expr> {
		let Some(node) = self.0.children().nth(2) else { return Err(AstError::Missing); };
		Expr::cast(node).ok_or(AstError::Incorrect)
	}
}

// VectorExpr //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::VectorExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct VectorExpr(SyntaxNode);

simple_astnode!(Syn, VectorExpr, Syn::VectorExpr);

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
