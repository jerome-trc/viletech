//! Interface between [`super::sema`] and [`super::lower`].

use std::{
	cell::RefCell,
	hash::{Hash, Hasher},
};

use cranelift::{
	codegen::ir::{self, UserExternalName},
	prelude::{AbiParam, FunctionBuilder},
};
use cranelift_module::Module;
use crossbeam::utils::Backoff;
use doomfront::rowan::ast::AstNode;
use petgraph::prelude::DiGraph;
use rustc_hash::FxHasher;

use crate::{
	ast,
	back::FunctionIr,
	front::{lower, sym::FunctionKind},
	issue::{self, Issue},
	types::{IrOPtr, IrPtr, Scope, TypePtr},
	SyntaxNode,
};

use super::{
	ceval,
	sema::{CEval, MonoSig, SemaContext},
	sym::{self, ParamType, Symbol},
	tsys::{TypeDatum, TypeDef},
};

/// `env` is the scope which the function inhabits.
pub(super) fn lazy_define(
	ctx: &SemaContext,
	env: &Scope,
	sym: &Symbol,
	datum: &sym::Function,
	mono_sig: MonoSig,
) -> Result<IrPtr, ()> {
	debug_assert!(matches!(datum.kind, FunctionKind::Ir));

	let (sender, receiver) = match ctx.mono.entry(mono_sig.clone()) {
		dashmap::mapref::entry::Entry::Occupied(occ) => {
			let s = occ.get().0.clone();
			let r = occ.get().1.clone();
			drop(occ);
			(s, r)
		}
		dashmap::mapref::entry::Entry::Vacant(vac) => {
			let (sender, receiver) = crossbeam::channel::bounded(1);
			let refmut = vac.insert((sender.clone(), receiver.clone()));
			drop(refmut);

			let new_ctx = ctx.switch_file(sym.location.file_ix);

			let root = SyntaxNode::new_root(new_ctx.ptree.root().clone());
			let fndecl_node = root
				.covering_element(sym.location.span)
				.into_node()
				.unwrap();
			let fndecl = ast::FunctionDecl::cast(fndecl_node).unwrap();

			if let Ok(ir_ptr) = define(&new_ctx, env, sym, datum, mono_sig, fndecl) {
				sender.send(ir_ptr).unwrap();
				return Ok(ir_ptr);
			} else {
				return Err(());
			}
		}
	};

	let backoff = Backoff::new();

	#[cfg(debug_assertions)]
	let start_time = std::time::Instant::now();

	loop {
		if let Ok(ir_ptr) = receiver.try_recv() {
			sender.send(ir_ptr).unwrap();
			return Ok(ir_ptr);
		}

		backoff.snooze();

		#[cfg(debug_assertions)]
		debug_assert!(
			start_time.elapsed().as_secs() < 5,
			"`sema::func::lazy_define` timed out"
		);
	}
}

pub(super) fn eager_define(
	ctx: &SemaContext,
	env: &Scope,
	sym: &Symbol,
	datum: &sym::Function,
	ast: ast::FunctionDecl,
	mono_sig: MonoSig,
) -> Result<(), ()> {
	debug_assert!(matches!(datum.kind, FunctionKind::Ir));

	let dashmap::mapref::entry::Entry::Vacant(vac) = ctx.mono.entry(mono_sig.clone()) else {
		return Ok(());
	};

	let (sender, receiver) = crossbeam::channel::bounded(1);
	let refmut = vac.insert((sender.clone(), receiver.clone()));
	drop(refmut);

	let new_ctx = ctx.switch_file(sym.location.file_ix);

	if let Ok(ir_ptr) = define(&new_ctx, env, sym, datum, mono_sig, ast) {
		sender.send(ir_ptr).unwrap();
		return Ok(());
	}

	Err(())
}

pub(super) fn monomorphize(
	ctx: &SemaContext,
	env: &Scope,
	sym: &Symbol,
	datum: &sym::Function,
	e_call: &ast::ExprCall,
) -> Result<MonoSig, ()> {
	if datum.signature_incomplete() {
		let new_ctx = ctx.switch_file(sym.location.file_ix);

		let root = SyntaxNode::new_root(new_ctx.ptree.root().clone());
		let fndecl_node = root
			.covering_element(sym.location.span)
			.into_node()
			.unwrap();
		let fndecl = ast::FunctionDecl::cast(fndecl_node).unwrap();

		let mut args = e_call.arg_list().unwrap().iter();
		let mut ast_params = fndecl.params().unwrap().iter();

		for param in &datum.params {
			let ast_param = ast_params.next().unwrap();

			let cev = match (args.next(), ast_param.default()) {
				(Some(arg), None) | (Some(arg), Some(_)) => {
					ceval::expr(ctx, 0, env, arg.expr().unwrap())
				}
				(None, Some(e_default)) => ceval::expr(ctx, 0, env, e_default),
				(None, None) => {
					// TODO: raise an error.
					continue;
				}
			};

			match cev {
				CEval::Err => continue,
				CEval::Container(_) => {
					// TODO: raise an error.
					continue;
				}
				CEval::Function(_) => {
					// TODO: raise an error.
					continue;
				}
				CEval::Type(_) => match &param.ptype {
					ParamType::Any => {
						// TODO: raise an error.
						continue;
					}
					ParamType::Type => todo!(),
					ParamType::Normal(_) => todo!(),
				},
				CEval::Value(_) => match &param.ptype {
					ParamType::Any => todo!(),
					ParamType::Type => todo!(),
					ParamType::Normal(_) => todo!(),
				},
			}
		}
	}

	todo!()
}

