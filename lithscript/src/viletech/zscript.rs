//! Transpilation, semantic checking, and LIR lowering for GZDoom's ZScript.

use doomfront::{
	rowan::ast::AstNode,
	zdoom::zscript::{ast, SyntaxNode},
};

use super::{Pass1, Pass3};

pub(super) fn pass1(pass: Pass1) {
	for tu in &pass.src.zscript {
		let ast = SyntaxNode::new_root(tu.root.clone());

		for child in ast.children() {
			match ast::TopLevel::cast(child) {
				Some(ast::TopLevel::EnumDef(_)) => {
					todo!()
				}
				Some(ast::TopLevel::ClassDef(_)) => {
					todo!()
				}
				Some(ast::TopLevel::StructDef(_)) => {
					todo!()
				}
				_ => continue,
			}
		}
	}
}

pub(super) fn pass3(_: Pass3) {
	todo!()
}
