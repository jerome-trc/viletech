//! Transpilation, semantic checking, and LIR lowering for GZDoom's DECORATE.

use doomfront::zdoom::decorate::SyntaxNode;

use super::Pass1;

pub(super) fn pass1(pass: Pass1) {
	for tu in &pass.src.decorate {
		let ast = SyntaxNode::new_root(tu.root.clone());

		for _ in ast.children() {
			// TODO: Declare actor class types.
		}
	}
}
