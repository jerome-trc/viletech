//! Recursion scheme implementation.

use crate::vzs::{abi::QWord, MAX_PARAMS};

use super::{
	detail::{Index, OwningNode},
	InstRef, Instruction,
};

impl Instruction<OwningNode> {
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
			Instruction::Call { func, args, arg_c } => {
				let mut mapped_args = Box::new([QWord::invalid(); MAX_PARAMS]);

				for (i, arg) in args.iter().enumerate() {
					mapped_args[i] = mapper(*arg);
				}

				Instruction::Call {
					func,
					args: mapped_args,
					arg_c: *arg_c,
				}
			}
			Instruction::Pop => Instruction::Pop,
			Instruction::Push(qw) => Instruction::Push(mapper(*qw)),
			Instruction::Allocate(typeinfo) => Instruction::Allocate(typeinfo),
			Instruction::Panic => Instruction::Panic,
			Instruction::NoOp => Instruction::NoOp,
		}
	}
}
