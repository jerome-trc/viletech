//! VZScript's [Cranelift](cranelift)-based backend.

use std::{mem::MaybeUninit, sync::Arc};

use cranelift::{
	codegen::ir::{types as abi_t, ArgumentExtension, ArgumentPurpose},
	prelude::{
		codegen::ir::InstBuilder, settings::OptLevel, AbiParam, Block, EntityRef, FunctionBuilder,
		FunctionBuilderContext, Signature, Variable,
	},
};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataId, FuncId, Linkage, Module};
use parking_lot::{Mutex, RwLock};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use smallvec::smallvec;
use util::rstring::RString;

use crate::{
	compile::{self, symbol::Symbol, Compiler, NativePtr},
	rti::{self, SignatureHash},
	runtime::RuntimePtr,
	tsys::{FuncType, TypeHandle},
	vir,
	zname::ZName,
	FxDashMap, Project, Runtime,
};

pub type AbiType = cranelift::codegen::ir::Type;
pub type AbiTypes = smallvec::SmallVec<[AbiType; 1]>;
pub type SsaValue = cranelift::prelude::Value;
pub type SsaValues = smallvec::SmallVec<[SsaValue; 1]>;

#[must_use]
pub fn codegen(compiler: Compiler, opt: OptLevel, hotswap: bool) -> RuntimePtr {
	assert_eq!(compiler.stage, compile::Stage::CodeGen);
	assert!(!compiler.failed);

	let Compiler {
		native_ptrs,
		strings,
		symbols,
		..
	} = compiler;

	let native_ptrs = Arc::new(native_ptrs);
	let rtinfo = FxDashMap::default();
	let mut rt = Runtime::new(strings);

	symbols
		.iter()
		.par_bridge()
		.fold(
			|| Vec::with_capacity(rayon::current_num_threads() / symbols.len()),
			|mut batch, symbol| {
				batch.push(symbol);
				batch
			},
		)
		.for_each(|mut batch| {
			let cgu = CodeGenUnit::new(&rt, &rtinfo, native_ptrs.clone(), opt, hotswap);

			cgu.run(batch)
		});

	rt.rtinfo = rtinfo.into_read_only();

	rt
}

/// To wrap in an [`Arc`] so that JIT memory is freed properly.
pub(crate) struct JitModule {
	/// Only `None` during dropping.
	pub(crate) inner: Mutex<Option<JITModule>>,
}

impl std::fmt::Debug for JitModule {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("JitModule")
			.field("inner", &"JITModule")
			.finish()
	}
}

unsafe impl Send for JitModule {}
unsafe impl Sync for JitModule {}

impl Drop for JitModule {
	fn drop(&mut self) {
		let mut guard = self.inner.lock();
		let inner = guard.take().unwrap();

		unsafe {
			inner.free_memory();
		}
	}
}

#[derive(Debug)]
pub(self) struct CodeGenUnit<'r> {
	pub(self) module: Arc<JitModule>,
	pub(self) rtinfo: &'r FxDashMap<ZName, rti::Record>,
}

impl<'r> CodeGenUnit<'r> {
	#[must_use]
	pub(self) fn new(
		runtime: &'r RuntimePtr,
		rtinfo: &'r FxDashMap<ZName, rti::Record>,
		native: Arc<FxHashMap<&'static str, NativePtr>>,
		opt_level: OptLevel,
		hotswap: bool,
	) -> Self {
		let o_lvl = match opt_level {
			OptLevel::None => "none",
			OptLevel::Speed => "speed",
			OptLevel::SpeedAndSize => "speed_and_size",
		};

		let mut builder = JITBuilder::with_flags(
			&[
				("use_colocated_libcalls", "false"),
				("is_pic", if hotswap { "true" } else { "false" }),
				("opt_level", o_lvl),
				#[cfg(not(debug_assertions))]
				("enable_verifier", "false"),
			],
			cranelift_module::default_libcall_names(),
		)
		.expect("JIT module builder creation failed");

		builder.hotswap(hotswap);

		builder.symbol_lookup_fn(Box::new(move |name_str| {
			native.get(name_str).map(|np| match np {
				NativePtr::Data { ptr, .. } => *ptr,
				NativePtr::Function { ptr, .. } => *ptr,
			})
		}));

		let rtptr = std::ptr::addr_of!(**runtime);
		builder.symbol("__runtime__", rtptr.cast());

		let module = JITModule::new(builder);

		// We want this off in debug builds and on otherwise.
		// Assert accordingly in case Cranelift ever changes the default.
		debug_assert!(module.isa().flags().enable_verifier());

		Self {
			module: Arc::new(JitModule {
				inner: Mutex::new(Some(module)),
			}),
			rtinfo,
		}
	}

