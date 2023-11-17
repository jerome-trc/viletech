//! Details of Lithica's [Cranelift](cranelift)-based backend.

use std::{any::TypeId, hash::BuildHasherDefault};

use cranelift::codegen::ir::UserExternalName;
use cranelift_module::{FuncId, Module};
use rustc_hash::FxHashMap;

use crate::{
	compile::{self, module::JitModule},
	runtime::Runtime,
	types::{FxDashMap, IrOPtr},
	Compiler,
};

/// The complete set of possible compilation artifacts which can be emitted by [`finalize`].
#[derive(Debug)]
pub struct Compilation {
	pub runtime: Runtime,
	/// Pretty-printed Cranelift Intermediate Format.
	///
	/// This is a middle stage between Lith ASTs and machine code; LithC interprets
	/// this as it is generated to perform compile-time evaluation.
	pub clif: Option<FxHashMap<FuncId, String>>,
	/// Pretty-printed Cranelift VCode,
	/// which resembles the final generated machine instructions.
	pub disasm: Option<FxHashMap<FuncId, String>>,
}

/// [`Compilation::clif`] will only be `Some` if `emit_clif` is `true`.
/// [`Compilation::disasm`] will only be `Some` if `disasm` is `true`.
#[must_use]
pub fn finalize(mut compiler: Compiler, emit_clif: bool, disasm: bool) -> Compilation {
	assert!(!compiler.failed);
	assert_eq!(compiler.stage, compile::Stage::CodeGen);

	let mut module = compiler.module.take().unwrap();
	let ir = std::mem::take(&mut compiler.ir);
	let fn_count = ir.len();

	let mut clif_map = if emit_clif {
		Some(FxHashMap::with_capacity_and_hasher(
			fn_count,
			BuildHasherDefault::default(),
		))
	} else {
		None
	};

	let mut disasm_map = if disasm {
		Some(FxHashMap::with_capacity_and_hasher(
			fn_count,
			BuildHasherDefault::default(),
		))
	} else {
		None
	};

	jit_compile_functions(
		&compiler,
		&mut module,
		ir,
		clif_map.as_mut(),
		disasm_map.as_mut(),
	);

	module
		.finalize_definitions()
		.expect("JIT definition finalization failed");

	Compilation {
		runtime: Runtime {
			_function_rti: FxHashMap::default(),
			_data_rti: FxHashMap::default(),
			_type_rti: FxHashMap::default(),
			module,

			userdata: std::ptr::null_mut(),
			userdata_t: TypeId::of::<()>(),
		},
		clif: clif_map,
		disasm: disasm_map,
	}
}

fn jit_compile_functions(
	_: &Compiler,
	module: &mut JitModule,
	ir: FxDashMap<UserExternalName, FunctionIr>,
	mut clif_map: Option<&mut FxHashMap<FuncId, String>>,
	mut disasm_map: Option<&mut FxHashMap<FuncId, String>>,
) {
	let mut ctx = module.make_context();
	let want_disasm = disasm_map.is_some();

	for (_, FunctionIr { id, ptr, .. }) in ir.into_iter() {
		let clif = unsafe { ptr.read() };
		ctx.set_disasm(want_disasm);

		if let Some(m) = clif_map.as_mut() {
			let mut buf = String::new();
			cranelift::codegen::write::write_function(&mut buf, &clif).unwrap();
			m.insert(id, buf);
		}

		ctx.func = clif;

		module
			.define_function(id, &mut ctx)
			.expect("JIT function definition failed");

		if let Some(m) = disasm_map.as_mut() {
			let comp_code = ctx.compiled_code().unwrap();
			let vcode = comp_code.vcode.as_ref().unwrap();
			m.insert(id, vcode.clone());
		}

		module.clear_context(&mut ctx);
	}
}

#[derive(Debug)]
pub(crate) struct FunctionIr {
	pub(crate) id: FuncId,
	pub(crate) ptr: IrOPtr,
	pub(crate) _sig_hash: u64,
}
