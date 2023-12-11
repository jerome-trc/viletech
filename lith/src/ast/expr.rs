//! Abstract syntax tree nodes for representing expressions.

use doomfront::{
	rowan::{ast::AstNode, Direction},
	simple_astnode, AstError, AstResult,
};

use crate::{Syn, SyntaxNode, SyntaxToken};

use super::{Annotation, ArgList, CoreElement, Item, LitToken, Name, TypeSpec};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Expr {
	Aggregate(ExprAggregate),
	Binary(ExprBin),
	Block(ExprBlock),
	Call(ExprCall),
	Construct(ExprConstruct),
	Field(ExprField),
	Group(ExprGroup),
	Ident(ExprIdent),
	Index(ExprIndex),
	Literal(ExprLit),
	Postfix(ExprPostfix),
	Prefix(ExprPrefix),
	Struct(ExprStruct),
	Type(ExprType),
}

impl AstNode for Expr {
	type Language = Syn;

	fn can_cast(kind: Syn) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syn::ExprAggregate
				| Syn::ExprBin | Syn::ExprBlock
				| Syn::ExprCall | Syn::ExprConstruct
				| Syn::ExprField | Syn::ExprGroup
				| Syn::ExprIdent | Syn::ExprIndex
				| Syn::ExprLit | Syn::ExprPostfix
				| Syn::ExprPrefix
				| Syn::ExprStruct
				| Syn::ExprType
		)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::ExprAggregate => Some(Self::Aggregate(ExprAggregate(node))),
			Syn::ExprBin => Some(Self::Binary(ExprBin(node))),
			Syn::ExprBlock => Some(Self::Block(ExprBlock(node))),
			Syn::ExprCall => Some(Self::Call(ExprCall(node))),
			Syn::ExprConstruct => Some(Self::Construct(ExprConstruct(node))),
			Syn::ExprField => Some(Self::Field(ExprField(node))),
			Syn::ExprGroup => Some(Self::Group(ExprGroup(node))),
			Syn::ExprIdent => Some(Self::Ident(ExprIdent(node))),
			Syn::ExprIndex => Some(Self::Index(ExprIndex(node))),
			Syn::ExprLit => Some(Self::Literal(ExprLit(node))),
			Syn::ExprPostfix => Some(Self::Postfix(ExprPostfix(node))),
			Syn::ExprPrefix => Some(Self::Prefix(ExprPrefix(node))),
			Syn::ExprStruct => Some(Self::Struct(ExprStruct(node))),
			Syn::ExprType => ExprType::cast(node).map(Self::Type),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Aggregate(inner) => inner.syntax(),
			Self::Call(inner) => inner.syntax(),
			Self::Construct(inner) => inner.syntax(),
			Self::Binary(inner) => inner.syntax(),
			Self::Block(inner) => inner.syntax(),
			Self::Field(inner) => inner.syntax(),
			Self::Group(inner) => inner.syntax(),
			Self::Ident(inner) => inner.syntax(),
			Self::Index(inner) => inner.syntax(),
			Self::Literal(inner) => inner.syntax(),
			Self::Postfix(inner) => inner.syntax(),
			Self::Prefix(inner) => inner.syntax(),
			Self::Struct(inner) => inner.syntax(),
			Self::Type(inner) => inner.syntax(),
		}
	}
}

/// A subset of [`Expr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum PrimaryExpr {
	Aggregate(ExprAggregate),
	Block(ExprBlock),
	Call(ExprCall),
	Group(ExprGroup),
	Ident(ExprIdent),
	Index(ExprIndex),
	Literal(ExprLit),
	Field(ExprField),
	Postfix(ExprPostfix),
	Struct(ExprStruct),
}

impl From<PrimaryExpr> for Expr {
	fn from(value: PrimaryExpr) -> Self {
		match value {
			PrimaryExpr::Aggregate(inner) => Self::Aggregate(inner),
			PrimaryExpr::Block(inner) => Self::Block(inner),
			PrimaryExpr::Call(inner) => Self::Call(inner),
			PrimaryExpr::Group(inner) => Self::Group(inner),
			PrimaryExpr::Ident(inner) => Self::Ident(inner),
			PrimaryExpr::Index(inner) => Self::Index(inner),
			PrimaryExpr::Literal(inner) => Self::Literal(inner),
			PrimaryExpr::Field(inner) => Self::Field(inner),
			PrimaryExpr::Postfix(inner) => Self::Postfix(inner),
			PrimaryExpr::Struct(inner) => Self::Struct(inner),
		}
	}
}

impl AstNode for PrimaryExpr {
	type Language = Syn;

