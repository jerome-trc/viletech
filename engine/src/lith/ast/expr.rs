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

use crate::utils::lang::{Identifier, Span};

use super::{literal::Literal, Resolver};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PrefixOp {
	Plus,
	Minus,
	Increment,
	Decrement,
	LogicalNot,
	BitwiseNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PostfixOp {
	Increment,
	Decrement,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Expression {
	pub span: Option<Span>,
	#[serde(flatten)]
	pub kind: ExpressionKind,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum ExpressionKind {
	Ident(Identifier),
	Literal(Literal),
	Type(TypeExpr),
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
	Call {
		lhs: Box<Expression>,
		exprs: Vec<FunctionCallArg>,
	},
	Array(Box<ArrayExpr>),
	Structure(Box<StructExpr>),
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ExprList {
	pub span: Span,
	pub exprs: Vec1<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TypeExpr {
	pub span: Span,
	#[serde(flatten)]
	pub kind: TypeExprKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum TypeExprKind {
	Anonymous,
	Resolver(Resolver),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BinaryOpExprs {
	pub lhs: Expression,
	pub rhs: Expression,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TernaryOpExprs {
	pub cond: Expression,
	pub if_true: Expression,
	pub if_false: Expression,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArrayIndexExprs {
	pub lhs: Expression,
	pub index: Expression,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FunctionCallArg {
	pub span: Span,
	#[serde(flatten)]
	pub kind: FunctionCallArgKind,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum FunctionCallArgKind {
	Unnamed(Expression),
	Named(Identifier, Expression),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArrayExpr {
	pub exprs: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StructExpr {
	pub inits: Vec<(Identifier, Expression)>,
}
