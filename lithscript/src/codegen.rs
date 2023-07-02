//! Translating [LIR](crate::lir) to Cranelift Intermediate Format.

use std::{collections::VecDeque, ops::Deref};

use cranelift::prelude::{
	types, FunctionBuilder, FunctionBuilderContext, InstBuilder, Signature, Value, Variable,
};
use cranelift_jit::JITModule;
use cranelift_module::{DataDescription, Linkage, Module};
use smallvec::{smallvec, SmallVec};

use crate::{
	compile::{Precompile, QName, SymbolTable},
	lir::{self, IxExpr},
	project::Library,
	rti,
	tsys::{FuncType, NumType, TypeDef, TypeHandle, TypeInfo},
	BackendType, Project,
};

type ValVec = SmallVec<[Value; 1]>;

/// Translates a compilation order's [LIR](crate::lir) to Cranelift Intermediate Format.
#[must_use]
pub fn compile(mut precomps: VecDeque<Precompile>, symtab: SymbolTable) -> Project {
	let mut ret = Project::default();

	for (i, precomp) in precomps.drain(..).rev().enumerate() {
		assert!(!precomp.any_errors());
		ret.libs.push(compile_module(i, precomp, &symtab));
	}

	// TODO: Populate project's RTI table. Probably has to happen deeper in this code.

	ret
}

#[must_use]
fn compile_module(ix_lib: usize, precomp: Precompile, symtab: &SymbolTable) -> Library {
	let mut jit = precomp.module.lock();

	let mut cg = CodeGen {
		symtab,
		fctx: FunctionBuilderContext::new(),
		cctx: jit.make_context(),
		jit: &mut jit,
	};

	for kvp in &symtab.0 {
		if kvp.value().ix_lib != ix_lib {
			continue;
		}

		let sym_g = kvp.value().load();
		let lir::Symbol::Data(data) = sym_g.as_ref() else { continue; };
		let QName::Value(string) = kvp.key() else { unreachable!() };

		let id = cg
			.jit
			.declare_data(string.deref(), data.linkage, data.mutable, false)
			.expect("JIT data declaration failed");

		let mut desc = DataDescription::new();

		desc.define(Box::new([/* TODO */]));

		cg.jit
			.define_data(id, &desc)
			.expect("JIT data definition failed");
	}

	for kvp in &symtab.0 {
		if kvp.value().ix_lib != ix_lib {
			continue;
		}

		let sym_g = kvp.value().load();
		let lir::Symbol::Function(func) = sym_g.as_ref() else { continue; };
		let QName::Value(string) = kvp.key() else { unreachable!() };

		cg.translate_func(func);

		let id = cg
			.jit
			.declare_function(string.deref(), func.linkage, &cg.cctx.func.signature)
			.expect("JIT function declaration failed");
		cg.jit
			.define_function(id, &mut cg.cctx)
			.expect("JIT function definition failed");

		cg.jit.clear_context(&mut cg.cctx);
	}

	cg.jit
		.finalize_definitions()
		.expect("JIT definition finalization failed");

	drop(jit);

	Library {
		name: precomp.lib_name,
		version: precomp.lib_vers,
		module: precomp.module,
	}
}

struct CodeGen<'j> {
	symtab: &'j SymbolTable,
	fctx: FunctionBuilderContext,
	cctx: cranelift::codegen::Context,
	jit: &'j mut JITModule,
}

impl CodeGen<'_> {
	fn translate_func(&mut self, func: &lir::Function) {
		let mut builder = FunctionBuilder::new(&mut self.cctx.func, &mut self.fctx);
		let entry = builder.create_block();

		builder.append_block_params_for_function_params(entry);
		builder.switch_to_block(entry);
		builder.seal_block(entry);

		let mut tlat = Translator {
			symtab: self.symtab,
			jit: self.jit,
			func,
			builder,
			vars: vec![],
		};

		tlat.builder.func.signature = tlat.signature_for_func(func);

		for (_, _) in func.params.iter().enumerate() {
			let _ = tlat.builder.block_params(entry);
		}

		tlat.declare_locals(&func.body);
		let mut ret = smallvec![];

		for expr in &func.body.0 {
			ret = tlat.expr(expr);
		}

		tlat.builder.ins().return_(&ret);
		tlat.builder.finalize();
	}
}

struct Translator<'fb> {
	symtab: &'fb SymbolTable,
	jit: &'fb mut JITModule,
	func: &'fb lir::Function,
	builder: FunctionBuilder<'fb>,
	vars: Vec<Variable>,
}

