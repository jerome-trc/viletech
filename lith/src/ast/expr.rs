//! Abstract syntax tree nodes for representing expressions.

use doomfront::{rowan::ast::AstNode, simple_astnode, AstError, AstResult};

use crate::{Syntax, SyntaxNode, SyntaxToken};

use super::{LitToken, Name, TopLevel};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Expr {
	Binary(ExprBin),
	Block(ExprBlock),
	Field(ExprField),
	Group(ExprGroup),
	Ident(ExprIdent),
	Literal(ExprLit),
	Postfix(ExprPostfix),
	Prefix(ExprPrefix),
	Type(ExprType),
}

impl AstNode for Expr {
	type Language = Syntax;

	fn can_cast(kind: Syntax) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syntax::ExprBin
				| Syntax::ExprBlock
				| Syntax::ExprField
				| Syntax::ExprGroup
				| Syntax::ExprIdent
				| Syntax::ExprLit
				| Syntax::ExprPostfix
				| Syntax::ExprPrefix
				| Syntax::ExprType
		)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syntax::ExprBin => Some(Self::Binary(ExprBin(node))),
			Syntax::ExprBlock => Some(Self::Block(ExprBlock(node))),
			Syntax::ExprField => Some(Self::Field(ExprField(node))),
			Syntax::ExprGroup => Some(Self::Group(ExprGroup(node))),
			Syntax::ExprIdent => Some(Self::Ident(ExprIdent(node))),
			Syntax::ExprLit => Some(Self::Literal(ExprLit(node))),
			Syntax::ExprPostfix => Some(Self::Postfix(ExprPostfix(node))),
			Syntax::ExprPrefix => Some(Self::Prefix(ExprPrefix(node))),
			Syntax::ExprType => ExprType::cast(node).map(Self::Type),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Binary(inner) => inner.syntax(),
			Self::Block(inner) => inner.syntax(),
			Self::Field(inner) => inner.syntax(),
			Self::Group(inner) => inner.syntax(),
			Self::Ident(inner) => inner.syntax(),
			Self::Literal(inner) => inner.syntax(),
			Self::Postfix(inner) => inner.syntax(),
			Self::Prefix(inner) => inner.syntax(),
			Self::Type(inner) => inner.syntax(),
		}
	}
}

/// A subset of [`Expr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum PrimaryExpr {
	Block(ExprBlock),
	Group(ExprGroup),
	Ident(ExprIdent),
	Literal(ExprLit),
	Field(ExprField),
	Postfix(ExprPostfix),
}

impl From<PrimaryExpr> for Expr {
	fn from(value: PrimaryExpr) -> Self {
		match value {
			PrimaryExpr::Block(inner) => Self::Block(inner),
			PrimaryExpr::Group(inner) => Self::Group(inner),
			PrimaryExpr::Ident(inner) => Self::Ident(inner),
			PrimaryExpr::Literal(inner) => Self::Literal(inner),
			PrimaryExpr::Field(inner) => Self::Field(inner),
			PrimaryExpr::Postfix(inner) => Self::Postfix(inner),
		}
	}
}

impl AstNode for PrimaryExpr {
	type Language = Syntax;

	fn can_cast(kind: Syntax) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syntax::ExprBlock
				| Syntax::ExprField
				| Syntax::ExprGroup
				| Syntax::ExprIdent
				| Syntax::ExprLit
				| Syntax::ExprPostfix
		)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syntax::ExprBlock => Some(Self::Block(ExprBlock(node))),
			Syntax::ExprField => Some(Self::Field(ExprField(node))),
			Syntax::ExprGroup => Some(Self::Group(ExprGroup(node))),
			Syntax::ExprIdent => Some(Self::Ident(ExprIdent(node))),
			Syntax::ExprLit => Some(Self::Literal(ExprLit(node))),
			Syntax::ExprPostfix => Some(Self::Postfix(ExprPostfix(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Block(inner) => inner.syntax(),
			Self::Field(inner) => inner.syntax(),
			Self::Group(inner) => inner.syntax(),
			Self::Ident(inner) => inner.syntax(),
			Self::Literal(inner) => inner.syntax(),
			Self::Postfix(inner) => inner.syntax(),
		}
	}
}

// Binary //////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ExprBin`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprBin(pub(super) SyntaxNode);

simple_astnode!(Syntax, ExprBin, Syntax::ExprBin);

impl ExprBin {
	#[must_use]
	pub fn left(&self) -> Expr {
		Expr::cast(self.0.first_child().unwrap()).unwrap()
	}

	pub fn right(&self) -> AstResult<Expr> {
		let lhs = self.0.first_child().unwrap();
		let rhs = self.0.last_child().unwrap();

		if rhs.index() == lhs.index() {
			return Err(AstError::Missing);
		}

		Expr::cast(rhs).ok_or(AstError::Incorrect)
	}