fn define(
	ctx: &SemaContext,
	outer_scope: &Scope,
	sym: &Symbol,
	datum: &sym::Function,
	mono_sig: MonoSig,
	ast: ast::FunctionDecl,
) -> Result<IrPtr, ()> {
	debug_assert!(matches!(&datum.kind, FunctionKind::Ir));

	let lctx = ctx.lctxs[ctx.thread_ix].lock();
	let mut fctx = RefCell::borrow_mut(&lctx.fctx);
	let mut cctx = RefCell::borrow_mut(&lctx.cctx);
	let mut signature = lctx.sig.borrow_mut();

	signature.params.push(AbiParam::new(ctx.ptr_t));

	for mono_param in mono_sig.params {
		get_abi_params(&mut signature.params, &mono_param);
	}

	get_abi_params(&mut signature.returns, &mono_sig.ret_t);

	let mut tlat = Translator {
		ctx,
		failed: false,
		builder: FunctionBuilder::new(&mut cctx.func, &mut fctx),
		_cflow: DiGraph::default(),
		_next_var: 0,
	};

	let body = ast.body().unwrap();

	let uextname = UserExternalName {
		namespace: ctx.file_ix.index() as u32,
		index: sym.location.span.start().into(),
	};

	let _ = tlat
		.builder
		.func
		.params
		.ensure_user_func_name(uextname.clone());

	tlat.builder
		.func
		.params
		.ensure_base_srcloc(ctx.make_srcloc(body.syntax().text_range().start()));

	let blk_entry = tlat.builder.create_block();
	tlat.builder
		.append_block_params_for_function_params(blk_entry);
	tlat.builder.switch_to_block(blk_entry);
	tlat.builder.seal_block(blk_entry);

	let mut body_scope = outer_scope.clone();

	for innard in body.innards() {
		match innard {
			ast::CoreElement::Statement(stmt) => {
				tlat.builder
					.set_srcloc(ctx.make_srcloc(stmt.syntax().text_range().start()));

				lower::statement(&mut tlat, &mut body_scope, stmt);
			}
			ast::CoreElement::Item(item) => {
				ctx.raise(
					Issue::new(
						ctx.path,
						item.syntax().text_range(),
						issue::Level::Error(issue::Error::Unimplemented),
					)
					.with_message_static(
						"item declarations in function bodies are not yet supported",
					),
				);

				tlat.failed = true;
			}
			ast::CoreElement::Annotation(anno) => {
				ctx.raise(
					Issue::new(
						ctx.path,
						anno.syntax().text_range(),
						issue::Level::Error(issue::Error::Unimplemented),
					)
					.with_message_static("annotations in function bodies are not yet supported"),
				);

				tlat.failed = true;
			}
			ast::CoreElement::Import(import) => {
				ctx.raise(
					Issue::new(
						ctx.path,
						import.syntax().text_range(),
						issue::Level::Error(issue::Error::Unimplemented),
					)
					.with_message_static("imports in function bodies are not yet supported"),
				);

				tlat.failed = true;
			}
		}
	}

	if tlat.failed {
		// TODO: inform other threads that definition failed.
		return Err(());
	}

	tlat.builder.finalize();

	let ir = std::mem::replace(&mut cctx.func, ir::Function::new());
	let ir_ptr = IrOPtr::alloc(ctx.arena, ir);

	let sig_hash = {
		let mut hasher = FxHasher::default();
		signature.params.hash(&mut hasher);
		signature.returns.hash(&mut hasher);
		hasher.finish()
	};

	let fn_id;

	{
		let mut guard = ctx.module.lock();

		// TODO: eventually this can become debug-only.
		cranelift::codegen::verify_function(&cctx.func, guard.isa())
			.expect("CLIF function did not pass verification");

		fn_id = guard
			.declare_anonymous_function(&signature)
			.expect("JIT function declaration failed");

		guard.clear_context(&mut cctx);
		drop(guard);

		signature.params.clear();
		signature.returns.clear();
	};

	drop(cctx);
	drop(fctx);
	drop(signature);
	drop(lctx);

	let ret = IrPtr::from(&ir_ptr);

	ctx.ir.insert(
		uextname,
		FunctionIr {
			id: fn_id,
			ptr: ir_ptr,
			_sig_hash: sig_hash,
		},
	);

	Ok(ret)
}

// Miscellaneous details ///////////////////////////////////////////////////////

fn get_abi_params(p: &mut Vec<AbiParam>, tdef: &TypeDef) {
	match &tdef.datum {
		TypeDatum::_Array { inner, len } => {
			for _ in 0..*len {
				get_abi_params(p, inner);
			}
		}
		TypeDatum::Primitive(prim) => {
			if let Some(abi_t) = prim.abi() {
				p.push(AbiParam::new(abi_t));
			}
		}
		TypeDatum::_Structure(structure) => {
			for field in &structure.fields {
				get_abi_params(p, field.tspec.as_ref());
			}
		}
	}
}

pub(super) type BlockIx = petgraph::graph::NodeIndex<petgraph::graph::DefaultIx>;

pub(super) type _SsaValue = cranelift::prelude::Value;
pub(super) type _SsaValues = smallvec::SmallVec<[_SsaValue; 1]>;

pub(super) struct Translator<'c> {
	pub(super) ctx: &'c SemaContext<'c>,
	pub(super) failed: bool,
	pub(super) builder: FunctionBuilder<'c>,
	pub(super) _cflow: DiGraph<FlowBlock, Flow>,
	pub(super) _next_var: u32,
}

#[derive(Debug)]
pub(super) enum FlowBlock {
	_Normal,
	_If,
	_Else,
	_For,
	_While,
	_Case,
}

#[derive(Debug)]
pub(super) enum Flow {
	_Pass,
	_Continue { to: BlockIx },
	_Break { to: BlockIx, break_t: TypePtr },
	_Return { ret_t: TypePtr },
}