impl Translator<'_> {
	fn declare_locals(&mut self, block: &lir::Block) {
		for stat in &block.0 {
			if let lir::Expr::Local(types) = stat {
				for t in types {
					let var = Variable::from_u32(self.vars.len() as u32);
					self.vars.push(var);
					self.builder.declare_var(var, *t);
				}
			}
		}
	}

	#[must_use]
	fn expr(&mut self, expr: &lir::Expr) -> ValVec {
		match expr {
			lir::Expr::Aggregate(exprs) => exprs
				.iter()
				.copied()
				.flat_map(|e| self.expr(&self.func.body[e]))
				.collect(),
			lir::Expr::Assign { var, expr } => {
				let val = self.expr(&self.func.body[*expr]);
				assert_eq!(val.len(), 1);
				self.builder.def_var(self.vars[*var], val[0]);
				smallvec![]
			}
			lir::Expr::Bin { lhs, op, rhs } => self.translate_bin_expr(*lhs, *op, *rhs),
			lir::Expr::Block(block) => {
				let mut ret = smallvec![];

				for expr in &block.0 {
					ret = self.expr(expr);
				}

				ret
			}
			lir::Expr::Call { name, args } => self.translate_call(name, args),
			lir::Expr::CallIndirect(call) => self.translate_call_indirect(call),
			lir::Expr::Continue => {
				let cur = self.builder.current_block().unwrap();
				let _ = self.builder.ins().jump(cur, &[]);
				smallvec![]
			}
			lir::Expr::IfElse(ie_e) => self.translate_if_else(ie_e),
			lir::Expr::Immediate(imm) => smallvec![self.translate_immediate(*imm)],
			lir::Expr::Local(_) => smallvec![],
			lir::Expr::Loop(block) => {
				self.translate_loop(block);
				smallvec![]
			}
			lir::Expr::Unary { operand, op } => self.translate_unary_expr(*operand, *op),
			_ => unimplemented!(),
		}
	}

	#[must_use]
	fn translate_bin_expr(&mut self, lhs: IxExpr, op: lir::BinOp, rhs: IxExpr) -> ValVec {
		let x = self.expr(&self.func.body[lhs]);
		let y = self.expr(&self.func.body[rhs]);

		assert_eq!(x.len(), 1);
		assert_eq!(y.len(), 1);

		match op {
			lir::BinOp::BAnd => [self.builder.ins().band(x[0], y[0])].into(),
			lir::BinOp::BAndNot => [self.builder.ins().band_not(x[0], y[0])].into(),
			lir::BinOp::BOr => [self.builder.ins().bor(x[0], y[0])].into(),
			lir::BinOp::BOrNot => [self.builder.ins().bor_not(x[0], y[0])].into(),
			lir::BinOp::BXor => [self.builder.ins().bxor(x[0], y[0])].into(),
			lir::BinOp::BXorNot => [self.builder.ins().bxor_not(x[0], y[0])].into(),
			lir::BinOp::FAdd => [self.builder.ins().fadd(x[0], y[0])].into(),
			lir::BinOp::FCmp(cc) => [self.builder.ins().fcmp(cc, x[0], y[0])].into(),
			lir::BinOp::FCpySign => [self.builder.ins().fcopysign(x[0], y[0])].into(),
			lir::BinOp::FDiv => [self.builder.ins().fdiv(x[0], y[0])].into(),
			lir::BinOp::FMax => [self.builder.ins().fmax(x[0], y[0])].into(),
			lir::BinOp::FMin => [self.builder.ins().fmin(x[0], y[0])].into(),
			lir::BinOp::FMul => [self.builder.ins().fmul(x[0], y[0])].into(),
			lir::BinOp::FSub => [self.builder.ins().fsub(x[0], y[0])].into(),
			lir::BinOp::IAdd => [self.builder.ins().iadd(x[0], y[0])].into(),
			lir::BinOp::ICmp(cc) => [self.builder.ins().icmp(cc, x[0], y[0])].into(),
			lir::BinOp::IConcat => [self.builder.ins().iconcat(x[0], y[0])].into(),
			lir::BinOp::IShl => [self.builder.ins().ishl(x[0], y[0])].into(),
			lir::BinOp::ISub => [self.builder.ins().isub(x[0], y[0])].into(),
			lir::BinOp::SAddSat => [self.builder.ins().sadd_sat(x[0], y[0])].into(),
			lir::BinOp::SAddOf => {
				let (ret, _flowed) = self.builder.ins().sadd_overflow(x[0], y[0]);
				[ret].into()
			}
			lir::BinOp::SDiv => [self.builder.ins().sdiv(x[0], y[0])].into(),
			lir::BinOp::SRem => [self.builder.ins().srem(x[0], y[0])].into(),
			lir::BinOp::SShr => [self.builder.ins().sshr(x[0], y[0])].into(),
			lir::BinOp::UAddSat => [self.builder.ins().uadd_sat(x[0], y[0])].into(),
			lir::BinOp::UAddOf => {
				let (ret, _flowed) = self.builder.ins().uadd_overflow(x[0], y[0]);
				[ret].into()
			}
			lir::BinOp::UDiv => [self.builder.ins().udiv(x[0], y[0])].into(),
			lir::BinOp::URem => [self.builder.ins().urem(x[0], y[0])].into(),
			lir::BinOp::UShr => [self.builder.ins().ushr(x[0], y[0])].into(),
		}
	}

	#[must_use]
	fn translate_call(&mut self, name: &QName, args: &[IxExpr]) -> ValVec {
		let kvp = self.symtab.get(name).unwrap();
		let sym_g = kvp.value().load();
		let lir::Symbol::Function(func) = sym_g.as_ref() else { unreachable!() };
		let sig = self.signature_for_func(func);

		let args = args
			.iter()
			.copied()
			.flat_map(|arg| self.expr(&self.func.body[arg]))
			.collect::<SmallVec<[Value; 4]>>();

		let callee = self
			.jit
			.declare_function(kvp.key().as_str(), Linkage::Import, &sig)
			.expect("failed to declare function for call");
		let local_callee = self.jit.declare_func_in_func(callee, self.builder.func);
		let inst = self.builder.ins().call(local_callee, &args);

		self.builder.inst_results(inst).iter().copied().collect()
	}

	#[must_use]
	fn translate_call_indirect(&mut self, call: &lir::IndirectCall) -> ValVec {
		let sig = self.signature_for_func_t(&call.typedef);
		let sigref = self.builder.import_signature(sig);

		let callee = self.expr(&self.func.body[call.lhs]);
		assert_eq!(callee.len(), 1);

		let args = call
			.args
			.iter()
			.copied()
			.flat_map(|arg| self.expr(&self.func.body[arg]))
			.collect::<SmallVec<[Value; 4]>>();

		let inst = self.builder.ins().call_indirect(sigref, callee[0], &args);

		self.builder.inst_results(inst).iter().copied().collect()
	}

	#[must_use]
	fn translate_if_else(&mut self, ifelse: &lir::IfElseExpr) -> ValVec {
		let lir::IfElseExpr {
			condition: cond,
			if_true,
			if_false,
			out_t,
			cold,
		} = ifelse;

		let condition = self.expr(&self.func.body[*cond]);
		assert_eq!(condition.len(), 1);

		let blk_true = self.builder.create_block();
		let blk_false = self.builder.create_block();
		let blk_merge = self.builder.create_block();

		if let Some(ty) = out_t {
			for t in Self::typedef_cltypes(ty) {
				self.builder.append_block_param(blk_merge, t);
			}
		}

		match cold {
			lir::IfElseCold::Neither => {}
			lir::IfElseCold::True => self.builder.set_cold_block(blk_true),
			lir::IfElseCold::False => self.builder.set_cold_block(blk_false),
		}

		let _ = self
			.builder
			.ins()
			.brif(condition[0], blk_true, &[], blk_false, &[]);

		self.builder.switch_to_block(blk_true);
		self.builder.seal_block(blk_true);
		let true_eval = self.expr(&self.func.body[*if_true]);
		self.builder.ins().jump(blk_merge, &true_eval);

		self.builder.switch_to_block(blk_false);
		self.builder.seal_block(blk_false);
		let false_eval = self.expr(&self.func.body[*if_false]);
		self.builder.ins().jump(blk_merge, &false_eval);

		assert_eq!(true_eval.len(), false_eval.len());

		self.builder.switch_to_block(blk_merge);
		self.builder.seal_block(blk_merge);

		let phi = self.builder.block_params(blk_merge);
		phi.iter().copied().collect()
	}

	#[must_use]
	fn translate_immediate(&mut self, imm: lir::Immediate) -> Value {
		match imm {
			lir::Immediate::I8(int) => self.builder.ins().iconst(types::I8, int as i64),
			lir::Immediate::I16(int) => self.builder.ins().iconst(types::I16, int as i64),
			lir::Immediate::I32(int) => self.builder.ins().iconst(types::I32, int as i64),
			lir::Immediate::I64(int) => self.builder.ins().iconst(types::I64, int),
			lir::Immediate::F32(float) => self.builder.ins().f32const(float),
			lir::Immediate::F64(float) => self.builder.ins().f64const(float),
		}
	}

	fn translate_loop(&mut self, block: &lir::Block) {
		let blk = self.builder.create_block();
		let _ = self.builder.ins().jump(blk, &[]);
		self.builder.switch_to_block(blk);
		self.builder.seal_block(blk);

		for expr in &block.0 {
			let _ = self.expr(expr);
		}

		self.builder.ins().jump(blk, &[]);
	}

	#[must_use]
	fn translate_unary_expr(&mut self, operand: IxExpr, op: lir::UnaryOp) -> ValVec {
		let o = self.expr(&self.func.body[operand]);

		assert_eq!(o.len(), 1);

		match op {
			lir::UnaryOp::BNot => [self.builder.ins().bnot(o[0])].into(),
			lir::UnaryOp::Ceil => [self.builder.ins().ceil(o[0])].into(),
			lir::UnaryOp::Cls => [self.builder.ins().cls(o[0])].into(),
			lir::UnaryOp::Clz => [self.builder.ins().clz(o[0])].into(),
			lir::UnaryOp::Ctz => [self.builder.ins().ctz(o[0])].into(),
			lir::UnaryOp::FAbs => [self.builder.ins().fabs(o[0])].into(),
			lir::UnaryOp::F32FromSInt => {
				[self.builder.ins().fcvt_from_sint(types::F32, o[0])].into()
			}
			lir::UnaryOp::F32FromUInt => {
				[self.builder.ins().fcvt_from_uint(types::F32, o[0])].into()
			}
			lir::UnaryOp::F64FromSInt => {
				[self.builder.ins().fcvt_from_sint(types::F64, o[0])].into()
			}
			lir::UnaryOp::F64FromUInt => {
				[self.builder.ins().fcvt_from_uint(types::F64, o[0])].into()
			}
			lir::UnaryOp::FToSInt(ty) => {
				debug_assert!(ty.is_int());
				[self.builder.ins().fcvt_to_sint(ty, o[0])].into()
			}
			lir::UnaryOp::FToUInt(ty) => {
				debug_assert!(ty.is_int());
				[self.builder.ins().fcvt_to_uint(ty, o[0])].into()
			}
			lir::UnaryOp::FDemote(ty) => {
				debug_assert!(ty.is_float());
				[self.builder.ins().fdemote(ty, o[0])].into()
			}
			lir::UnaryOp::Floor => [self.builder.ins().floor(o[0])].into(),
			lir::UnaryOp::FNeg => [self.builder.ins().fneg(o[0])].into(),
			lir::UnaryOp::FPromote(ty) => {
				debug_assert!(ty.is_float());
				[self.builder.ins().fpromote(ty, o[0])].into()
			}
			lir::UnaryOp::INeg => [self.builder.ins().ineg(o[0])].into(),
			lir::UnaryOp::ISplit => {
				let (lo, hi) = self.builder.ins().isplit(o[0]);
				smallvec![lo, hi]
			}
			lir::UnaryOp::Nearest => [self.builder.ins().nearest(o[0])].into(),
			lir::UnaryOp::PopCnt => [self.builder.ins().popcnt(o[0])].into(),
			lir::UnaryOp::SExtend(ty) => {
				debug_assert!(ty.is_int());
				[self.builder.ins().sextend(ty, o[0])].into()
			}
			lir::UnaryOp::Sqrt => [self.builder.ins().sqrt(o[0])].into(),
			lir::UnaryOp::Trunc => [self.builder.ins().trunc(o[0])].into(),
		}
	}

	// Helpers /////////////////////////////////////////////////////////////////

	#[must_use]
	fn typedef_cltypes(typedef: &rti::Handle<TypeDef>) -> SmallVec<[BackendType; 1]> {
		match typedef.inner() {
			TypeInfo::Array(_) | TypeInfo::Class(_) => unimplemented!(),
			TypeInfo::Num(numeric) => match numeric {
				NumType::I8 | NumType::U8 => [types::I8].into(),
				NumType::I16 | NumType::U16 => [types::I16].into(),
				NumType::I32 | NumType::U32 => [types::I32].into(),
				NumType::I64 | NumType::U64 => [types::I64].into(),
				NumType::F32 => [types::F32].into(),
				NumType::F64 => [types::F64].into(),
			},
			TypeInfo::Void(_) => smallvec![],
			TypeInfo::Function(_) | TypeInfo::TypeDef(_) => unreachable!(),
		}
	}

	#[must_use]
	fn signature_for_func(&self, _: &lir::Function) -> Signature {
		unimplemented!()
	}

	#[must_use]
	fn signature_for_func_t(&self, _: &TypeHandle<FuncType>) -> Signature {
		unimplemented!()
	}
}
