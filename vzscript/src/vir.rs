//! VZScript Intermediate Representation.
//!
//! A semantically-rich, high level common middle between VZScript's transpilation
//! inputs and the backend.

use crate::compile::{NameKey, SymbolKey};

#[derive(Debug)]
pub(crate) struct Block(pub(crate) Vec<Node>);

impl std::ops::Deref for Block {
	type Target = Vec<Node>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl std::ops::DerefMut for Block {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

#[derive(Debug)]
pub(crate) enum Node {
	Aggregate(Vec<NodeIx>),
	Assign {
		name: NameKey,
		expr: NodeIx,
	},
	Bin {
		lhs: NodeIx,
		rhs: NodeIx,
		op: BinOp,
	},
	Block(Block),
	Branch {
		cond: NodeIx,
		if_true: NodeIx,
		if_false: NodeIx,
	},
	Break(/* ??? */),
	Call {
		function: SymbolKey,
		args: Vec<NodeIx>,
	},
	Continue(/* ??? */),
	/// An infinite loop.
	Loop(Block),
	Immediate(Immediate),
	Ret(NodeIx),
	Unary {
		operand: NodeIx,
		op: UnaryOp,
	},
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BinOp {
	Add,
	BitAnd,
	BitOr,
	Sub,
	Div,
	Max,
	Min,
	Remainder,
	ShiftL,
	ShiftR,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Immediate {
	I8(i8),
	I16(i16),
	I32(i32),
	I64(i64),
	F32(f32),
	F64(f64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnaryOp {
	BitNegate,
	Negate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NodeIx(u32);
