//! The Lith Intermediate Representation.
//!
//! LIR is a high-level instruction set to which the abstract syntax trees of
//! LithScript and its backwards-compatibility targets are lowered (after
//! performing semantic checks on them) before being translated to Cranelift
//! Intermediate Format (CLIF). This provides a "lowest common denominator" for
//! compilation so each language does not require its own backend (or AST
//! transpilation to the yet-unstable Lith).

use std::collections::HashMap;

use cranelift::prelude::{types::Type as CraneliftType, FloatCC, IntCC};
use cranelift_module::Linkage;
use smallvec::SmallVec;
use util::rstring::RString;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Name {
	Var(RString),
	Func(RString),
	Module(RString),
}

impl Name {
	#[must_use]
	pub fn as_str(&self) -> &str {
		match self {
			Self::Var(string) | Self::Func(string) | Self::Module(string) => string,
		}
	}
}

#[derive(Debug, Default)]
pub struct Module {
	pub(crate) _symbols: HashMap<Name, Item>,
}

#[derive(Debug)]
pub enum Item {
	/// A symbolic constant or static variable.
	Data(Data),
	Function(Function),
}

#[derive(Debug)]
pub struct Data {
	pub(crate) _linkage: Linkage,
	pub(crate) _type_ix: IxTypeDef,
	pub(crate) _init: Expr,
	pub(crate) _mutable: bool,
}

#[derive(Debug)]
pub struct Function {
	pub(crate) _linkage: Linkage,
	pub(crate) _params: Vec<Parameter>,
	pub(crate) _ret: IxTypeDef,
	pub(crate) _body: Block,
}

#[derive(Debug)]
pub struct Parameter {
	pub(crate) _type_ix: IxTypeDef,
	pub(crate) _default: Option<Expr>,
}

#[derive(Debug)]
pub struct Block {
	pub(crate) _cold: bool,
	pub(crate) _statements: Vec<Expr>,
	pub(crate) _ret: Option<Box<Expr>>,
}

#[derive(Debug)]
pub struct TypeDef {
	pub(crate) _data: TypeData,
}

#[derive(Debug)]
pub enum TypeData {
	Void,
	TypeDef,
	Numeric(NumericType),
}

#[derive(Debug)]
pub enum NumericType {
	I8,
	U8,
	I16,
	U16,
	I32,
	U32,
	I64,
	U64,
	F32,
	F64,
}

#[derive(Debug)]
pub enum Expr {
	/// Construction of a structure or array.
	Aggregate(Vec<Self>),
	/// Evaluation never emits any SSA values.
	Assign {
		expr: Box<Self>,
	},
	Bin {
		lhs: Box<Self>,
		op: BinOp,
		rhs: Box<Self>,
	},
	Block(Block),
	Break,
	Call {
		name: Name,
		args: Vec<Self>,
	},
	CallIndirect {
		type_ix: IxTypeDef,
		lhs: Box<Self>,
		args: Vec<Self>,
	},
	/// Evaluation never emits any SSA values.
	Continue,
	IfElse {
		condition: Box<Self>,
		if_true: Box<Self>,
		if_false: Box<Self>,
	},
	/// Evaluation only emits a single SSA value.
	Immediate(Immediate),
	/// Evaluation never emits any SSA values.
	Local,
	/// An infinite loop. Never emits any SSA values.
	Loop(Block),
	Unary {
		operand: Box<Self>,
		op: UnaryOp,
	},
	/// A lone scalar/pointer/vector value, or an aggregate field.
	Var(SmallVec<[Name; 1]>),
}

#[derive(Debug)]
pub enum BinOp {
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

#[derive(Debug)]
pub enum UnaryOp {
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
	FToSInt(CraneliftType),
	/// Convert a floating-point scalar to an unsigned integer.
	///
	/// The given type corresponds to the latter.
	FToUInt(CraneliftType),
	/// Convert a floating-point scalar to a smaller float type.
	FDemote(CraneliftType),
	/// Floating point rounding to an integer, towards negative infinity.
	Floor,
	/// Floating-point negation.
	FNeg,
	/// Convert a floating-point scalar to a larger float type.
	FPromote(CraneliftType),
	/// Integer negation.
	INeg,
	/// Split an integer into low and high parts.
	ISplit,
	/// Floating-point rounding to an integer, towards nearest with ties to even.
	Nearest,
	/// Count the number of one bits in an integer.
	PopCnt,
	/// Convert an integer to a larger integral type via sign extension.
	SExtend(CraneliftType),
	/// Floating-point square root.
	Sqrt,
	/// Floating-point rounding to an integer, towards zero.
	Trunc,
}

#[derive(Debug)]
pub enum Immediate {
	I8(i8),
	I16(i16),
	I32(i32),
	I64(i64),
	F32(f32),
	F64(f64),
}

// Strong index newtypes ///////////////////////////////////////////////////////

/// Index to a [`TypeDef`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IxTypeDef(u32);
