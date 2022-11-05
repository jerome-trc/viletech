/*

Copyright (C) 2021-2022 Jessica "Gutawer" Russell

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.

*/

use serde::Serialize;
use vec1::Vec1;

use super::{
	interner::{NameSymbol, StringSymbol},
	Span,
};

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Identifier {
	pub span: Span,
	pub symbol: NameSymbol,
}

impl From<Identifier> for NameSymbol {
	fn from(item: Identifier) -> Self {
		item.symbol
	}
}

impl From<&Identifier> for NameSymbol {
	fn from(item: &Identifier) -> Self {
		item.symbol
	}
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct DottableId {
	pub span: Span,
	pub ids: Vec1<Identifier>,
}

#[derive(Serialize, Debug, Clone, Copy, Eq, PartialEq)]
pub struct StringConst {
	pub span: Span,
	pub symbol: StringSymbol,
}

#[derive(Serialize, Debug, Clone, Copy, Eq, PartialEq)]
pub struct NameConst {
	pub span: Span,
	pub symbol: NameSymbol,
}

#[derive(Serialize, Debug, Clone, Copy, Eq, PartialEq)]
pub struct IntConst {
	pub span: Span,
	pub val: u64,
	pub long: bool,
	pub unsigned: bool,
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub struct FloatConst {
	pub span: Span,
	pub val: f64,
	pub double: bool,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "data")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConstKind {
	String(StringConst),
	Name(NameConst),
	Int(IntConst),
	Float(FloatConst),
	Bool(bool),
	Null,
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub struct Const {
	pub span: Span,
	#[serde(flatten)]
	pub kind: ConstKind,
}

#[derive(Serialize, Debug, Clone, Copy, Eq, PartialEq)]
pub struct VersionInfo {
	pub major: u16,
	pub minor: u16,
	pub revision: u16,
}

impl VersionInfo {
	pub const fn new(major: u16, minor: u16, revision: u16) -> Self {
		Self {
			major,
			minor,
			revision,
		}
	}
}

pub(super) fn parse_lump_version(s: &str) -> Option<VersionInfo> {
	fn split_once_inclusive(s: &str, p: impl Fn(char) -> bool) -> (&str, &str) {
		let mut splitter = s.splitn(2, p);
		let first = splitter.next().unwrap();
		let second = &s[first.len()..];
		(first, second)
	}

	let mut s = s;

	s = split_once_inclusive(s, |c| ![' ', '\n', '\t', '\r'].contains(&c)).1;
	let (s0, s1) = split_once_inclusive(s, |c| !('0'..'9').contains(&c));
	s = s1;
	let major = s0.parse::<u16>().ok()?;
	if !s.starts_with('.') {
		return None;
	}
	s = &s[1..];

	s = split_once_inclusive(s, |c| ![' ', '\n', '\t', '\r'].contains(&c)).1;
	let (s0, s1) = split_once_inclusive(s, |c| !('0'..'9').contains(&c));
	s = s1;
	let minor = s0.parse::<u16>().ok()?;

	let revision = if s.starts_with('.') {
		s = &s[1..];
		s = split_once_inclusive(s, |c| ![' ', '\n', '\t', '\r'].contains(&c)).1;
		let (s0, s1) = split_once_inclusive(s, |c| !('0'..'9').contains(&c));
		s = s1;
		s0.parse::<u16>().ok()?
	} else {
		0
	};

	if s.chars().next().is_some() {
		return None;
	}

	Some(VersionInfo::new(major, minor, revision))
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "data")]
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
	Ident(Identifier),
	Const(Const),
	Binary {
		op: BinaryOp,
		exprs: Box<BinaryOpExprs>,
	},
	Prefix {
		op: PrefixOp,
		expr: Box<Expression>,
	},
	Postfix {
		op: PostfixOp,
		expr: Box<Expression>,
	},
	Ternary(Box<TernaryOpExprs>),
	ArrayIndex(Box<ArrayIndexExprs>),
	FunctionCall {
		lhs: Box<Expression>,
		exprs: Vec<FunctionCallArg>,
	},
	Vector2(Box<(Expression, Expression)>),
	Vector3(Box<(Expression, Expression, Expression)>),
	ClassCast(Identifier, Vec<FunctionCallArg>),
	Super,

	Unknown,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Expression {
	pub span: Option<Span>,
	#[serde(flatten)]
	pub kind: ExpressionKind,
}

#[derive(Serialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum BinaryOp {
	Add,
	Subtract,
	Times,
	Divide,
	Modulo,
	Raise,

	LeftShift,
	RightShift,
	UnsignedRightShift,

	CrossProd,
	DotProd,
	Concat,

	LessThan,
	LessThanEquals,
	GreaterThan,
	GreaterThanEquals,
	Equals,
	NotEquals,
	ApproxEquals,
	ThreeWayComp,

	LogicalAnd,
	BitwiseAnd,
	LogicalOr,
	BitwiseOr,
	BitwiseXor,

	Is,
	Scope,
	MemberAccess,

	Assign,
	PlusAssign,
	MinusAssign,
	TimesAssign,
	DivideAssign,
	ModuloAssign,
	LeftShiftAssign,
	RightShiftAssign,
	UnsignedRightShiftAssign,
	BitwiseOrAssign,
	BitwiseAndAssign,
	BitwiseXorAssign,
}

#[derive(Serialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum PrefixOp {
	Plus,
	Minus,
	Increment,
	Decrement,
	LogicalNot,
	BitwiseNot,
	SizeOf,
	AlignOf,
}

#[derive(Serialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum PostfixOp {
	Increment,
	Decrement,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub enum LabeledStatement {
	Default,
	Case(Box<Expression>),
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ExprList {
	pub span: Span,
	pub list: Vec1<Expression>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ArraySizes {
	pub span: Span,
	pub list: Vec1<Option<Expression>>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct BinaryOpExprs {
	pub lhs: Expression,
	pub rhs: Expression,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct TernaryOpExprs {
	pub cond: Expression,
	pub if_true: Expression,
	pub if_false: Expression,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ArrayIndexExprs {
	pub lhs: Expression,
	pub index: Expression,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "data")]
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionCallArgKind {
	Unnamed(Expression),
	Named(Identifier, Expression),
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct FunctionCallArg {
	pub span: Span,
	#[serde(flatten)]
	pub kind: FunctionCallArgKind,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "data")]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum IntTypeKind {
	SByte,
	Byte,
	Short,
	UShort,
	Int,
	UInt,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct IntType {
	pub span: Span,
	#[serde(flatten)]
	pub kind: IntTypeKind,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct EnumVariant {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub name: Identifier,
	pub init: Option<Expression>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct EnumDefinition {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub name: Identifier,
	pub enum_type: Option<IntType>,
	pub variants: Vec<EnumVariant>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ConstDefinition {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub name: Identifier,
	pub expr: Expression,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct FlagDefinition {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub flag_name: Identifier,
	pub var_name: Identifier,
	pub shift: IntConst,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct PropertyDefinition {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub name: Identifier,
	pub vars: Vec1<Identifier>,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "data")]
#[derive(Debug, Clone, PartialEq)]
pub enum DefaultStatementKind {
	Property {
		prop: DottableId,
		vals: Option<ExprList>,
	},
	AddFlag(DottableId),
	RemoveFlag(DottableId),
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct DefaultStatement {
	pub span: Span,
	#[serde(flatten)]
	pub kind: DefaultStatementKind,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct DefaultDefinition {
	pub span: Span,
	pub statements: Vec<DefaultStatement>,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct NonWhitespace {
	pub span: Span,
	pub symbol: NameSymbol,
}
