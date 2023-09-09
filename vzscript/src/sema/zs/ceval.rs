//! Compile-time evaluation of ZScript functions.

use doomfront::{
	rowan::ast::AstNode,
	zdoom::{ast::IntSuffix, zscript::ast},
};

use crate::{
	issue::{self, Issue},
	sema::{ConstEval, SemaContext},
	vir,
};

pub(super) fn expr(ctx: &SemaContext, ast: ast::Expr) -> Result<ConstEval, ()> {
	match ast {
		ast::Expr::Binary(e_bin) => bin_expr(ctx, e_bin),
		ast::Expr::Call(e_call) => call_expr(ctx, e_call),
		ast::Expr::ClassCast(_) => todo!(),
		ast::Expr::Group(e_grp) => expr(ctx, e_grp.inner()),
		ast::Expr::Ident(_) => todo!(),
		ast::Expr::Index(e_index) => index_expr(ctx, e_index),
		ast::Expr::Literal(e_lit) => literal(ctx, e_lit),
		ast::Expr::Member(_) => todo!(),
		ast::Expr::Postfix(e_post) => postfix_expr(ctx, e_post),
		ast::Expr::Prefix(e_pre) => prefix_expr(ctx, e_pre),
		ast::Expr::Super(e_super) => {
			ctx.raise(
				Issue::new(
					ctx.path,
					e_super.syntax().text_range(),
					"`super` expressions are invalid in a constant context".to_string(),
					issue::Level::Error(issue::Error::ConstEval),
				)
				.with_note("`super` can only be used in class methods".to_string()),
			);

			Err(())
		}
		ast::Expr::Ternary(e_ternary) => ternary_expr(ctx, e_ternary),
		ast::Expr::Vector(e_vector) => vector_expr(ctx, e_vector),
	}
}

fn bin_expr(ctx: &SemaContext, ast: ast::BinExpr) -> Result<ConstEval, ()> {
	let Ok(l_eval) = expr(ctx, ast.left()) else {
		return Err(());
	};

	let Ok(r_eval) = expr(ctx, ast.right().unwrap()) else {
		return Err(());
	};

	let (op_tok, op) = ast.operator();

	// TODO: Remember to check for divide by zero!

	todo!()
}

fn call_expr(ctx: &SemaContext, ast: ast::CallExpr) -> Result<ConstEval, ()> {
	let Ok(called) = expr(ctx, ast::Expr::from(ast.called())) else {
		return Err(());
	};

	// TODO: Check that `called` can be invoked in a const context.

	let arg_list = ast.arg_list();

	for arg in arg_list.args() {
		let Ok(arg_eval) = expr(ctx, arg.expr()) else {
			return Err(());
		};
	}

	// TODO: Param/arg type validation, named argment order validation...
	todo!()
}

fn index_expr(ctx: &SemaContext, ast: ast::IndexExpr) -> Result<ConstEval, ()> {
	let Ok(indexed) = expr(ctx, ast.indexed()) else {
		return Err(());
	};

	let Ok(index) = expr(ctx, ast.index().unwrap()) else {
		return Err(());
	};

	// TODO:
	// - Check if `index` can coerce to an integer
	// - Check if `indexed` can actually be indexed
	// - Bounds check
	todo!()
}

fn postfix_expr(ctx: &SemaContext, ast: ast::PostfixExpr) -> Result<ConstEval, ()> {
	let Ok(operand) = expr(ctx, ast.operand()) else {
		return Err(());
	};

	let (op_tok, op) = ast.operator();

	match op {
		ast::PostfixOp::Minus2 => todo!(),
		ast::PostfixOp::Plus2 => todo!(),
	}
}

fn prefix_expr(ctx: &SemaContext, ast: ast::PrefixExpr) -> Result<ConstEval, ()> {
	let Ok(operand) = expr(ctx, ast.operand()) else {
		return Err(());
	};

	let (op_tok, op) = ast.operator();

	match op {
		ast::PrefixOp::Bang => todo!(),
		ast::PrefixOp::Minus => todo!(),
		ast::PrefixOp::Minus2 => todo!(),
		ast::PrefixOp::Plus => todo!(),
		ast::PrefixOp::Plus2 => todo!(),
		ast::PrefixOp::Tilde => todo!(),
	}
}

