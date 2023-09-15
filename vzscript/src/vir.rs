//! The VZScript Intermediate Representation.
//!
//! A common middle ground between VZScript's transpilation inputs and the backend.

use cranelift::prelude::{FloatCC, IntCC};
use doomfront::rowan::TextSize;
use smallvec::SmallVec;

use crate::{
	back::AbiType,
	compile::intern::{NameIx, SymbolIx},
	rti,
	tsys::{FuncType, TypeDef, TypeHandle},
	zname::ZName,
};

#[derive(Debug)]
pub(crate) struct Function {
	pub(crate) body: Box<[Node]>,
	pub(crate) vars: Box<[AbiType]>,
	pub(crate) cold: bool,
}

impl std::ops::Index<NodeIx> for Function {
	type Output = Node;

	fn index(&self, index: NodeIx) -> &Self::Output {
		&self.body[index.0 as usize]
	}
}

impl std::ops::Index<&NodeIx> for Function {
	type Output = Node;

	fn index(&self, index: &NodeIx) -> &Self::Output {
		&self.body[index.0 as usize]
	}
}

#[derive(Debug)]
pub(crate) enum Node {
	Arg(usize),
	Assign {
		/// An index into [`Function::vars`].
		var: usize,
		expr: NodeIx,
	},
	// Evaluation only emits a single SSA value.
	Bin {
		lhs: NodeIx,
		rhs: NodeIx,
		op: BinOp,
	},
	BlockOpen {
		cold: bool,
	},
	BlockClose,
	Branch(Branch),
	Break {
		/// If breaking from the containing loop, this is 0.
		levels: usize,
	},
	Call {
		symbol: SymbolRef,
		args: Box<[NodeIx]>,
	},
	CallIndirect {
		typedef: TypeHandle<FuncType>,
		lhs: NodeIx,
		args: Box<[NodeIx]>,
	},
	/// Evaluation never emits any SSA values.
	Continue {
		/// If continuing the containing loop, this is 0.
		levels: usize,
	},
	Data {
		symbol: SymbolRef,
	},
	/// Evaluation only emits a single SSA value.
	Immediate(Immediate),
	/// Evaluation never emits any SSA values.
	Ret(NodeIx),
	// Evaluation only emits a single SSA value.
	Unary {
		operand: NodeIx,
		op: UnaryOp,
	},
}

#[derive(Debug)]
pub(crate) struct Branch {
	/// [`Expr`] to test if non-zero.
	pub(crate) condition: NodeIx,
	/// [`Expr`] to evaluate and emit if [`Self::condition`] is non-zero.
	pub(crate) if_true: NodeIx,
	/// [`Expr`] to evaluate and emit if [`Self::condition`] is zero.
	pub(crate) if_false: NodeIx,
	pub(crate) cold: ColdBranch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColdBranch {
	Neither,
	True,
	False,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BinOp {
	/// Bitwise and.
	BAnd,
	/// Bitwise and not (i.e. `x & ~y`).
	BAndNot,
	/// Bitwise or.
	BOr,
	/// Bitwise or not (i.e. `x | ~y`).
	BOrNot,
	/// Bitwise xor.
	BXor,
	/// Bitwise xor not (i.e. `x ^ ~y`).
	BXorNot,
	/// Floating-point addition.
	FAdd,
	/// Floating-point comparison.
	FCmp(FloatCC),
	/// Floating-point sign copy.
	FCpySign,
	/// Floating-point division.
	FDiv,
	/// Floating-point maximum.
	FMax,
	/// Floating-point minimum.
	FMin,
	/// Floating-point multiplication.
	FMul,
	/// Floating-point subtraction.
	FSub,
	/// Wrapping integer addition.
	IAdd,
	/// Integer comparison.
	ICmp(IntCC),
	/// Concatenate two integers to form a larger one.
	IConcat,
	/// Integer shift left.
	IShl,
	/// Wrapping integer subtraction.
	ISub,
	/// Signed integer addition with saturation.
	SAddSat,
	/// Signed integer addition with overflow.
	SAddOf,
	/// Signed integer division rounded towards zero.
	SDiv,
	/// Signed integer remainder.
	SRem,
	/// Signed integer shift right.
	SShr,
	/// Unsigned integer addition with saturation.
	UAddSat,
	/// Unsigned integer addition with overflow.
	UAddOf,
	/// Unsigned integer division.
	UDiv,
	/// Unsigned integer remainder.
	URem,
	/// Unsigned integer shift right.
	UShr,
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
	F32X2(f32, f32),
	F32X4(f32, f32, f32, f32),
	Address(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnaryOp {
	/// Bitwise not.
	BNot,
	/// Floating point rounding to an integer, towards positive infinity.
	Ceil,
	/// Count leading sign bits.
	Cls,
	/// Count leading zeroes.
	Clz,
	/// Count trailing zeroes.
	Ctz,
	/// Floating-point absolute value.
	FAbs,
	/// Convert a signed integer to a floating-point scalar.
	F32FromSInt,
	/// Convert an unsigned integer to a floating-point scalar.
	F32FromUInt,
	/// Convert a signed integer to a floating-point scalar.
	F64FromSInt,
	/// Convert an unsigned integer to a floating-point scalar.
	F64FromUInt,
	/// Convert a floating-point scalar to a signed integer.
	///
	/// The given type corresponds to the latter.
	FToSInt(AbiType),
	/// Convert a floating-point scalar to an unsigned integer.
	///
	/// The given type corresponds to the latter.
	FToUInt(AbiType),
	/// Convert a floating-point scalar to a smaller float type.
	FDemote(AbiType),
	/// Floating point rounding to an integer, towards negative infinity.
	Floor,
	/// Floating-point negation.
	FNeg,
	/// Convert a floating-point scalar to a larger float type.
	FPromote(AbiType),
	/// Integer negation.
	INeg,
	/// Split an integer into low and high parts.
	ISplit,
	/// Floating-point rounding to an integer, towards nearest with ties to even.
	Nearest,
	/// Count the number of one bits in an integer.
	PopCnt,
	/// Convert an integer to a larger integral type via sign extension.
	SExtend(AbiType),
	/// Floating-point square root.
	Sqrt,
	/// Floating-point rounding to an integer, towards zero.
	Trunc,
}

/// A strongly-typed index into [`Function::body`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NodeIx(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SymbolRef {
	Native(&'static str),
	User(SymbolIx),
}
