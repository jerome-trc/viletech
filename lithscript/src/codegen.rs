//! Translating [LIR](crate::lir) to Cranelift Intermediate Format.

use std::{collections::HashMap, sync::Arc};

use cranelift::prelude::{
	types, FunctionBuilder, FunctionBuilderContext, InstBuilder, Signature, Value, Variable,
};
use cranelift_jit::JITModule;
use cranelift_module::{DataDescription, Module, Linkage};
use smallvec::{smallvec, SmallVec};

use crate::{lir, viletech::JitModule};

pub(crate) fn compile_module(ir: lir::Module, jit: &Arc<JitModule>) {
	let mut jit_guard = jit.0.lock();
	let jit = unsafe { jit_guard.assume_init_mut() };

	let mut cg = CodeGen {
		fctx: FunctionBuilderContext::new(),
		cctx: jit.make_context(),
		// SAFETY: `JitModule` gets initialized upon construction and is untouched
		// until being passed here.
		jit,
	};

	for (name, sym) in &ir.symbols {
		let lir::Name::Var(string) = name else { unreachable!() };
		let lir::Item::Data(data) = sym else { continue; };

		let id = cg
			.jit
			.declare_data(string, data.linkage, data.mutable, false)
			.expect("JIT data declaration failed");

		let mut desc = DataDescription::new();

		desc.define(unimplemented!(
			"compile-time evaluation not yet implemented"
		));

		cg.jit.define_data(id, &desc);
	}

	for (name, sym) in &ir.symbols {
		let lir::Name::Func(string) = name else { unreachable!() };
		let lir::Item::Function(func) = sym else { continue; };

		translate_func(&ir, &mut cg, func);

		let id = cg
			.jit
			.declare_function(string, func.linkage, &cg.cctx.func.signature)
			.expect("JIT function declaration failed");
		cg.jit
			.define_function(id, &mut cg.cctx)
			.expect("JIT function definition failed");

		cg.jit.clear_context(&mut cg.cctx);
	}

	cg.jit
		.finalize_definitions()
		.expect("JIT definition finalization failed");
}

struct CodeGen<'j> {
	fctx: FunctionBuilderContext,
	cctx: cranelift::codegen::Context,
	jit: &'j mut JITModule,
}

fn translate_func(ir: &lir::Module, cg: &mut CodeGen, func: &lir::Function) {
	let mut builder = FunctionBuilder::new(&mut cg.cctx.func, &mut cg.fctx);
	let entry = builder.create_block();

	builder.append_block_params_for_function_params(entry);
	builder.switch_to_block(entry);
	builder.seal_block(entry);

	let mut tlat = Translator {
		ir,
		jit: &mut cg.jit,
		builder,
		scopes: vec![],
	};

	tlat.builder.func.signature = tlat.signature_for(func);

	for (i, param) in func.params.iter().enumerate() {
		let vals = tlat.builder.block_params(entry);
		let var = todo!();
	}

	tlat.decl_locals(&func.body);

	for expr in &func.body.statements {
		tlat.expr(expr);
	}

	let ret = match &func.body.ret {
		Some(e) => tlat.expr(e),
		None => smallvec![],
	};

	tlat.builder.ins().return_(&ret);
	tlat.builder.finalize();
}

struct Translator<'fb> {
	ir: &'fb lir::Module,
	jit: &'fb mut JITModule,
	builder: FunctionBuilder<'fb>,
	scopes: Vec<Scope>,
}

#[derive(Debug)]
struct Scope {
	vars: HashMap<String, Variable>,
}

