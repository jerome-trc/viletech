//! Compile-time evaluation routines.
//!
//! "Naked expressions" - i.e. `const SYMCONST: i32 = 2 + 2;` - are computed by
//! interpreting the AST, since in the general case these will be too trivial
//! to justify anything more sophisticated.
//!
//! Any time a function is called, one of three things will happen:
//! - a script-defined function will be lazily lowered to CLIF and interpreted
//! - a compiler builtin will be passed the argument list's AST
//! - a native-registered function will be passed the argument list's AST

use cranelift::codegen::{data_value::DataValue, ir};
use cranelift_interpreter::{instruction::DfgInstructionContext, step::ControlFlow};
use doomfront::rowan::ast::AstNode;
use smallvec::smallvec;

use crate::{
	ast,
	interpret::{self, Interpreter},
	issue::{self, Issue},
	sym::{self, Datum, FunctionKind, Symbol},
	types::{CEvalIntrin, Scope},
	CEval, CEvalNative, CeValue, SemaContext,
};

use super::func;

// Expression evaluation ///////////////////////////////////////////////////////

/// If the expression being evaluated is part of a function declaration's return
/// value type specifier, `env` will include the names of the function's parameters.
pub(super) fn expr(ctx: &SemaContext, depth: u8, env: &Scope, ast: ast::Expr) -> CEval {
	let Some(next_depth) = depth.checked_add(1) else {
		ctx.raise(
			Issue::new(
				ctx.path,
				ast.syntax().text_range(),
				issue::Level::Error(issue::Error::CEvalRecursion),
			)
			.with_message_static("compile-time evaluation recurred too deeply")
			.with_note_static("try simplifying this expression"),
		);

		return CEval::Err;
	};

	match ast {
		ast::Expr::Call(e_call) => expr_call(ctx, next_depth, env, e_call),
		ast::Expr::Ident(e_ident) => expr_ident(ctx, env, e_ident),
		ast::Expr::Literal(e_lit) => expr_literal(ctx, e_lit),
		ast::Expr::Binary(_)
		| ast::Expr::Field(_)
		| ast::Expr::Group(_)
		| ast::Expr::Index(_)
		| ast::Expr::Postfix(_)
		| ast::Expr::Prefix(_)
		| ast::Expr::Type(_) => unimplemented!(),
	}
}

fn expr_call(ctx: &SemaContext, depth: u8, env: &Scope, ast: ast::ExprCall) -> CEval {
	let e_called = ast::Expr::from(ast.called());
	let span = e_called.syntax().text_range();
	let callable = expr(ctx, depth, env, e_called);

	let callable_t = match callable {
		CEval::Container(_) => {
			ctx.raise(
				Issue::new(
					ctx.path,
					span,
					issue::Level::Error(issue::Error::IllegalCall),
				)
				.with_message_static(
					"call expressions require a function; found a container reference",
				),
			);

			return CEval::Err;
		}
		CEval::Function(f) => f,
		CEval::Type(_) => {
			ctx.raise(
				Issue::new(
					ctx.path,
					span,
					issue::Level::Error(issue::Error::IllegalCall),
				)
				.with_message_static("call expressions require a function; found a type"), // TODO: how to report this type's name, if any?
			);

			return CEval::Err;
		}
		CEval::Value(_) => {
			ctx.raise(
				Issue::new(
					ctx.path,
					span,
					issue::Level::Error(issue::Error::IllegalCall),
				)
				.with_message_static("call expressions require a function; found a value"), // TODO: some sort of type provenance, so a name can be shown here.
			);

			return CEval::Err; // TODO: function pointers?
		}
		CEval::Err => return CEval::Err,
	};

	let d_fn = match &callable_t.datum {
		Datum::Function(d_fn) => d_fn,
		Datum::Container(_) => {
			ctx.raise(
				Issue::new(
					ctx.path,
					ast.syntax().text_range(),
					issue::Level::Error(issue::Error::IllegalCall),
				)
				.with_message_static(
					"call expressions require a function; found a container reference",
				),
			);

			return CEval::Err;
		}
		Datum::Local(_) => {
			ctx.raise(
				Issue::new(
					ctx.path,
					ast.syntax().text_range(),
					issue::Level::Error(issue::Error::IllegalCall),
				)
				.with_message_static("call expressions require a function; found a local variable"),
			);

			return CEval::Err;
		}
		Datum::SymConst(_) => {
			ctx.raise(
				Issue::new(
					ctx.path,
					ast.syntax().text_range(),
					issue::Level::Error(issue::Error::IllegalCall),
				)
				.with_message_static(
					"call expressions require a function; found a symbolic constant",
				),
			);

			return CEval::Err;
		}
	};

	match &d_fn.kind {
		FunctionKind::Ir => try_call_ir(ctx, env, &callable_t, d_fn, ast),
		FunctionKind::Builtin { ceval, .. } => try_call_builtin(ctx, *ceval, ast),
		FunctionKind::Native { ceval, .. } => try_call_native(ctx, *ceval, ast),
	}
}

