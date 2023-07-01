//! A temporary place for Lith compilation code until the pipeline crystallizes.

use crate::SyntaxNode;

use super::Pass1;

pub(super) fn pass1(pass: Pass1) {
	for ctr in &pass.src.lith {
		let ast = SyntaxNode::new_root(ctr.root.clone());

		for _ in ast.children() {
			// TODO: Declare types.
		}
	}
}