	fn can_cast(kind: Syn) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syn::ExprBlock
				| Syn::ExprCall | Syn::ExprField
				| Syn::ExprGroup | Syn::ExprIdent
				| Syn::ExprIndex | Syn::ExprLit
				| Syn::ExprPostfix
				| Syn::ExprStruct
		)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::ExprAggregate => Some(Self::Aggregate(ExprAggregate(node))),
			Syn::ExprBlock => Some(Self::Block(ExprBlock(node))),
			Syn::ExprCall => Some(Self::Call(ExprCall(node))),
			Syn::ExprField => Some(Self::Field(ExprField(node))),
			Syn::ExprGroup => Some(Self::Group(ExprGroup(node))),
			Syn::ExprIdent => Some(Self::Ident(ExprIdent(node))),
			Syn::ExprIndex => Some(Self::Index(ExprIndex(node))),
			Syn::ExprLit => Some(Self::Literal(ExprLit(node))),
			Syn::ExprPostfix => Some(Self::Postfix(ExprPostfix(node))),
			Syn::ExprStruct => Some(Self::Struct(ExprStruct(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Aggregate(inner) => inner.syntax(),
			Self::Block(inner) => inner.syntax(),
			Self::Call(inner) => inner.syntax(),
			Self::Field(inner) => inner.syntax(),
			Self::Group(inner) => inner.syntax(),
			Self::Ident(inner) => inner.syntax(),
			Self::Index(inner) => inner.syntax(),
			Self::Literal(inner) => inner.syntax(),
			Self::Postfix(inner) => inner.syntax(),
			Self::Struct(inner) => inner.syntax(),
		}
	}
}

// Aggregate ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ExprAggregate`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprAggregate(SyntaxNode);

simple_astnode!(Syn, ExprAggregate, Syn::ExprAggregate);

impl ExprAggregate {
	pub fn initializers(&self) -> impl Iterator<Item = AggregateInit> {
		self.0.children().filter_map(AggregateInit::cast)
	}
}

/// Wraps a node tagged [`Syn::AggregateInit`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum AggregateInit {
	Anon(AnonInit),
	Field(FieldInit),
	Index(IndexInit),
}

impl AstNode for AggregateInit {
	type Language = Syn;

	fn can_cast(kind: Syn) -> bool
	where
		Self: Sized,
	{
		kind == Syn::AggregateInit
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		if node.kind() != Syn::AggregateInit {
			return None;
		}

		match node.first_token().map(|t| t.kind()) {
			Some(Syn::Dot) => Some(Self::Field(FieldInit(node))),
			Some(Syn::BracketL) => Some(Self::Index(IndexInit(node))),
			Some(_) => Some(Self::Anon(AnonInit(node))),
			None => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Anon(inner) => &inner.0,
			Self::Field(inner) => &inner.0,
			Self::Index(inner) => &inner.0,
		}
	}
}

/// Wraps a node tagged [`Syn::AggregateInit`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AnonInit(SyntaxNode);

impl AnonInit {
	pub fn expr(&self) -> AstResult<Expr> {
		Expr::cast(self.0.first_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}
}

/// Wraps a node tagged [`Syn::AggregateInit`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FieldInit(SyntaxNode);

impl FieldInit {
	pub fn name(&self) -> AstResult<Name> {
		let dot = self.0.first_token().unwrap();
		debug_assert_eq!(dot.kind(), Syn::Dot);

		dot.siblings_with_tokens(Direction::Next)
			.find_map(|elem| {
				elem.into_token()
					.filter(|t| matches!(t.kind(), Syn::Ident | Syn::LitName))
			})
			.map(Name)
			.ok_or(AstError::Missing)
	}

	pub fn expr(&self) -> AstResult<Expr> {
		Expr::cast(self.0.last_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}
}

/// Wraps a node tagged [`Syn::AggregateInit`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct IndexInit(SyntaxNode);

impl IndexInit {
	pub fn index(&self) -> AstResult<Expr> {
		self.eq_token()?
			.siblings_with_tokens(Direction::Prev)
			.filter_map(|elem| elem.into_node())
			.find_map(Expr::cast)
			.ok_or(AstError::Missing)
	}

	pub fn expr(&self) -> AstResult<Expr> {
		self.eq_token()?
			.siblings_with_tokens(Direction::Next)
			.filter_map(|elem| elem.into_node())
			.find_map(Expr::cast)
			.ok_or(AstError::Missing)
	}

	fn eq_token(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|t| t.kind() == Syn::Eq))
			.ok_or(AstError::Incorrect)
	}
}

// Binary //////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ExprBin`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprBin(SyntaxNode);