impl Translator<'_> {
	fn decl_locals(&mut self, block: &lir::Block) {
		todo!()
	}

	#[must_use]
	fn expr(&mut self, expr: &lir::Expr) -> SmallVec<[Value; 1]> {
		match expr {
			lir::Expr::Aggregate(exprs) => {
				exprs
					.iter()
					.map(|e| self.expr(e))
					.flatten()
					.collect()
			}
			lir::Expr::Assign { expr } => {
				todo!();
				smallvec![]
			}
			lir::Expr::Bin { lhs, op, rhs } => {
				let x = self.expr(lhs);
				let y = self.expr(rhs);

				assert_eq!(x.len(), 1);
				assert_eq!(y.len(), 1);

				match op {
					lir::BinOp::BAnd => smallvec![self.builder.ins().band(x[0], y[0])],
					lir::BinOp::BAndNot => smallvec![self.builder.ins().band_not(x[0], y[0])],
					lir::BinOp::BOr => smallvec![self.builder.ins().bor(x[0], y[0])],
					lir::BinOp::BOrNot => smallvec![self.builder.ins().bor_not(x[0], y[0])],
					lir::BinOp::BXor => smallvec![self.builder.ins().bxor(x[0], y[0])],
					lir::BinOp::BXorNot => smallvec![self.builder.ins().bxor_not(x[0], y[0])],
					lir::BinOp::FAdd => smallvec![self.builder.ins().fadd(x[0], y[0])],
					lir::BinOp::FCmp(cc) => smallvec![self.builder.ins().fcmp(*cc, x[0], y[0])],
					lir::BinOp::FCpySign => smallvec![self.builder.ins().fcopysign(x[0], y[0])],
					lir::BinOp::FDiv => smallvec![self.builder.ins().fdiv(x[0], y[0])],
					lir::BinOp::FMax => smallvec![self.builder.ins().fmax(x[0], y[0])],
					lir::BinOp::FMin => smallvec![self.builder.ins().fmin(x[0], y[0])],
					lir::BinOp::FMul => smallvec![self.builder.ins().fmul(x[0], y[0])],
					lir::BinOp::FSub => smallvec![self.builder.ins().fsub(x[0], y[0])],
					lir::BinOp::IAdd => smallvec![self.builder.ins().iadd(x[0], y[0])],
					lir::BinOp::ICmp(cc) => smallvec![self.builder.ins().icmp(*cc, x[0], y[0])],
					lir::BinOp::IConcat => smallvec![self.builder.ins().iconcat(x[0], y[0])],
					lir::BinOp::IShl => smallvec![self.builder.ins().ishl(x[0], y[0])],
					lir::BinOp::ISub => smallvec![self.builder.ins().isub(x[0], y[0])],
					lir::BinOp::SAddSat => smallvec![self.builder.ins().sadd_sat(x[0], y[0])],
					lir::BinOp::SAddOf => {
						let (ret, _flowed) = self.builder.ins().sadd_overflow(x[0], y[0]);
						smallvec![ret]
					},
					lir::BinOp::SDiv => smallvec![self.builder.ins().sdiv(x[0], y[0])],
					lir::BinOp::SRem => smallvec![self.builder.ins().srem(x[0], y[0])],
					lir::BinOp::SShr => smallvec![self.builder.ins().sshr(x[0], y[0])],
					lir::BinOp::UAddSat => smallvec![self.builder.ins().uadd_sat(x[0], y[0])],
					lir::BinOp::UAddOf => {
						let (ret, _flowed) = self.builder.ins().uadd_overflow(x[0], y[0]);
						smallvec![ret]
					},
					lir::BinOp::UDiv => smallvec![self.builder.ins().udiv(x[0], y[0])],
					lir::BinOp::URem => smallvec![self.builder.ins().urem(x[0], y[0])],
					lir::BinOp::UShr => smallvec![self.builder.ins().ushr(x[0], y[0])],
				}
			}
			lir::Expr::Call { name, args } => {
				let item = self.ir.symbols.get(name).unwrap();
				let lir::Item::Function(func) = item else { unreachable!() };
				let sig = self.signature_for(func);

				let args = args
					.iter().map(|arg| self.expr(arg))
					.flatten()
					.collect::<SmallVec<[Value; 4]>>();

				let callee = self.jit.declare_function(name.as_str(), Linkage::Import, &sig).expect("failed to declare function for call");
				let local_callee = self.jit.declare_func_in_func(callee, self.builder.func);
				let inst = self.builder.ins().call(local_callee, &args);

				self.builder
					.inst_results(inst)
					.into_iter()
					.map(|v| *v)
					.collect()
			}
			lir::Expr::CallIndirect { type_ix, lhs, args } => {
				let sig = todo!();

				let callee = self.expr(lhs);
				assert_eq!(callee.len(), 1);

				let args = args
					.iter()
					.map(|arg| self.expr(arg))
					.flatten()
					.collect::<Vec<_>>();

				let inst = self.builder.ins().call_indirect(sig, callee[0], &args);

				self.builder
					.inst_results(inst)
					.into_iter()
					.map(|v| *v)
					.collect()
			}
			lir::Expr::Continue => {
				let cur = self.builder.current_block().unwrap();
				let inst = self.builder.ins().jump(cur, &[]);

				self.builder
					.inst_results(inst)
					.into_iter()
					.map(|v| *v)
					.collect()
			}
			lir::Expr::IfElse {
				condition,
				if_true: if_then,
				if_false: else_then,
			} => {
				let cond = self.expr(condition);
				assert_eq!(cond.len(), 1);

				let blk_true = self.builder.create_block();
				let blk_false = self.builder.create_block();
				let blk_merge = self.builder.create_block();

				let inst = self
					.builder
					.ins()
					.brif(cond[0], blk_true, &[], blk_false, &[]);

				self.builder.ins().jump(blk_merge, &[todo!()]);
				self.builder.switch_to_block(blk_merge);
				self.builder.seal_block(blk_merge);

				let phi = self.builder.block_params(blk_merge);

				phi
					.into_iter()
					.map(|v| *v)
					.collect()
			}
			lir::Expr::Immediate(imm) => match imm {
				lir::Immediate::I8(int) => {
					smallvec![self.builder.ins().iconst(types::I8, *int as i64)]
				}
				lir::Immediate::I16(int) => {
					smallvec![self.builder.ins().iconst(types::I16, *int as i64)]
				}
				lir::Immediate::I32(int) => {
					smallvec![self.builder.ins().iconst(types::I32, *int as i64)]
				}
				lir::Immediate::I64(int) => {
					smallvec![self.builder.ins().iconst(types::I64, *int as i64)]
				}
				lir::Immediate::F32(float) => smallvec![self.builder.ins().f32const(*float)],
				lir::Immediate::F64(float) => smallvec![self.builder.ins().f64const(*float)],
			},
			lir::Expr::Local => smallvec![],
			lir::Expr::Loop(block) => {
				let blk = self.builder.create_block();
				let _ = self.builder.ins().jump(blk, &[]);
				self.builder.switch_to_block(blk);

				for expr in &block.statements {
					let _ = self.expr(expr);
				}

				self.builder.ins().jump(blk, &[]);

				smallvec![]
			}
			lir::Expr::Unary { operand, op } => {
				let o = self.expr(operand);

				assert_eq!(o.len(), 1);

				match op {
					lir::UnaryOp::BNot => smallvec![self.builder.ins().bnot(o[0])],
					lir::UnaryOp::Ceil => smallvec![self.builder.ins().ceil(o[0])],
					lir::UnaryOp::Cls => smallvec![self.builder.ins().cls(o[0])],
					lir::UnaryOp::Clz => smallvec![self.builder.ins().clz(o[0])],
					lir::UnaryOp::Ctz => smallvec![self.builder.ins().ctz(o[0])],
					lir::UnaryOp::FAbs => smallvec![self.builder.ins().fabs(o[0])],
					lir::UnaryOp::F32FromSInt => smallvec![self.builder.ins().fcvt_from_sint(types::F32, o[0])],
					lir::UnaryOp::F32FromUInt => smallvec![self.builder.ins().fcvt_from_uint(types::F32, o[0])],
					lir::UnaryOp::F64FromSInt => smallvec![self.builder.ins().fcvt_from_sint(types::F64, o[0])],
					lir::UnaryOp::F64FromUInt => smallvec![self.builder.ins().fcvt_from_uint(types::F64, o[0])],
					lir::UnaryOp::FToSInt(ty) => {
						debug_assert!(ty.is_int());
						smallvec![self.builder.ins().fcvt_to_sint(*ty, o[0])]
					},
					lir::UnaryOp::FToUInt(ty) => {
						debug_assert!(ty.is_int());
						smallvec![self.builder.ins().fcvt_to_uint(*ty, o[0])]
					},
					lir::UnaryOp::FDemote(ty) => {
						debug_assert!(ty.is_float());
						smallvec![self.builder.ins().fdemote(*ty, o[0])]
					},
					lir::UnaryOp::Floor => smallvec![self.builder.ins().floor(o[0])],
					lir::UnaryOp::FNeg => smallvec![self.builder.ins().fneg(o[0])],
					lir::UnaryOp::FPromote(ty) => {
						debug_assert!(ty.is_float());
						smallvec![self.builder.ins().fpromote(*ty, o[0])]
					},
					lir::UnaryOp::INeg => smallvec![self.builder.ins().ineg(o[0])],
					lir::UnaryOp::ISplit => {
						let (lo, hi) = self.builder.ins().isplit(o[0]);
						smallvec![lo, hi]
					},
					lir::UnaryOp::Nearest => smallvec![self.builder.ins().nearest(o[0])],
					lir::UnaryOp::PopCnt => smallvec![self.builder.ins().popcnt(o[0])],
					lir::UnaryOp::SExtend(ty) => {
						debug_assert!(ty.is_int());
						smallvec![self.builder.ins().sextend(*ty, o[0])]
					},
					lir::UnaryOp::Sqrt => smallvec![self.builder.ins().sqrt(o[0])],
					lir::UnaryOp::Trunc => smallvec![self.builder.ins().trunc(o[0])],
				}
			}
			lir::Expr::Var(names) => {
				names.iter().map(|name| {
					let lir::Name::Var(n) = name else { unreachable!() };
					self.builder.use_var(todo!())
				}).collect()
			}
			_ => unimplemented!(),
		}
	}

	#[must_use]
	fn signature_for(&self, func: &lir::Function) -> Signature {
		todo!()
	}
}