fn ternary_expr(ctx: &SemaContext, ast: ast::TernaryExpr) -> Result<ConstEval, ()> {
	let Ok(cond_eval) = expr(ctx, ast.condition()) else {
		return Err(());
	};

	// TODO: Lazily evaluate left or right depending on `cond_eval`.
	todo!()
}

fn vector_expr(ctx: &SemaContext, ast: ast::VectorExpr) -> Result<ConstEval, ()> {
	let mut lanes = 2;

	let e_x = ast.x();
	let e_y = ast.y();

	if let Some(e_z) = ast.z() {
		lanes += 1;
	}

	if let Some(e_w) = ast.w() {
		lanes += 1;
	}

	let t = match lanes {
		2 => todo!(),
		3 => todo!(),
		4 => todo!(),
		_ => unreachable!(),
	};

	Ok(ConstEval {
		typedef: Some(t),
		ir: todo!(),
	})
}

pub(super) fn literal(ctx: &SemaContext, literal: ast::Literal) -> Result<ConstEval, ()> {
	let token = literal.token();

	if token.null() {
		Ok(ConstEval {
			typedef: None,
			ir: vir::Node::Immediate(vir::Immediate::Address(0)),
		})
	} else if let Some(boolean) = token.bool() {
		Ok(ConstEval {
			typedef: Some(ctx.builtins.bool_t.clone()),
			ir: vir::Node::Immediate(vir::Immediate::I8(boolean as i8)),
		})
	} else if let Some(result) = token.int() {
		match result {
			Ok((int, suffix)) => {
				// (RAT) I don't *really* know what to do about these.
				// ZCC's type inference produces a 32-bit variable for each.
				let (int_t, imm) =
					match suffix {
						IntSuffix::None | IntSuffix::L => {
							if int > (i32::MAX as u64) {
								ctx.raise(Issue::new(
								ctx.path,
								literal.syntax().text_range(),
								"integer value out of range for type `int` (no suffix or `l`)".to_string(),
								issue::Level::Error(issue::Error::IntConvert),
							).with_note(format!(
								"valid range for `int` is from {} to {}",
									i32::MIN,
									i32::MAX
							)));

								return Err(());
							}

							(
								ctx.builtins.int32_t.clone(),
								vir::Immediate::I32(int as i32),
							)
						}
						IntSuffix::U | IntSuffix::UU => {
							if int > (u32::MAX as u64) {
								ctx.raise(Issue::new(
								ctx.path,
								literal.syntax().text_range(),
								"integer value out of range for type `uint32` (suffix `u` or `uu`)".to_string(),
								issue::Level::Error(issue::Error::IntConvert),
							).with_note(format!(
								"valid range for `uint32` is from {} to {}",
									u32::MIN,
									u32::MAX
							)));

								return Err(());
							}

							(
								ctx.builtins.uint32_t.clone(),
								vir::Immediate::I32(int as i32),
							)
						}
						IntSuffix::LL => {
							if int > (i64::MAX as u64) {
								ctx.raise(
									Issue::new(
										ctx.path,
										literal.syntax().text_range(),
										"integer value out of range for type `int64` (suffix `ll`)"
											.to_string(),
										issue::Level::Error(issue::Error::IntConvert),
									)
									.with_note(format!(
										"valid range for `int64` is from {} to {}",
										i64::MIN,
										i64::MAX
									)),
								);

								return Err(());
							}

							(
								ctx.builtins.int64_t.clone(),
								vir::Immediate::I64(int as i64),
							)
						}
						IntSuffix::UL => (
							ctx.builtins.uint64_t.clone(),
							vir::Immediate::I64(int as i64),
						),
					};

				Ok(ConstEval {
					typedef: Some(int_t),
					ir: vir::Node::Immediate(imm),
				})
			}
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
			Err(err) => todo!(),
		}
	} else if let Some(text) = token.string() {
		let istring = ctx.intern_string(text);
		// TODO: Not exactly sure what should go here yet.
		todo!()
	} else if let Some(text) = token.name() {
		let name_ix = ctx.names.intern(token.syntax());

		Ok(ConstEval {
			typedef: Some(ctx.builtins.iname_t.clone()),
			ir: vir::Node::Immediate(vir::Immediate::I32(i32::from(name_ix))),
		})
	} else {
		unreachable!()
	}
}
