//! Helper for making it slightly more convenient to construct i-node trees.

use std::sync::Arc;

use crate::lith::{abi::QWord, OwningNode};

use super::{detail::Index, BinOp, INodeOwning, Instruction, LineInfo, Tree};

/// Helper for making it slightly more convenient to construct i-node trees.
#[derive(Debug, Default)]
pub(in crate::lith) struct Builder {
	nodes: Vec<INodeOwning>,
}

impl Builder {
	#[must_use]
	pub fn bin_op(&mut self, line_info: LineInfo, op: BinOp, left: Index, right: Index) -> Index {
		let ret = self.nodes.len();

		self.nodes.push(INodeOwning {
			line_info,
			inst: Instruction::BinOp {
				l: left,
				r: right,
				op,
			},
		});

		Index(ret)
	}

	#[must_use]
	pub fn imm(&mut self, line_info: LineInfo, lit: QWord) -> Index {
		let ret = self.nodes.len();

		self.nodes.push(INodeOwning {
			line_info,
			inst: Instruction::Immediate(lit),
		});

		Index(ret)
	}

	pub fn panic(&mut self, line_info: LineInfo) {
		self.nodes.push(INodeOwning {
			line_info,
			inst: Instruction::Panic,
		});
	}

	#[must_use]
	pub fn pop(&mut self, line_info: LineInfo) -> Index {
		let ret = self.nodes.len();

		self.nodes.push(INodeOwning {
			line_info,
			inst: Instruction::Pop,
		});

		Index(ret)
	}

	pub fn push(&mut self, line_info: LineInfo, value: Index) {
		self.nodes.push(INodeOwning {
			line_info,
			inst: Instruction::Push(value),
		});
	}

	pub fn ret(&mut self, line_info: LineInfo) {
		self.nodes.push(INodeOwning {
			line_info,
			inst: Instruction::Return,
		});
	}

	#[must_use]
	pub fn build(self) -> Arc<Tree> {
		debug_assert!(
			!self.nodes.is_empty(),
			"Tried to build a Lith function without any nodes."
		);

		debug_assert!(
			matches!(
				&self.nodes.last().unwrap().inst,
				Instruction::<OwningNode>::Return
			),
			"Tried to build a Lith function without a final return."
		);

		Tree::new(self.nodes)
	}
}
