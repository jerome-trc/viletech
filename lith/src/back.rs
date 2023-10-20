//! Details of Lithica's [Cranelift](cranelift)-based backend.

use std::{hash::BuildHasherDefault, mem::MaybeUninit};

use cranelift::{
	codegen::ir::{
		self, ArgumentExtension, ArgumentPurpose, FuncRef, GlobalValue, UserExternalName,
	},
	prelude::{settings::OptLevel, AbiParam, ExtFuncData, ExternalName, GlobalValueData, Imm64},
};
use cranelift_jit::{JITBuilder, JITModule};

use cranelift_module::{DataId, FuncId, Linkage, Module};
use rustc_hash::FxHashMap;
use util::pushvec::PushVec;

use crate::{compile, runtime::Runtime, Compiler};

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
	let arenas = std::mem::take(&mut compiler.arenas);
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

	define_functions(
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
			function_rti: FxHashMap::default(),
			data_rti: FxHashMap::default(),
			type_rti: FxHashMap::default(),
			module,
			arenas,
		},
		clif: clif_map,
		disasm: disasm_map,
	}
}

fn define_functions(
	_: &Compiler,
	module: &mut JitModule,
	ir: PushVec<(FuncId, ir::Function)>,
	mut clif_map: Option<&mut FxHashMap<FuncId, String>>,
	mut disasm_map: Option<&mut FxHashMap<FuncId, String>>,
) {
	let mut ctx = module.make_context();
	let want_disasm = disasm_map.is_some();

	for (id, clif) in ir.into_iter() {
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

/// Newtype providing `Send` and `Sync` implementations around a [`JITModule`],
/// and ensure that JIT memory gets freed at the correct time.
#[derive(Debug)]
pub(crate) struct JitModule(MaybeUninit<JITModule>);

impl JitModule {
	#[must_use]
	pub(crate) fn new(compiler: &Compiler) -> Self {
		let o_lvl = match compiler.cfg.opt {
			OptLevel::None => "none",
			OptLevel::Speed => "speed",
			OptLevel::SpeedAndSize => "speed_and_size",
		};

		let mut builder = JITBuilder::with_flags(
			&[
				("use_colocated_libcalls", "false"),
				("preserve_frame_pointers", "true"),
				(
					"is_pic",
					if compiler.cfg.hotswap {
						"true"
					} else {
						"false"
					},
				),
				("opt_level", o_lvl),
				#[cfg(not(debug_assertions))]
				("enable_verifier", "false"),
			],
			cranelift_module::default_libcall_names(),
		)
		.expect("JIT module builder creation failed");

		// TODO: runtime intrinsics need to be registered here.

		for (name, nfn) in compiler.native.functions.iter() {
			if let Some(rtn) = &nfn.rt {
				builder.symbol(name.to_string(), rtn.ptr as *const u8);
			};
		}

		let mut module = JITModule::new(builder);
		let ptr_t = module.isa().pointer_type();

		for (name, nfn) in compiler.native.functions.iter() {
			if let Some(rtn) = &nfn.rt {
				let mut signature = module.make_signature();

				// First, the `runtime::Context` pointer.
				let mut params = vec![AbiParam::new(ptr_t)];

				for p in rtn.params {
					params.push(*p);
				}

				signature.params = params;
				signature.returns = rtn.returns.to_owned();

				let _ = module
					.declare_function(name, Linkage::Import, &signature)
					.expect("declaration of a native function to a JIT module failed");
			}
		}

		// We want this off in debug builds and on otherwise.
		// Assert accordingly in case Cranelift ever changes the default.
		debug_assert!(module.isa().flags().enable_verifier());

		Self(MaybeUninit::new(module))
	}

	/// Counterpart to [`Module::declare_func_in_func`] which better serves
	/// the needs of Lithica's sema. pass and its CLIF interpreter.
	#[must_use]
	pub(crate) fn declare_func_in_func(
		&mut self,
		func_id: FuncId,
		ext_name: UserExternalName,
		func: &mut ir::Function,
	) -> FuncRef {
		let decl = self.declarations().get_function_decl(func_id);

		let signature = func.import_signature(decl.signature.clone());

		let user_name_ref = func.declare_imported_user_function(ext_name);

		let colocated = decl.linkage.is_final();

		func.import_function(ExtFuncData {
			name: ExternalName::user(user_name_ref),
			signature,
			colocated,
		})
	}

	/// Counterpart to [`Module::declare_data_in_func`] which better serves
	/// the needs of Lithica's sema. pass and its CLIF interpreter.
	#[must_use]
	pub(crate) fn declare_data_in_func(
		&mut self,
		data_id: DataId,
		ext_name: UserExternalName,
		func: &mut ir::Function,
	) -> GlobalValue {
		let decl = self.declarations().get_data_decl(data_id);

		let user_name_ref = func.declare_imported_user_function(ext_name);

		let colocated = decl.linkage.is_final();

		func.create_global_value(GlobalValueData::Symbol {
			name: ExternalName::user(user_name_ref),
			offset: Imm64::new(0),
			colocated,
			tls: decl.tls,
		})
	}
}

impl std::ops::Deref for JitModule {
	type Target = JITModule;

	fn deref(&self) -> &Self::Target {
		// SAFETY: The `JITModule` within only goes uninitialized when this type's
		// destructor is run, and this type's interface protects against any
		// other de-initialization.
		unsafe { self.0.assume_init_ref() }
	}
}

impl std::ops::DerefMut for JitModule {
	fn deref_mut(&mut self) -> &mut Self::Target {
		// SAFETY: Same as `Deref`.
		unsafe { self.0.assume_init_mut() }
	}
}

impl Drop for JitModule {
	fn drop(&mut self) {
		// SAFETY: this object was initialized upon construction and never mutated.
		// Freeing memory is safe because any handles left open that point into it
		// would have caused a panic when the runtime carry RTI storage dropped.
		unsafe {
			let definitely_init = std::mem::replace(&mut self.0, MaybeUninit::uninit());
			definitely_init.assume_init().free_memory();
		}
	}
}

unsafe impl Send for JitModule {}
unsafe impl Sync for JitModule {}
