//! Compile-time evaluation of ZScript functions.

use doomfront::zdoom::zscript::ast;

use crate::{
	sema::{ConstEval, SemaContext},
	vir,
};

pub(super) fn expr(ctx: &SemaContext, ast: ast::Expr) -> Result<ConstEval, ()> {
	match ast {
		ast::Expr::Binary(_) => todo!(),
		ast::Expr::Call(_) => todo!(),
		ast::Expr::ClassCast(_) => todo!(),
		ast::Expr::Group(_) => todo!(),
		ast::Expr::Ident(_) => todo!(),
		ast::Expr::Index(_) => todo!(),
		ast::Expr::Literal(_) => todo!(),
		ast::Expr::Member(_) => todo!(),
		ast::Expr::Postfix(_) => todo!(),
		ast::Expr::Prefix(_) => todo!(),
		ast::Expr::Super(_) => todo!(),
		ast::Expr::Ternary(_) => todo!(),
		ast::Expr::Vector(_) => todo!(),
	}
}

pub(super) fn literal(ctx: &SemaContext, literal: ast::Literal) -> Result<ConstEval, ()> {
	let token = literal.token();

	if token.null() {
		Ok(ConstEval {
			typedef: None,
			ir: vir::Node::Immediate(vir::Immediate::pointer(0)),
		})
	} else if let Some(boolean) = token.bool() {
		Ok(ConstEval {
			typedef: Some(ctx.builtins.bool_t.clone()),
			ir: vir::Node::Immediate(vir::Immediate::I8(boolean as i8)),
		})
	} else if let Some(int) = token.int() {
		todo!()
	} else if let Some(float) = token.float() {
		todo!()
	} else if let Some(text) = token.string() {
		let istring = ctx.intern_string(text);
		// TODO: Not exactly sure what should go here yet.
		todo!()
	} else if let Some(text) = token.name() {
		let name_ix = ctx.names.intern(text);

		Ok(ConstEval {
			typedef: Some(ctx.builtins.iname_t.clone()),
			ir: vir::Node::Immediate(vir::Immediate::I32(i32::from(name_ix))),
		})
	} else {
		unreachable!()
	}
}
