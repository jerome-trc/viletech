//! Details of Lithica's [Cranelift](cranelift)-based backend.

use std::mem::MaybeUninit;

use cranelift::{
	codegen::ir::{FuncRef, GlobalValue, UserExternalName},
	prelude::{ExtFuncData, ExternalName, GlobalValueData, Imm64},
};
use cranelift_jit::JITModule;

use cranelift_module::{DataId, FuncId, Module};

use crate::IrFunction;

/// Newtype providing `Send` and `Sync` implementations around a [`JITModule`],
/// and ensure that JIT memory gets freed at the correct time.
#[derive(Debug)]
pub(crate) struct JitModule(MaybeUninit<JITModule>);

impl JitModule {
	/// Counterpart to [`Module::declare_func_in_func`] which better serves
	/// the needs of Lithica's sema. pass and its CLIF interpreter.
	#[must_use]
	pub(crate) fn declare_func_in_func(
		&mut self,
		func_id: FuncId,
		ext_name: UserExternalName,
		func: &mut IrFunction,
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

	/// Counterpart to [`Module::declare_func_in_func`] which better serves
	/// the needs of Lithica's sema. pass and its CLIF interpreter.
	#[must_use]
	pub(crate) fn declare_data_in_func(
		&mut self,
		data_id: DataId,
		ext_name: UserExternalName,
		func: &mut IrFunction,
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
		unsafe {
			let definitely_init = std::mem::replace(&mut self.0, MaybeUninit::uninit());
			definitely_init.assume_init().free_memory();
		}
	}
}

unsafe impl Send for JitModule {}
unsafe impl Sync for JitModule {}
