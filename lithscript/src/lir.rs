//! The Lith Intermediate Representation.
//!
//! LIR is a high-level instruction set to which the abstract syntax trees of
//! LithScript and its backwards-compatibility targets are lowered (after
//! performing semantic checks on them) before being translated to Cranelift
//! Intermediate Format (CLIF). This provides a "lowest common denominator" for
//! compilation so each language does not require its own backend (or AST
//! transpilation to the yet-unstable Lith).
//!
//! In order to keep the toolchain simple and compact, LIR carries rich semantic
//! information which is irrelevant to code generation in order to facilitate
//! semantic checking without another IR layer.

use std::sync::Arc;

use arc_swap::ArcSwap;
use cranelift::prelude::{types::Type as CraneliftType, FloatCC, IntCC};
use cranelift_module::Linkage;
use smallvec::SmallVec;

use crate::{
	compile::IName,
	rti,
	tsys::{FuncType, TypeDef, TypeHandle},
};

#[derive(Debug, Default)]
pub(crate) enum Symbol {
	Data(Data),
	Function(Function),
	/// Mind that this may also be a type alias.
	Type(rti::Handle<TypeDef>),
	/// A placeholder value used during compilation for symbols which
	/// have been declared but have not yet been defined.
	#[default]
	Unknown,
}

#[derive(Debug)]
pub struct LibSymbol {
	pub(crate) inner: ArcSwap<Symbol>,
	pub(crate) ix_lib: usize,
}

#[derive(Debug, Clone)]
pub struct AtomicSymbol(pub(crate) Arc<LibSymbol>);

impl AtomicSymbol {
	#[must_use]
	pub(crate) fn load(&self) -> arc_swap::Guard<Arc<Symbol>> {
		self.0.inner.load()
	}
}

impl std::ops::Deref for AtomicSymbol {
	type Target = Arc<LibSymbol>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(Debug)]
pub struct Data {
	pub(crate) linkage: Linkage,
	pub(crate) typedef: rti::Handle<TypeDef>,
	pub(crate) init: Block,
	pub(crate) mutable: bool,
}

#[derive(Debug)]
pub struct Function {
	pub(crate) linkage: Linkage,
	pub(crate) params: Vec<Parameter>,
	pub(crate) ret_t: rti::Handle<TypeDef>,
	pub(crate) body: Block,
}

#[derive(Debug)]
pub struct Parameter {
	pub(crate) _typedef: rti::Handle<TypeDef>,
	pub(crate) _default: Option<Block>,
}

/// Thin wrapper around a [`Vec`] of [`Expr`]s.
#[derive(Debug)]
pub(crate) struct Block(pub(crate) Vec<Expr>);

impl Block {
	/// Convenience function for returning the index of a newly-added expression.
	pub(crate) fn push(&mut self, node: Expr) -> usize {
		let ret = self.0.len();
		self.0.push(node);
		ret
	}
}

impl std::ops::Index<IxExpr> for Block {
	type Output = Expr;

	fn index(&self, index: IxExpr) -> &Self::Output {
		&self.0[index.0 as usize]
	}
}

/// An element in a [`Block`].
#[derive(Debug)]
pub(crate) enum Expr {
	/// Construction of a structure or array.
	Aggregate(Vec<IxExpr>),
	/// Evaluation never emits any SSA values.
	Assign {
		var: usize,
		expr: IxExpr,
	},
	Bin {
		lhs: IxExpr,
		op: BinOp,
		rhs: IxExpr,
	},
	Block(Block),
	Break,
	/// A call to a compile-time-known function.
	Call {
		name: IName,
		args: Vec<IxExpr>,
	},
	/// A call to a function pointer (vtable calls included).
	CallIndirect(IndirectCall),
	/// Evaluation never emits any SSA values.
	Continue,
	/// Always points to a [`Symbol::Data`].
	Data(IName),
	IfElse(IfElseExpr),
	/// Evaluation only emits a single SSA value.
	Immediate(Immediate),
	/// Evaluation never emits any SSA values.
	Local(SmallVec<[CraneliftType; 1]>),
	/// An infinite loop. Never emits any SSA values.
	Loop(Block),
	/// Evaluation never emits any SSA values.
	Ret(IxExpr),
	Unary {
		operand: IxExpr,
		op: UnaryOp,
	},
	/// A lone scalar/pointer/vector value, or an aggregate field.
	Var,
}

#[derive(Debug)]
pub struct IfElseExpr {
	pub(crate) condition: IxExpr,
	pub(crate) if_true: IxExpr,
	pub(crate) if_false: IxExpr,
	pub(crate) out_t: Option<rti::Handle<TypeDef>>,
	pub(crate) cold: IfElseCold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IfElseCold {
	True,
	False,
	Neither,
}

#[derive(Debug)]
pub struct IndirectCall {
	pub(crate) typedef: TypeHandle<FuncType>,
	pub(crate) lhs: IxExpr,
	pub(crate) args: Vec<IxExpr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Immediate {
	I8(i8),
	I16(i16),
	I32(i32),
	I64(i64),
	F32(f32),
	F64(f64),
}

/// Strongly-typed index to an [`Expr`] in a [`Block`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IxExpr(pub(crate) u32);
