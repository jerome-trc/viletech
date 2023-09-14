//! Compile-time evaluation of VZScript functions.

use crate::{
	ast,
	sema::{CEval, SemaContext},
};

pub(super) fn expr(ctx: &SemaContext, ast: ast::Expr) -> Result<CEval, ()> {
	match ast {
		ast::Expr::Array(_) => todo!(),
		ast::Expr::Binary(_) => todo!(),
		ast::Expr::Block(_) => todo!(),
		ast::Expr::Call(_) => todo!(),
		ast::Expr::Class(_) => todo!(),
		ast::Expr::Construct(_) => todo!(),
		ast::Expr::Enum(_) => todo!(),
		ast::Expr::Field(_) => todo!(),
		ast::Expr::For(_) => todo!(),
		ast::Expr::Group(_) => todo!(),
		ast::Expr::Function(_) => todo!(),
		ast::Expr::Ident(_) => todo!(),
		ast::Expr::Index(_) => todo!(),
		ast::Expr::Literal(_) => todo!(),
		ast::Expr::Prefix(_) => todo!(),
		ast::Expr::Struct(_) => todo!(),
		ast::Expr::Switch(_) => todo!(),
		ast::Expr::Type(_) => todo!(),
		ast::Expr::Union(_) => todo!(),
		ast::Expr::Variant(_) => todo!(),
		ast::Expr::While(_) => todo!(),
	}
}
