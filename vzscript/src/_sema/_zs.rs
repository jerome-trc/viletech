//! Semantic mid-section for ZScript.

mod class;

use std::sync::Arc;

use doomfront::{
	rowan::ast::AstNode,
	zdoom::zscript::{ast, SyntaxNode},
};
use parking_lot::RwLock;

use crate::{
	compile::{
		symbol::{self, Definition, Location, Symbol, ValueDef},
		Scope,
	},
	issue::{self, Issue},
	vir,
};

use super::{ConstEval, SemaContext};

pub(self) fn define_constant(ctx: &SemaContext, constdef: ast::ConstDef) {
	#[must_use]
	fn define(ctx: &SemaContext, constdef: ast::ConstDef, location: Location) -> Symbol {
		let expr = constdef.initializer().unwrap();

		let consteval = match expr {
			ast::Expr::Binary(e_bin) => match consteval_bin(ctx, e_bin) {
				Ok(eval) => eval,
				Err(()) => {
					return ctx.error_symbol(true);
				}
			},
			ast::Expr::ClassCast(e_cast) => {
				ctx.raise(Issue::new(
					ctx.path,
					e_cast.syntax().text_range(),
					"class cast expressions cannot be used to initialize constants".to_string(),
					issue::Level::Error(issue::Error::IllegalConstInit),
				));

				return ctx.error_symbol(true);
			}
			ast::Expr::Literal(e_lit) => match consteval_lit(ctx, e_lit) {
				Ok(eval) => eval,
				Err(()) => {
					return ctx.error_symbol(true);
				}
			},
			ast::Expr::Super(e_super) => {
				ctx.raise(Issue::new(
					ctx.path,
					e_super.syntax().text_range(),
					"`super` expressions cannot be used to initialize constants".to_string(),
					issue::Level::Error(issue::Error::IllegalConstInit),
				));

				return ctx.error_symbol(true);
			}
		};

		let Some(typedef) = consteval.typedef else {
			ctx.raise(Issue::new(
				ctx.path, constdef.syntax().text_range(),
				"type of expression could not be inferred".to_string(),
				issue::Level::Error(issue::Error::UnknownExprType),
			));

			return ctx.error_symbol(true);
		};
	}
}

fn consteval_lit(ctx: &SemaContext, literal: ast::Literal) -> Result<ConstEval, ()> {
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
	} else if let Some(result) = token.int() {
		match result {
			Ok(int) => Ok(ConstEval {
				typedef: Some(ctx.builtins.int32_t.clone()),
				ir: vir::Node::Immediate(vir::Immediate::I32(int)),
			}),
			Err(err) => {
				ctx.raise(Issue::new(
					ctx.path,
					literal.syntax().text_range(),
					format!("invalid integer number: {err}"),
					issue::Level::Error(issue::Error::ParseInt),
				));

				Err(())
			}
		}
	} else if let Some(result) = token.float() {
		match result {
			Ok(float) => Ok(ConstEval {
				typedef: Some(ctx.builtins.float64_t.clone()),
				ir: vir::Node::Immediate(vir::Immediate::F64(float)),
			}),
			Err(err) => {
				ctx.raise(Issue::new(
					ctx.path,
					literal.syntax().text_range(),
					format!("invalid floating-point number: {err}"),
					issue::Level::Error(issue::Error::ParseFloat),
				));

				Err(())
			}
		}
	} else if let Some(string) = token.string() {
		let istring = ctx.intern_string(string);
		let addr = istring.as_thin_ptr() as usize;

		Ok(ConstEval {
			typedef: Some(ctx.builtins.string_t.clone()),
			ir: todo!(),
		})
	} else if let Some(name) = token.name() {
		let name_ix = ctx.names.intern(name);

		Ok(ConstEval {
			typedef: Some(ctx.builtins.iname_t.clone()),
			ir: vir::Node::Immediate(vir::Immediate::I32(i32::from(name_ix))),
		})
	} else {
		unreachable!()
	}
}