simple_astnode!(Syn, ExprBin, Syn::ExprBin);

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

		if op.kind() == Syn::At {
			let ident = op.next_token().ok_or(AstError::Missing)?;

			return Ok(BinOp::User { at: op, ident });
		}

		let ret = match op.kind() {
			Syn::Ampersand => BinOp::Ampersand(op),
			Syn::Ampersand2 => BinOp::Ampersand2(op),
			Syn::Ampersand2Eq => BinOp::Ampersand2Eq(op),
			Syn::AmpersandEq => BinOp::AmpersandEq(op),
			Syn::AngleL => BinOp::AngleL(op),
			Syn::AngleL2 => BinOp::AngleL2(op),
			Syn::AngleL2Eq => BinOp::AngleL2Eq(op),
			Syn::AngleLEq => BinOp::AngleLEq(op),
			Syn::AngleR => BinOp::AngleR(op),
			Syn::AngleR2 => BinOp::AngleR2(op),
			Syn::AngleR2Eq => BinOp::AngleR2Eq(op),
			Syn::AngleREq => BinOp::AngleREq(op),
			Syn::Asterisk => BinOp::Asterisk(op),
			Syn::Asterisk2 => BinOp::Asterisk2(op),
			Syn::Asterisk2Eq => BinOp::Asterisk2Eq(op),
			Syn::AsteriskEq => BinOp::AsteriskEq(op),
			Syn::At => BinOp::At(op),
			Syn::Bang => BinOp::Bang(op),
			Syn::BangEq => BinOp::BangEq(op),
			Syn::Caret => BinOp::Caret(op),
			Syn::CaretEq => BinOp::CaretEq(op),
			Syn::Eq => BinOp::Eq(op),
			Syn::Eq2 => BinOp::Eq2(op),
			Syn::Minus => BinOp::Minus(op),
			Syn::MinusEq => BinOp::MinusEq(op),
			Syn::Percent => BinOp::Percent(op),
			Syn::PercentEq => BinOp::PercentEq(op),
			Syn::Pipe => BinOp::Pipe(op),
			Syn::Pipe2 => BinOp::Pipe2(op),
			Syn::Pipe2Eq => BinOp::Pipe2Eq(op),
			Syn::PipeEq => BinOp::PipeEq(op),
			Syn::Plus => BinOp::Plus(op),
			Syn::Plus2 => BinOp::Plus2(op),
			Syn::Plus2Eq => BinOp::Plus2Eq(op),
			Syn::PlusEq => BinOp::PlusEq(op),
			Syn::Slash => BinOp::Slash(op),
			Syn::SlashEq => BinOp::SlashEq(op),
			Syn::Tilde => BinOp::Tilde(op),
			Syn::TildeEq2 => BinOp::TildeEq2(op),
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

/// Wraps a node tagged [`Syn::ExprBlock`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprBlock(SyntaxNode);

simple_astnode!(Syn, ExprBlock, Syn::ExprBlock);

impl ExprBlock {
	pub fn innards(&self) -> impl Iterator<Item = CoreElement> {
		self.0.children().filter_map(CoreElement::cast)
	}
}

// Call ////////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ExprCall`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprCall(SyntaxNode);

simple_astnode!(Syn, ExprCall, Syn::ExprCall);

impl ExprCall {
	#[must_use]
	pub fn called(&self) -> PrimaryExpr {
		PrimaryExpr::cast(self.0.first_child().unwrap()).unwrap()
	}

	pub fn arg_list(&self) -> AstResult<ArgList> {
		ArgList::cast(self.0.last_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}
}

// Construct ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ExprConstruct`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprConstruct(SyntaxNode);

simple_astnode!(Syn, ExprConstruct, Syn::ExprConstruct);

impl ExprConstruct {
	pub fn aggregate_type(&self) -> AstResult<PrimaryExpr> {
		PrimaryExpr::cast(self.0.first_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}

	pub fn initializers(&self) -> impl Iterator<Item = AggregateInit> {
		self.0.children().filter_map(AggregateInit::cast)
	}
}

// Field ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ExprField`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprField(SyntaxNode);

simple_astnode!(Syn, ExprField, Syn::ExprField);

impl ExprField {
	#[must_use]
	pub fn left(&self) -> PrimaryExpr {
		PrimaryExpr::cast(self.0.first_child().unwrap()).unwrap()
	}

	pub fn right(&self) -> AstResult<Name> {
		let ret = self.0.last_token().unwrap();

		match ret.kind() {
			Syn::Ident | Syn::LitName => Ok(Name(ret)),
			Syn::Dot => Err(AstError::Missing),
			_ => Err(AstError::Incorrect),
		}
	}
}

// Group ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ExprGroup`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprGroup(SyntaxNode);

simple_astnode!(Syn, ExprGroup, Syn::ExprGroup);

impl ExprGroup {
	#[must_use]
	pub fn inner(&self) -> Expr {
		Expr::cast(self.0.first_child().unwrap()).unwrap()
	}
}

// Ident ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ExprIdent`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprIdent(SyntaxNode);

simple_astnode!(Syn, ExprIdent, Syn::ExprIdent);

impl ExprIdent {
	#[must_use]
	pub fn token(&self) -> SyntaxToken {
		let ret = self.0.first_token().unwrap();
		debug_assert_eq!(ret.kind(), Syn::Ident);
		ret
	}
}

// Index ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ExprIndex`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprIndex(SyntaxNode);

simple_astnode!(Syn, ExprIndex, Syn::ExprIndex);

impl ExprIndex {
	#[must_use]
	pub fn called(&self) -> PrimaryExpr {
		PrimaryExpr::cast(self.0.first_child().unwrap()).unwrap()
	}

	pub fn index(&self) -> AstResult<Expr> {
		let last = self.0.last_child().ok_or(AstError::Missing)?;
		let first = self.0.first_child().ok_or(AstError::Missing)?;

		if last.index() != (first.index() + 1) {
			return Err(AstError::Missing);
		}

		Expr::cast(last).ok_or(AstError::Incorrect)
	}
}

// Literal /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ExprLit`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprLit(SyntaxNode);

simple_astnode!(Syn, ExprLit, Syn::ExprLit);

impl ExprLit {
	#[must_use]
	pub fn token(&self) -> LitToken {
		LitToken(self.0.first_token().unwrap())
	}

	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn string_suffix(&self) -> Option<SyntaxToken> {
		let lit = self.0.first_token().unwrap();
		let suffix = self.0.last_token().unwrap();

		if lit.kind() != Syn::LitString {
			return None;
		}

		if suffix.kind() != Syn::Ident {
			return None;
		}

		if suffix.index() != lit.index() + 1 {
			return None;
		}

		Some(suffix)
	}
}

// Postfix /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ExprPostfix`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprPostfix(SyntaxNode);

simple_astnode!(Syn, ExprPostfix, Syn::ExprPostfix);

// Prefix //////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ExprPrefix`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprPrefix(SyntaxNode);

simple_astnode!(Syn, ExprPrefix, Syn::ExprPrefix);

impl ExprPrefix {
	pub fn operand(&self) -> AstResult<Expr> {
		Expr::cast(self.0.last_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}

	#[must_use]
	pub fn operator(&self) -> PrefixOp {
		let ret = self.0.first_token().unwrap();

		match ret.kind() {
			Syn::Bang => PrefixOp::Bang(ret),
			Syn::Minus => PrefixOp::Minus(ret),
			Syn::Tilde => PrefixOp::Tilde(ret),
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

// Struct //////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ExprStruct`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprStruct(SyntaxNode);

simple_astnode!(Syn, ExprStruct, Syn::ExprStruct);

impl ExprStruct {
	pub fn innards(&self) -> impl Iterator<Item = StructInnard> {
		self.0.children().filter_map(StructInnard::cast)
	}
}

/// Wraps a node tagged [`Syn::FieldDecl`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FieldDecl(SyntaxNode);

simple_astnode!(Syn, FieldDecl, Syn::FieldDecl);

impl FieldDecl {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		let ret = self.0.first_token().unwrap();
		debug_assert_eq!(ret.kind(), Syn::Ident);
		ret
	}

	#[must_use]
	pub fn type_spec(&self) -> TypeSpec {
		TypeSpec::cast(self.0.last_child().unwrap()).unwrap()
	}
}

/// Anything that can inhabit a [struct expression](ExprStruct).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StructInnard {
	Annotation(Annotation),
	Field(FieldDecl),
	Item(Item),
}

impl AstNode for StructInnard {
	type Language = Syn;

	fn can_cast(kind: Syn) -> bool
	where
		Self: Sized,
	{
		if Item::can_cast(kind) {
			return true;
		}

		matches!(kind, Syn::Annotation | Syn::FieldDecl)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		if let Some(item) = Item::cast(node.clone()) {
			return Some(Self::Item(item));
		}

		match node.kind() {
			Syn::Annotation => Some(Self::Annotation(Annotation(node))),
			Syn::FieldDecl => Some(Self::Field(FieldDecl(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Annotation(inner) => inner.syntax(),
			Self::Field(inner) => inner.syntax(),
			Self::Item(inner) => inner.syntax(),
		}
	}
}

// Type ////////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ExprType`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprType(SyntaxNode);

simple_astnode!(Syn, ExprType, Syn::ExprType);

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
	type Language = Syn;

	fn can_cast(kind: Syn) -> bool
	where
		Self: Sized,
	{
		matches!(kind, Syn::ArrayPrefix)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::ArrayPrefix => Some(Self::Array(ArrayPrefix(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Array(inner) => inner.syntax(),
		}
	}
}

/// Wraps a node tagged [`Syn::ArrayPrefix`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ArrayPrefix(SyntaxNode);

simple_astnode!(Syn, ArrayPrefix, Syn::ArrayPrefix);

impl ArrayPrefix {
	pub fn length(&self) -> AstResult<Expr> {
		Expr::cast(self.0.first_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}
}
