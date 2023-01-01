//! Expressions and operators.

use serde::Serialize;
use vec1::Vec1;

use crate::utils::lang::{Identifier, Span};

use super::{
	literal::Literal, DeclQualifier, FuncParameter, ResolverPart, StatementBlock, TypeExpr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum BinaryOp {
	Add,
	AddAssign,
	Subtract,
	SubtractAssign,
	Multiply,
	MultiplyAssign,
	Divide,
	DivideAssign,
	Modulo,
	ModuloAssign,
	Raise,
	RaiseAssign,
	LeftShift,
	LeftShiftAssign,
	RightShift,
	RightShiftAssign,
	UnsignedRightShift,
	UnsignedRightShiftAssign,
	Concat,
	ConcatAssign,
	LessThan,
	LessThanOrEquals,
	GreaterThan,
	GreaterThanOrEquals,
	Equals,
	NotEquals,
	ApproxEquals,
	ThreeWayComp,
	LogicalAnd,
	LogicalAndAssign,
	LogicalOr,
	LogicalOrAssign,
	LogicalXor,
	LogicalXorAssign,
	BitwiseAnd,
	BitwiseAndAssign,
	BitwiseOr,
	BitwiseOrAssign,
	BitwiseXor,
	BitwiseXorAssign,
	TypeCompare,
	NegativeTypeCompare,
	ScopeRes,
	Assign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PrefixOp {
	AntiNegate,
	Negate,
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
	pub span: Span,
	#[serde(flatten)]
	pub kind: ExpressionKind,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum ExpressionKind {
	Identifier(Identifier),
	Literal(Literal),
	Type(TypeExpr),
	Prefix(Box<PrefixOpExpr>),
	Postfix(Box<PostfixOpExpr>),
	Binary {
		op: BinaryOp,
		exprs: Box<BinaryOpExprs>,
	},
	Ternary(Box<TernaryOpExprs>),
	Field(Box<FieldExpr>),
	Index(Box<ArrayIndexExprs>),
	Call {
		lhs: Box<Expression>,
		args: Vec<CallArg>,
	},
	MethodCall {
		lhs: Box<Expression>,
		method: ResolverPart,
		args: Vec<CallArg>,
	},
	Lambda {
		quals: Vec<DeclQualifier>,
		params: Vec<FuncParameter>,
		return_type: Option<TypeExpr>,
		body: StatementBlock,
	},
	Array(Box<ArrayExpr>),
	Structure(Box<StructExpr>),
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ExprList {
	pub span: Span,
	pub exprs: Vec1<Expression>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PrefixOpExpr {
	pub op: PrefixOp,
	pub expr: Expression,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PostfixOpExpr {
	pub op: PostfixOp,
	pub expr: Expression,
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
pub struct FieldExpr {
	pub owner: Expression,
	pub field: Identifier,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArrayIndexExprs {
	pub lhs: Expression,
	pub index: Expression,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CallArg {
	pub span: Span,
	#[serde(flatten)]
	pub kind: CallArgKind,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum CallArgKind {
	Unnamed(Expression),
	Named { name: Identifier, expr: Expression },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArrayExpr {
	pub exprs: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StructExpr {
	pub inits: Vec<(Identifier, Expression)>,
}