fn expr_ident(ctx: &SemaContext, env: &Scope, ast: ast::ExprIdent) -> CEval {
	let token = ast.token();
	let name_ix = ctx.names.intern(&token);

	let Some(sym_ptr) = env.get(&name_ix) else {
		ctx.raise(
			Issue::new(
				ctx.path,
				token.text_range(),
				issue::Level::Error(issue::Error::SymbolNotFound),
			)
			.with_message(format!("no value `{}` found in this scope", token.text())),
		);

		return CEval::Err;
	};

	match &sym_ptr.datum {
		Datum::Container(scope) => CEval::Container(scope.clone()),
		Datum::Function(_) => CEval::Function(sym_ptr.inner),
		Datum::SymConst(_) => todo!("lazy define"),
		Datum::Local(_) => unreachable!(),
	}
}

fn expr_literal(ctx: &SemaContext, ast: ast::ExprLit) -> CEval {
	let token = ast.token();

	if let Some(res_int) = token.int() {
		match res_int {
			Ok(_) => todo!(),
			Err(_) => todo!(),
		}
	} else if let Some(res_float) = token.float() {
		match res_float {
			Ok(_) => todo!(),
			Err(_) => todo!(),
		}
	} else if let Some(b) = token.bool() {
		CEval::Value(CeValue {
			data: smallvec![DataValue::I8(b.into())],
			ftype: ctx.sym_cache.bool_t.clone().into(),
		})
	} else if token.name().is_some() {
		CEval::Value(CeValue {
			data: smallvec![DataValue::from(ctx.names.intern(&token))],
			ftype: ctx.sym_cache.iname_t.clone().into(),
		})
	} else if let Some(string) = token.string() {
		unimplemented!("string interning")
	} else {
		unreachable!()
	}
}

// Function evaluation /////////////////////////////////////////////////////////

fn try_call_ir(
	ctx: &SemaContext,
	env: &Scope,
	sym: &Symbol,
	d_fn: &sym::Function,
	e_call: ast::ExprCall,
) -> CEval {
	let Ok(mono_sig) = func::monomorphize(ctx, env, sym, d_fn, &e_call) else {
		return CEval::Err;
	};

	let Ok(ir_ptr) = func::lazy_define(ctx, env, sym, d_fn, mono_sig) else {
		return CEval::Err;
	};

	let mut istate = Interpreter::new(ctx.compiler, ir_ptr);
	let mut fuel = 10_000_u32;

	let inst = ir_ptr
		.layout
		.first_inst(ir_ptr.layout.entry_block().unwrap())
		.unwrap();

	loop {
		let inst_ctx = DfgInstructionContext::new(inst, &ir_ptr.dfg);

		if fuel == 0 {
			ctx.raise(
				Issue::new(
					ctx.path,
					e_call.syntax().text_range(),
					issue::Level::Error(issue::Error::CEvalRunaway),
				)
				.with_message_static("compile-time execution took too long to finish")
				.with_note_static("this function may be too complex or looping infinitely"),
			);

			return CEval::Err;
		} else {
			fuel -= 1;
		}

		let cflow = match interpret::step(&mut istate, inst_ctx) {
			Ok(f) => f,
			Err(err) => {
				ctx.raise(
					Issue::new(
						ctx.path,
						e_call.syntax().text_range(),
						issue::Level::Error(issue::Error::CEvalInterpret),
					)
					.with_message(format!(
						"internal compiler error: fatal compile-time interpretation failure {err:?}"
					)),
				);

				return CEval::Err;
			}
		};

		match cflow {
			ControlFlow::Continue => {}
			ControlFlow::ContinueAt(_, _) => todo!(),
			ControlFlow::Assign(_) => todo!(),
			ControlFlow::Call(_, _) => todo!(),
			ControlFlow::Return(_) => todo!(),
			ControlFlow::ReturnCall(_, _) => todo!(),
			ControlFlow::Trap(_) => {
				// TODO: handle all different kinds of trap codes.
				return CEval::Err;
			}
		}
	}

	todo!()
}

fn try_call_builtin(
	ctx: &SemaContext,
	fn_opt: Option<CEvalIntrin>,
	e_call: ast::ExprCall,
) -> CEval {
	let Some(func) = fn_opt else {
		ctx.raise(
			Issue::new(
				ctx.path,
				e_call.syntax().text_range(),
				issue::Level::Error(issue::Error::CEvalImpossible),
			)
			.with_message(format!(
				"built-in function `{}` cannot be called in a compile-time context",
				e_call.called().syntax().text()
			)),
		);

		return CEval::Err;
	};

	func(ctx, e_call.arg_list().unwrap())
}

fn try_call_native(ctx: &SemaContext, fn_opt: Option<CEvalNative>, e_call: ast::ExprCall) -> CEval {
	let Some(func) = fn_opt else {
		ctx.raise(
			Issue::new(
				ctx.path,
				e_call.syntax().text_range(),
				issue::Level::Error(issue::Error::CEvalImpossible),
			)
			.with_message(format!(
				"native function `{}` cannot be called in a compile-time context",
				e_call.called().syntax().text()
			)),
		);

		return CEval::Err;
	};

	todo!()
}
