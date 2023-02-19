//! The instruction evaluation switch.

use crate::lith::{abi::QWord, InstPtr, Runtime, MAX_PARAMS};

use super::{Instruction, RefNode};

impl<'i> Instruction<RefNode<'i>> {
	#[must_use]
	pub(super) fn eval(&self, ctx: &mut Runtime) -> QWord {
		match self {
			Instruction::Jump(index) => {
				ctx.iptr = InstPtr::Running(*index);
				QWord::invalid()
			}
			Instruction::Return => {
				ctx.iptr = InstPtr::Return;
				QWord::invalid()
			}
			Instruction::BinOp { l, r, op } => unsafe { op.eval(l, r) },
			Instruction::Call { func, args } => {
				// TODO: There must be a better way to do this
				match args.len() {
					0 => func.eval(ctx, ()),
					1 => func.eval(ctx, (args[0],)),
					2 => func.eval(ctx, (args[0], args[1])),
					3 => func.eval(ctx, (args[0], args[1], args[2])),
					4 => func.eval(ctx, (args[0], args[1], args[2], args[3])),
					5 => func.eval(ctx, (args[0], args[1], args[2], args[3], args[4])),
					6 => func.eval(ctx, (args[0], args[1], args[2], args[3], args[4], args[5])),
					7 => func.eval(
						ctx,
						(
							args[0], args[1], args[2], args[3], args[4], args[5], args[6],
						),
					),
					8 => func.eval(
						ctx,
						(
							args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7],
						),
					),
					9 => func.eval(
						ctx,
						(
							args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7],
							args[8],
						),
					),
					10 => func.eval(
						ctx,
						(
							args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7],
							args[8], args[9],
						),
					),
					11 => func.eval(
						ctx,
						(
							args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7],
							args[8], args[9], args[10],
						),
					),
					12 => func.eval(
						ctx,
						(
							args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7],
							args[8], args[9], args[10], args[11],
						),
					),
					13 => func.eval(
						ctx,
						(
							args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7],
							args[8], args[9], args[10], args[11], args[12],
						),
					),
					14 => func.eval(
						ctx,
						(
							args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7],
							args[8], args[9], args[10], args[11], args[12], args[13],
						),
					),
					15 => func.eval(
						ctx,
						(
							args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7],
							args[8], args[9], args[10], args[11], args[12], args[13], args[14],
						),
					),
					MAX_PARAMS => func.eval(
						ctx,
						(
							args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7],
							args[8], args[9], args[10], args[11], args[12], args[13], args[14],
							args[15],
						),
					),
					_ => unreachable!(),
				}
			}
			Instruction::Immediate(qw) => *qw,
			Instruction::Pop => unsafe { ctx.stack.pop::<QWord>() },
			Instruction::Push(qw) => unsafe {
				ctx.stack.push(*qw);
				QWord::invalid()
			},
			Instruction::Allocate(typeinfo) => unsafe {
				let typeinfo = *typeinfo;
				let ptr = ctx.alloc_t(typeinfo.clone());
				QWord::from(ptr)
			},
			Instruction::Panic => {
				ctx.iptr = InstPtr::Panic;
				QWord::invalid()
			}
			Instruction::NoOp => QWord::invalid(),
		}
	}
}
