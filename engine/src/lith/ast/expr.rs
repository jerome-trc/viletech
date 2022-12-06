//! Expressions and operators.

/*

Copyright (C) 2022 ***REMOVED***

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

use crate::utils::lang::{FileSpan, Identifier};

use super::{literal::Literal, Resolver};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub enum PrefixOp {
	Plus,
	Minus,
	Increment,
	Decrement,
	LogicalNot,
	BitwiseNot,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub enum PostfixOp {
	Increment,
	Decrement,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Expression<'inp> {
	pub span: Option<FileSpan<'inp>>,
	#[serde(flatten)]
	pub kind: ExpressionKind<'inp>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum ExpressionKind<'inp> {
	Ident(Identifier<'inp>),
	Literal(Literal<'inp>),
	Type(TypeExpr<'inp>),
	Binary {
		op: BinaryOp,
		exprs: Box<BinaryOpExprs<'inp>>,
	},
	Prefix {
		op: PrefixOp,
		expr: Box<Expression<'inp>>,
	},
	Postfix {
		op: PostfixOp,
		expr: Box<Expression<'inp>>,
	},
	Ternary(Box<TernaryOpExprs<'inp>>),
	ArrayIndex(Box<ArrayIndexExprs<'inp>>),
	Call {
		lhs: Box<Expression<'inp>>,
		exprs: Vec<FunctionCallArg<'inp>>,
	},
	Array(Box<ArrayExpr<'inp>>),
	Structure(Box<StructExpr<'inp>>),
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ExprList<'inp> {
	pub span: FileSpan<'inp>,
	pub exprs: Vec1<Expression<'inp>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TypeExpr<'inp> {
	pub span: FileSpan<'inp>,
	#[serde(flatten)]
	pub kind: TypeExprKind<'inp>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum TypeExprKind<'inp> {
	Anonymous,
	Resolver(Resolver<'inp>),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BinaryOpExprs<'inp> {
	pub lhs: Expression<'inp>,
	pub rhs: Expression<'inp>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TernaryOpExprs<'inp> {
	pub cond: Expression<'inp>,
	pub if_true: Expression<'inp>,
	pub if_false: Expression<'inp>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArrayIndexExprs<'inp> {
	pub lhs: Expression<'inp>,
	pub index: Expression<'inp>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FunctionCallArg<'inp> {
	pub span: FileSpan<'inp>,
	#[serde(flatten)]
	pub kind: FunctionCallArgKind<'inp>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum FunctionCallArgKind<'inp> {
	Unnamed(Expression<'inp>),
	Named(Identifier<'inp>, Expression<'inp>),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArrayExpr<'inp> {
	pub exprs: Vec<Expression<'inp>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StructExpr<'inp> {
	pub inits: Vec<(Identifier<'inp>, Expression<'inp>)>,
}
