//! VZScript Intermediate Representation.
//!
//! A semantically-rich, high level common middle between VZScript's transpilation
//! inputs and the backend.

use util::rstring::RString;

use crate::compile::intern::SymbolIx;

#[derive(Debug, Default, Clone)]
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

impl From<Node> for Block {
	fn from(value: Node) -> Self {
		Self(vec![value])
	}
}

#[derive(Debug, Clone)]
pub(crate) enum Node {
	Aggregate(Vec<NodeIx>),
	Assign {
		name: RString,
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
		function: SymbolIx,
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
pub(crate) enum Immediate {
	I8(i8),
	I16(i16),
	I32(i32),
	I64(i64),
	I128(i128),
	F32(f32),
	F64(f64),
	F32X4(f32, f32, f32, f32),
}

impl Immediate {
	#[must_use]
	#[cfg(target_pointer_width = "32")]
	pub(crate) fn pointer(addr: i32) -> Self {
		Self::I32(addr)
	}

	#[must_use]
	#[cfg(target_pointer_width = "64")]
	pub(crate) fn pointer(addr: i64) -> Self {
		Self::I64(addr)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnaryOp {
	BitNegate,
	Negate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NodeIx(u32);