	pub(self) fn run(&self, batch: Vec<&Symbol>) {
		let mut guard = self.module.inner.lock();
		let jit = guard.as_mut().unwrap();

		let mut fctx = FunctionBuilderContext::new();
		let mut cctx = jit.make_context();

		for symbol in batch {
			let guard = symbol.def.load();

			let (typedef, handle, code) = match &guard.kind {
				compile::symbol::DefKind::Function {
					typedef,
					handle,
					code,
				} => (typedef, handle, code),
				compile::symbol::DefKind::None { .. } => unreachable!(),
				_ => continue,
			};

			let ir_fn = match code {
				compile::symbol::FunctionCode::Ir(ir) => ir,
				_ => continue,
			};

			self.lower_function(
				jit,
				FunctionBuilder::new(&mut cctx.func, &mut fctx),
				handle,
				ir_fn,
			);

			let id = jit
				.declare_function(handle.0.name(), Linkage::Export, &cctx.func.signature)
				.expect("JIT function declaration failed");
			jit.define_function(id, &mut cctx)
				.expect("JIT function definition failed");

			jit.clear_context(&mut cctx);
		}

		jit.finalize_definitions()
			.expect("JIT definition finalization failed");

		for (fn_id, fn_decl) in jit.declarations().get_functions() {
			let Some(decl_name) = fn_decl.name.as_ref() else {
				continue;
			};

			let zname = ZName(RString::new(decl_name));

			let sighash = SignatureHash::new(
				fn_decl
					.signature
					.params
					.iter()
					.map(|abi_p| abi_p.value_type),
				fn_decl
					.signature
					.returns
					.iter()
					.map(|abi_p| abi_p.value_type),
			);

			let store = rti::Store::new(
				zname.clone(),
				rti::Function {
					ptr: jit.get_finalized_function(fn_id).cast(),
					id: fn_id,
					sighash,
					module: self.module.clone(),
				},
			);

			let record = rti::Record::new_func(store);
			self.rtinfo.insert(zname, record);
		}
	}

	fn lower_function(
		&self,
		jit: &JITModule,
		mut builder: FunctionBuilder,
		fn_t: &TypeHandle<FuncType>,
		ir_fn: &vir::Function,
	) {
		builder.func.signature = signature_for(jit, fn_t);

		let entry = builder.create_block();
		builder.append_block_params_for_function_params(entry);
		builder.switch_to_block(entry);
		builder.seal_block(entry);

		let mut vars = Vec::with_capacity(ir_fn.vars.len());

		for (i, var_t) in ir_fn.vars.iter().copied().enumerate() {
			let var = Variable::new(i);
			builder.declare_var(var, var_t);
			vars.push(var);
		}

		let mut assembler = Assembler {
			jit,
			builder,
			ir_fn,
			blocks: vec![entry],
		};

		for node in ir_fn.body.iter() {
			assembler.lower_node(node);
		}

		assembler.builder.finalize();
	}
}

struct Assembler<'m> {
	jit: &'m JITModule,
	builder: FunctionBuilder<'m>,
	ir_fn: &'m vir::Function,
	/// Element 0 is always the entry block.
	blocks: Vec<Block>,
}

