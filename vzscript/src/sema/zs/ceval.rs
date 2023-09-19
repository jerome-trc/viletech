//! Compile-time evaluation of ZScript functions.

use doomfront::{
	rowan::ast::AstNode,
	zdoom::{ast::IntSuffix, zscript::ast},
};
use smallvec::smallvec;

use crate::{
	issue::{self, Issue},
	sema::{CEval, SemaContext},
	vir,
};

pub(super) fn expr(ctx: &SemaContext, ast: ast::Expr, depth: u16) -> Result<CEval, ()> {
	let Some(next_depth) = depth.checked_add(1) else {
		ctx.raise(
			Issue::new(
				ctx.path,
				ast.syntax().text_range(),
				"recursion limit reached during compile-time evaluation".to_string(),
				issue::Level::Error(issue::Error::CEvalRecursion),
			)
			.with_note("try simplifying this expression".to_string()),
		);

		return Err(());
	};

	match ast {
		ast::Expr::Binary(e_bin) => bin_expr(ctx, e_bin, next_depth),
		ast::Expr::Call(e_call) => call_expr(ctx, e_call, next_depth),
		ast::Expr::ClassCast(_) => todo!(),
		ast::Expr::Group(e_grp) => expr(ctx, e_grp.inner(), next_depth),
		ast::Expr::Ident(_) => todo!(),
		ast::Expr::Index(e_index) => index_expr(ctx, e_index, next_depth),
		ast::Expr::Literal(e_lit) => literal(ctx, e_lit),
		ast::Expr::Member(_) => todo!(),
		ast::Expr::Postfix(e_post) => postfix_expr(ctx, e_post, next_depth),
		ast::Expr::Prefix(e_pre) => prefix_expr(ctx, e_pre, next_depth),
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
		ast::Expr::Ternary(e_ternary) => ternary_expr(ctx, e_ternary, next_depth),
		ast::Expr::Vector(e_vector) => vector_expr(ctx, e_vector, next_depth),
	}
}

fn bin_expr(ctx: &SemaContext, ast: ast::BinExpr, depth: u16) -> Result<CEval, ()> {
	let Ok(l_eval) = expr(ctx, ast.left(), depth) else {
		return Err(());
	};

	let Ok(r_eval) = expr(ctx, ast.right().unwrap(), depth) else {
		return Err(());
	};

	let (op_tok, op) = ast.operator();

	// TODO: Remember to check for divide by zero!

	todo!()
}

fn call_expr(ctx: &SemaContext, ast: ast::CallExpr, depth: u16) -> Result<CEval, ()> {
	let Ok(called) = expr(ctx, ast::Expr::from(ast.called()), depth) else {
		return Err(());
	};

	// TODO: Check that `called` can be invoked in a const context.

	let arg_list = ast.arg_list();

	for arg in arg_list.args() {
		let Ok(arg_eval) = expr(ctx, arg.expr(), depth) else {
			return Err(());
		};
	}

	// TODO: Param/arg type validation, named argment order validation...
	todo!()
}

fn index_expr(ctx: &SemaContext, ast: ast::IndexExpr, depth: u16) -> Result<CEval, ()> {
	let Ok(indexed) = expr(ctx, ast.indexed(), depth) else {
		return Err(());
	};

	let Ok(index) = expr(ctx, ast.index().unwrap(), depth) else {
		return Err(());
	};

	// TODO:
	// - Check if `index` can coerce to an integer
	// - Check if `indexed` can actually be indexed
	// - Bounds check
	// - Remember to consider RNG tables
	todo!()
}

fn postfix_expr(ctx: &SemaContext, ast: ast::PostfixExpr, depth: u16) -> Result<CEval, ()> {
	let Ok(operand) = expr(ctx, ast.operand(), depth) else {
		return Err(());
	};

	let (op_tok, op) = ast.operator();

	match op {
		ast::PostfixOp::Minus2 => todo!(),
		ast::PostfixOp::Plus2 => todo!(),
	}
}

fn prefix_expr(ctx: &SemaContext, ast: ast::PrefixExpr, depth: u16) -> Result<CEval, ()> {
	let Ok(operand) = expr(ctx, ast.operand(), depth) else {
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

fn ternary_expr(ctx: &SemaContext, ast: ast::TernaryExpr, depth: u16) -> Result<CEval, ()> {
	let Ok(cond_eval) = expr(ctx, ast.condition(), depth) else {
		return Err(());
	};

	// TODO: Lazily evaluate left or right depending on `cond_eval`.
	todo!()
}

fn vector_expr(ctx: &SemaContext, ast: ast::VectorExpr, depth: u16) -> Result<CEval, ()> {
	let e_x = ast.x();
	let e_y = ast.y();

	match (ast.z(), ast.w()) {
		(None, None) => {
			let (Ok(ce_x), Ok(ce_y)) = (expr(ctx, e_x, depth), expr(ctx, e_y, depth)) else {
				return Err(());
			};

			let (Ok(f_x), Ok(f_y)) = (num_coerce(ctx, ce_x), num_coerce(ctx, ce_y)) else {
				return Err(());
			};
		}
		(Some(_), None) => {
			todo!()
		}
		(Some(_), Some(_)) => {
			todo!()
		}
		(None, Some(_)) => unreachable!(),
	}

	todo!()
}

pub(super) fn literal(ctx: &SemaContext, literal: ast::Literal) -> Result<CEval, ()> {
	let token = literal.token();

	if token.null() {
		Ok(CEval::Value {
			typedef: None,
			value: smallvec![vir::Immediate::Address(0)],
		})
	} else if let Some(boolean) = token.bool() {
		Ok(CEval::Value {
			typedef: Some(ctx.tcache().bool_t.clone()),
			value: smallvec![vir::Immediate::I8(boolean as i8)],
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
								ctx.tcache().int32_t.clone(),
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
								ctx.tcache().uint32_t.clone(),
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
								ctx.tcache().int64_t.clone(),
								vir::Immediate::I64(int as i64),
							)
						}
						IntSuffix::UL => (
							ctx.tcache().uint64_t.clone(),
							vir::Immediate::I64(int as i64),
						),
					};

				Ok(CEval::Value {
					typedef: Some(int_t),
					value: smallvec![imm],
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
			Ok(float) => Ok(CEval::Value {
				typedef: Some(ctx.tcache().float64_t.clone()),
				value: smallvec![vir::Immediate::F64(float)],
			}),
			Err(err) => todo!(),
		}
	} else if let Some(text) = token.string() {
		let istring = ctx.intern_string(text);
		// TODO: Not exactly sure what should go here yet.
		todo!()
	} else if let Some(text) = token.name() {
		let name_ix = ctx.names.intern(token.syntax());

		Ok(CEval::Value {
			typedef: Some(ctx.tcache().bool_t.clone()),
			value: smallvec![vir::Immediate::I32(i32::from(name_ix))],
		})
	} else {
		unreachable!()
	}
}

fn num_coerce(ctx: &SemaContext, ceval: CEval) -> Result<f32, ()> {
	match ceval {
		CEval::Value { typedef, value } => Err(()),
		CEval::SelfPtr { typedef } => Err(()),
		CEval::SuperPtr { typedef } => Err(()),
		CEval::Type { handle } => Err(()),
		CEval::TypeDef { record } => Err(()),
	}
}
