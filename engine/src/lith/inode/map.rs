//! Recursion scheme implementation.

use arrayvec::ArrayVec;

use crate::lith::abi::QWord;

use super::{
	detail::{Index, OwningNode},
	InstRef, Instruction,
};

impl Instruction<OwningNode> {
	#[inline]
	#[must_use]
	pub(super) fn map<F>(&self, mut mapper: F) -> InstRef<'_>
	where
		F: FnMut(Index) -> QWord,
	{
		match self {
			Instruction::Jump(index) => Instruction::Jump(*index),
			Instruction::Return => Instruction::Return,
			Instruction::BinOp { l, r, op } => Instruction::BinOp {
				l: mapper(*l),
				r: mapper(*r),
				op: *op,
			},
			Instruction::Immediate(lit) => Instruction::Immediate(*lit),
			Instruction::Call { func, args } => {
				let mut a = ArrayVec::new();

				for arg in args {
					a.push(mapper(*arg));
				}

				Instruction::Call { func, args: a }
			}
			Instruction::Pop => Instruction::Pop,
			Instruction::Push(qw) => Instruction::Push(mapper(*qw)),
			Instruction::Panic => Instruction::Panic,
			Instruction::NoOp => Instruction::NoOp,
		}
	}
}
