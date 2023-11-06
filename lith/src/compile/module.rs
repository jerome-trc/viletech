//! See [`JitModule`], which is used in sema. as well as the JIT backend.

use std::mem::MaybeUninit;

use cranelift::{
	codegen::ir::{self, FuncRef, GlobalValue, UserExternalName},
	prelude::{settings::OptLevel, AbiParam, ExtFuncData, ExternalName, GlobalValueData, Imm64},
};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataId, FuncId, Linkage, Module};

use crate::{
	compile::{NativeFunc, RunTimeNativeFunc},
	Compiler,
};

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
				// The responsibility to run the CLIF verifier falls to
				// the semantic pass in the frontend, not the backend.
				("enable_verifier", "false"),
			],
			cranelift_module::default_libcall_names(),
		)
		.expect("JIT module builder creation failed");

		// TODO: runtime intrinsics need to be registered here.

		for (name, nfn) in compiler.native.functions.iter() {
			match nfn {
				NativeFunc::RunTime(rt) => {
					let RunTimeNativeFunc::Static { ptr, .. } = rt;
					builder.symbol(name.to_string(), *ptr);
				}
				NativeFunc::CompileOrRunTime(_, rt) => {
					let RunTimeNativeFunc::Static { ptr, .. } = rt;
					builder.symbol(name.to_string(), *ptr);
				}
				NativeFunc::CompileTime(_) => continue,
			}
		}

		let mut module = JITModule::new(builder);
		let ptr_t = module.isa().pointer_type();
		let mut signature = module.make_signature();

		for (name, nfn) in compiler.native.functions.iter() {
			let (params, returns) = match nfn {
				NativeFunc::CompileOrRunTime(_, rt) => {
					let RunTimeNativeFunc::Static {
						params, returns, ..
					} = rt;
					(params, returns)
				}
				NativeFunc::RunTime(rt) => {
					let RunTimeNativeFunc::Static {
						params, returns, ..
					} = rt;
					(params, returns)
				}
				NativeFunc::CompileTime(_) => continue,
			};

			// First the `Runtime` pointer.
			signature.params.push(AbiParam::new(ptr_t));

			for param in params.iter() {
				signature.params.push(*param);
			}

			for ret in returns.iter() {
				signature.returns.push(*ret);
			}

			let _ = module
				.declare_function(name, Linkage::Import, &signature)
				.expect("declaration of a native function to a JIT module failed");

			module.clear_signature(&mut signature);
		}

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