	pub fn operator(&self) -> AstResult<BinOp> {
		let op = self
			.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|t| t.kind().is_glyph()))
			.unwrap();

		if op.kind() == Syntax::At {
			let ident = op.next_token().ok_or(AstError::Missing)?;

			return Ok(BinOp::User { at: op, ident });
		}

		let ret = match op.kind() {
			Syntax::Ampersand => BinOp::Ampersand(op),
			Syntax::Ampersand2 => BinOp::Ampersand2(op),
			Syntax::Ampersand2Eq => BinOp::Ampersand2Eq(op),
			Syntax::AmpersandEq => BinOp::AmpersandEq(op),
			Syntax::AngleL => BinOp::AngleL(op),
			Syntax::AngleL2 => BinOp::AngleL2(op),
			Syntax::AngleL2Eq => BinOp::AngleL2Eq(op),
			Syntax::AngleLEq => BinOp::AngleLEq(op),
			Syntax::AngleR => BinOp::AngleR(op),
			Syntax::AngleR2 => BinOp::AngleR2(op),
			Syntax::AngleR2Eq => BinOp::AngleR2Eq(op),
			Syntax::AngleREq => BinOp::AngleREq(op),
			Syntax::Asterisk => BinOp::Asterisk(op),
			Syntax::Asterisk2 => BinOp::Asterisk2(op),
			Syntax::Asterisk2Eq => BinOp::Asterisk2Eq(op),
			Syntax::AsteriskEq => BinOp::AsteriskEq(op),
			Syntax::At => BinOp::At(op),
			Syntax::Bang => BinOp::Bang(op),
			Syntax::BangEq => BinOp::BangEq(op),
			Syntax::Caret => BinOp::Caret(op),
			Syntax::CaretEq => BinOp::CaretEq(op),
			Syntax::Eq => BinOp::Eq(op),
			Syntax::Eq2 => BinOp::Eq2(op),
			Syntax::Minus => BinOp::Minus(op),
			Syntax::MinusEq => BinOp::MinusEq(op),
			Syntax::Percent => BinOp::Percent(op),
			Syntax::PercentEq => BinOp::PercentEq(op),
			Syntax::Pipe => BinOp::Pipe(op),
			Syntax::Pipe2 => BinOp::Pipe2(op),
			Syntax::Pipe2Eq => BinOp::Pipe2Eq(op),
			Syntax::PipeEq => BinOp::PipeEq(op),
			Syntax::Plus => BinOp::Plus(op),
			Syntax::Plus2 => BinOp::Plus2(op),
			Syntax::Plus2Eq => BinOp::Plus2Eq(op),
			Syntax::PlusEq => BinOp::PlusEq(op),
			Syntax::Slash => BinOp::Slash(op),
			Syntax::SlashEq => BinOp::SlashEq(op),
			Syntax::Tilde => BinOp::Tilde(op),
			Syntax::TildeEq2 => BinOp::TildeEq2(op),
			_ => unreachable!(),
		};

		Ok(ret)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum BinOp {
	Ampersand(SyntaxToken),
	Ampersand2(SyntaxToken),
	Ampersand2Eq(SyntaxToken),
	AmpersandEq(SyntaxToken),
	AngleL(SyntaxToken),
	AngleL2(SyntaxToken),
	AngleL2Eq(SyntaxToken),
	AngleLEq(SyntaxToken),
	AngleR(SyntaxToken),
	AngleR2(SyntaxToken),
	AngleR2Eq(SyntaxToken),
	AngleREq(SyntaxToken),
	Asterisk(SyntaxToken),
	Asterisk2(SyntaxToken),
	Asterisk2Eq(SyntaxToken),
	AsteriskEq(SyntaxToken),
	At(SyntaxToken),
	Bang(SyntaxToken),
	BangEq(SyntaxToken),
	Caret(SyntaxToken),
	CaretEq(SyntaxToken),
	Eq(SyntaxToken),
	Eq2(SyntaxToken),
	Minus(SyntaxToken),
	MinusEq(SyntaxToken),
	Percent(SyntaxToken),
	PercentEq(SyntaxToken),
	Pipe(SyntaxToken),
	Pipe2(SyntaxToken),
	Pipe2Eq(SyntaxToken),
	PipeEq(SyntaxToken),
	Plus(SyntaxToken),
	Plus2(SyntaxToken),
	Plus2Eq(SyntaxToken),
	PlusEq(SyntaxToken),
	Slash(SyntaxToken),
	SlashEq(SyntaxToken),
	Tilde(SyntaxToken),
	TildeEq2(SyntaxToken),
	User { at: SyntaxToken, ident: SyntaxToken },
}