impl Assembler<'_> {
	fn lower_node(&mut self, node: &vir::Node) {
		match node {
			vir::Node::Assign { var, expr } => {
				let val = self.lower_expr(&self.ir_fn[expr]);
				assert_eq!(val.len(), 1);
				self.builder.def_var(Variable::new(*var), val[0])
			}
			vir::Node::BlockOpen => {
				let b = self.builder.create_block();
				self.builder.switch_to_block(b);
				self.blocks.push(b);
			}
			vir::Node::BlockClose => {
				let b = self.blocks.pop().expect("dangling VIR block");
				self.builder.seal_block(b);
				self.builder.switch_to_block(*self.blocks.last().unwrap());
			}
			vir::Node::Ret(ix) => {
				let vals = self.lower_expr(&self.ir_fn[ix]);
				self.builder.ins().return_(&vals);
			}
			_ => unimplemented!(),
		}
	}

	#[must_use]
	fn lower_expr(&mut self, expr: &vir::Node) -> SsaValues {
		match expr {
			vir::Node::Arg(ix) => smallvec![self.builder.block_params(self.blocks[0])[*ix]],
			vir::Node::Bin { lhs, rhs, op } => self.lower_binary(*lhs, *rhs, *op),
			vir::Node::Immediate(imm) => self.lower_immediate(*imm),
			vir::Node::Unary { operand, op } => self.lower_unary(*operand, *op),
			_ => unimplemented!(),
		}
	}

	#[must_use]
	fn lower_binary(&mut self, lhs: vir::NodeIx, rhs: vir::NodeIx, op: vir::BinOp) -> SsaValues {
		let x = self.lower_expr(&self.ir_fn[lhs]);
		let y = self.lower_expr(&self.ir_fn[rhs]);

		assert_eq!(x.len(), 1);
		assert_eq!(y.len(), 1);

		let val = match op {
			vir::BinOp::BAnd => self.builder.ins().band(x[0], y[0]),
			vir::BinOp::BAndNot => self.builder.ins().band_not(x[0], y[0]),
			vir::BinOp::BOr => self.builder.ins().bor(x[0], y[0]),
			vir::BinOp::BOrNot => self.builder.ins().bor_not(x[0], y[0]),
			vir::BinOp::BXor => self.builder.ins().bxor(x[0], y[0]),
			vir::BinOp::BXorNot => self.builder.ins().bxor_not(x[0], y[0]),
			vir::BinOp::FAdd => self.builder.ins().fadd(x[0], y[0]),
			vir::BinOp::FCmp(cc) => self.builder.ins().fcmp(cc, x[0], y[0]),
			vir::BinOp::FCpySign => self.builder.ins().fcopysign(x[0], y[0]),
			vir::BinOp::FDiv => self.builder.ins().fdiv(x[0], y[0]),
			vir::BinOp::FMax => self.builder.ins().fmax(x[0], y[0]),
			vir::BinOp::FMin => self.builder.ins().fmin(x[0], y[0]),
			vir::BinOp::FMul => self.builder.ins().fmul(x[0], y[0]),
			vir::BinOp::FSub => self.builder.ins().fsub(x[0], y[0]),
			vir::BinOp::IAdd => self.builder.ins().iadd(x[0], y[0]),
			vir::BinOp::ICmp(cc) => self.builder.ins().icmp(cc, x[0], y[0]),
			vir::BinOp::IConcat => self.builder.ins().iconcat(x[0], y[0]),
			vir::BinOp::IShl => self.builder.ins().ishl(x[0], y[0]),
			vir::BinOp::ISub => self.builder.ins().isub(x[0], y[0]),
			vir::BinOp::SAddSat => self.builder.ins().sadd_sat(x[0], y[0]),
			vir::BinOp::SAddOf => {
				let (ret, _flowed) = self.builder.ins().sadd_overflow(x[0], y[0]);
				ret
			}
			vir::BinOp::SDiv => self.builder.ins().sdiv(x[0], y[0]),
			vir::BinOp::SRem => self.builder.ins().srem(x[0], y[0]),
			vir::BinOp::SShr => self.builder.ins().sshr(x[0], y[0]),
			vir::BinOp::UAddSat => self.builder.ins().uadd_sat(x[0], y[0]),
			vir::BinOp::UAddOf => {
				let (ret, _flowed) = self.builder.ins().uadd_overflow(x[0], y[0]);
				ret
			}
			vir::BinOp::UDiv => self.builder.ins().udiv(x[0], y[0]),
			vir::BinOp::URem => self.builder.ins().urem(x[0], y[0]),
			vir::BinOp::UShr => self.builder.ins().ushr(x[0], y[0]),
		};

		smallvec![val]
	}

	#[must_use]
	fn lower_immediate(&mut self, imm: vir::Immediate) -> SsaValues {
		let val = match imm {
			vir::Immediate::I8(int8) => self.builder.ins().iconst(abi_t::I8, int8 as i64),
			vir::Immediate::I16(int16) => self.builder.ins().iconst(abi_t::I16, int16 as i64),
			vir::Immediate::I32(int32) => self.builder.ins().iconst(abi_t::I32, int32 as i64),
			vir::Immediate::I64(int64) => self.builder.ins().iconst(abi_t::I64, int64),
			vir::Immediate::F32(fl32) => self.builder.ins().f32const(fl32),
			vir::Immediate::F64(fl64) => self.builder.ins().f64const(fl64),
			vir::Immediate::Address(addr) => self
				.builder
				.ins()
				.iconst(self.jit.target_config().pointer_type(), addr as i64),
			vir::Immediate::F32X2(_, _)
			| vir::Immediate::F32X4(_, _, _, _)
			| vir::Immediate::I128(_) => {
				unimplemented!()
			}
		};

		smallvec![val]
	}

	#[must_use]
	fn lower_unary(&mut self, operand: vir::NodeIx, op: vir::UnaryOp) -> SsaValues {
		let o = self.lower_expr(&self.ir_fn[operand]);

		assert_eq!(o.len(), 1);

		match op {
			vir::UnaryOp::BNot => [self.builder.ins().bnot(o[0])].into(),
			vir::UnaryOp::Ceil => [self.builder.ins().ceil(o[0])].into(),
			vir::UnaryOp::Cls => [self.builder.ins().cls(o[0])].into(),
			vir::UnaryOp::Clz => [self.builder.ins().clz(o[0])].into(),
			vir::UnaryOp::Ctz => [self.builder.ins().ctz(o[0])].into(),
			vir::UnaryOp::FAbs => [self.builder.ins().fabs(o[0])].into(),
			vir::UnaryOp::F32FromSInt => {
				[self.builder.ins().fcvt_from_sint(abi_t::F32, o[0])].into()
			}
			vir::UnaryOp::F32FromUInt => {
				[self.builder.ins().fcvt_from_uint(abi_t::F32, o[0])].into()
			}
			vir::UnaryOp::F64FromSInt => {
				[self.builder.ins().fcvt_from_sint(abi_t::F64, o[0])].into()
			}
			vir::UnaryOp::F64FromUInt => {
				[self.builder.ins().fcvt_from_uint(abi_t::F64, o[0])].into()
			}
			vir::UnaryOp::FToSInt(ty) => {
				debug_assert!(ty.is_int());
				[self.builder.ins().fcvt_to_sint(ty, o[0])].into()
			}
			vir::UnaryOp::FToUInt(ty) => {
				debug_assert!(ty.is_int());
				[self.builder.ins().fcvt_to_uint(ty, o[0])].into()
			}
			vir::UnaryOp::FDemote(ty) => {
				debug_assert!(ty.is_float());
				[self.builder.ins().fdemote(ty, o[0])].into()
			}
			vir::UnaryOp::Floor => [self.builder.ins().floor(o[0])].into(),
			vir::UnaryOp::FNeg => [self.builder.ins().fneg(o[0])].into(),
			vir::UnaryOp::FPromote(ty) => {
				debug_assert!(ty.is_float());
				[self.builder.ins().fpromote(ty, o[0])].into()
			}
			vir::UnaryOp::INeg => [self.builder.ins().ineg(o[0])].into(),
			vir::UnaryOp::ISplit => {
				let (lo, hi) = self.builder.ins().isplit(o[0]);
				smallvec![lo, hi]
			}
			vir::UnaryOp::Nearest => [self.builder.ins().nearest(o[0])].into(),
			vir::UnaryOp::PopCnt => [self.builder.ins().popcnt(o[0])].into(),
			vir::UnaryOp::SExtend(ty) => {
				debug_assert!(ty.is_int());
				[self.builder.ins().sextend(ty, o[0])].into()
			}
			vir::UnaryOp::Sqrt => [self.builder.ins().sqrt(o[0])].into(),
			vir::UnaryOp::Trunc => [self.builder.ins().trunc(o[0])].into(),
		}
	}
}

#[must_use]
fn signature_for(jit: &JITModule, fn_t: &TypeHandle<FuncType>) -> Signature {
	let mut sig = jit.make_signature();

	for param in &fn_t.params {
		let abi = param.typedef.abi();

		for t in abi {
			sig.params.push(AbiParam {
				value_type: t,
				purpose: ArgumentPurpose::Normal,
				extension: ArgumentExtension::None,
			});
		}
	}

	let ret_t = fn_t.ret.upgrade();
	let ret_abi = ret_t.abi();

	for t in ret_abi {
		sig.returns.push(AbiParam {
			value_type: t,
			purpose: ArgumentPurpose::Normal,
			extension: ArgumentExtension::None,
		});
	}

	sig
}