// Block ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ExprBlock`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprBlock(SyntaxNode);

simple_astnode!(Syntax, ExprBlock, Syntax::ExprBlock);

impl ExprBlock {
	pub fn innards(&self) -> impl Iterator<Item = TopLevel> {
		self.0.children().filter_map(TopLevel::cast)
	}
}

// Field ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ExprField`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprField(pub(super) SyntaxNode);

simple_astnode!(Syntax, ExprField, Syntax::ExprField);

impl ExprField {
	#[must_use]
	pub fn left(&self) -> PrimaryExpr {
		PrimaryExpr::cast(self.0.first_child().unwrap()).unwrap()
	}

	pub fn right(&self) -> AstResult<Name> {
		let ret = self.0.last_token().unwrap();

		match ret.kind() {
			Syntax::Ident | Syntax::LitName => Ok(Name(ret)),
			Syntax::Dot => Err(AstError::Missing),
			_ => Err(AstError::Incorrect),
		}
	}
}

// Group ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ExprGroup`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprGroup(pub(super) SyntaxNode);

simple_astnode!(Syntax, ExprGroup, Syntax::ExprGroup);

impl ExprGroup {
	#[must_use]
	pub fn inner(&self) -> Expr {
		Expr::cast(self.0.first_child().unwrap()).unwrap()
	}
}

// Ident ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ExprIdent`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprIdent(pub(super) SyntaxNode);

simple_astnode!(Syntax, ExprIdent, Syntax::ExprIdent);

impl ExprIdent {
	#[must_use]
	pub fn token(&self) -> SyntaxToken {
		let ret = self.0.first_token().unwrap();
		debug_assert_eq!(ret.kind(), Syntax::Ident);
		ret
	}
}

// Literal /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ExprLit`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprLit(pub(super) SyntaxNode);

simple_astnode!(Syntax, ExprLit, Syntax::ExprLit);

impl ExprLit {
	#[must_use]
	pub fn token(&self) -> LitToken {
		LitToken(self.0.first_token().unwrap())
	}

	/// The returned token is always tagged [`Syntax::Ident`].
	#[must_use]
	pub fn string_suffix(&self) -> Option<SyntaxToken> {
		let lit = self.0.first_token().unwrap();
		let suffix = self.0.last_token().unwrap();

		if lit.kind() != Syntax::LitString {
			return None;
		}

		if suffix.kind() != Syntax::Ident {
			return None;
		}

		if suffix.index() != lit.index() + 1 {
			return None;
		}

		Some(suffix)
	}
}

// Postfix /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ExprPostfix`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprPostfix(pub(super) SyntaxNode);

simple_astnode!(Syntax, ExprPostfix, Syntax::ExprPostfix);

// Prefix //////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ExprPrefix`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprPrefix(pub(super) SyntaxNode);

simple_astnode!(Syntax, ExprPrefix, Syntax::ExprPrefix);

impl ExprPrefix {
	pub fn operand(&self) -> AstResult<Expr> {
		Expr::cast(self.0.last_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}

	#[must_use]
	pub fn operator(&self) -> PrefixOp {
		let ret = self.0.first_token().unwrap();

		match ret.kind() {
			Syntax::Bang => PrefixOp::Bang(ret),
			Syntax::Minus => PrefixOp::Minus(ret),
			Syntax::Tilde => PrefixOp::Tilde(ret),
			other => unreachable!("unexpected prefix op kind {other:?}"),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum PrefixOp {
	Bang(SyntaxToken),
	Minus(SyntaxToken),
	Tilde(SyntaxToken),
}

// Type ////////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ExprType`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprType(pub(super) SyntaxNode);

simple_astnode!(Syntax, ExprType, Syntax::ExprType);

impl ExprType {
	pub fn prefixes(&self) -> impl Iterator<Item = TypePrefix> {
		self.0.children().filter_map(TypePrefix::cast)
	}

	pub fn inner(&self) -> AstResult<Expr> {
		Expr::cast(self.0.last_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum TypePrefix {
	Array(ArrayPrefix),
}

impl AstNode for TypePrefix {
	type Language = Syntax;

	fn can_cast(kind: Syntax) -> bool
	where
		Self: Sized,
	{
		matches!(kind, Syntax::ArrayPrefix)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syntax::ArrayPrefix => Some(Self::Array(ArrayPrefix(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Array(inner) => inner.syntax(),
		}
	}
}

/// Wraps a node tagged [`Syntax::ArrayPrefix`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ArrayPrefix(pub(super) SyntaxNode);

simple_astnode!(Syntax, ArrayPrefix, Syntax::ArrayPrefix);

impl ArrayPrefix {
	pub fn length(&self) -> AstResult<Expr> {
		Expr::cast(self.0.first_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}
}
