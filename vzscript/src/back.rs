//! VZScript's [Cranelift](cranelift)-based backend.

use std::{mem::MaybeUninit, sync::Arc};

use cranelift::prelude::settings::OptLevel;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataId, FuncId};
use parking_lot::{Mutex, RwLock};
use rustc_hash::FxHashMap;

use crate::{
	compile::{self, Compiler, NativePtr},
	Project, Runtime,
};

pub type SsaType = cranelift::codegen::ir::Type;
pub type SsaValues = smallvec::SmallVec<[SsaType; 1]>;

pub fn codegen(
	compiler: Compiler,
	opt: OptLevel,
	hotswap: bool,
) -> (Project, Arc<RwLock<Runtime>>) {
	assert_eq!(compiler.stage, compile::Stage::CodeGen);
	assert!(!compiler.failed);

	let Compiler {
		project,
		native,
		strings,
		..
	} = compiler;

	let native_symbols = Arc::new(native);

	let _ = CodeGenUnit::new(native_symbols.clone(), opt, hotswap);

	(project, Runtime::new(strings))
}

/// To wrap in an [`Arc`] so that JIT memory is freed properly.
#[derive(Debug)]
pub(crate) struct JitModule {
	pub(crate) inner: MaybeUninit<Mutex<JITModule>>,
}

unsafe impl Send for JitModule {}
unsafe impl Sync for JitModule {}

impl Drop for JitModule {
	fn drop(&mut self) {
		unsafe {
			std::mem::replace(&mut self.inner, MaybeUninit::uninit())
				.assume_init()
				.into_inner()
				.free_memory();
		}
	}
}

#[derive(Debug)]
pub(crate) struct CodeGenUnit {
	pub(crate) module: Arc<JitModule>,
	pub(crate) functions: Vec<FuncId>,
	pub(crate) data: Vec<DataId>,
}

impl CodeGenUnit {
	#[must_use]
	pub(crate) fn new(
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
			],
			cranelift_module::default_libcall_names(),
		)
		.expect("JIT module builder creation failed");

		builder.hotswap(hotswap);

		builder.symbol_lookup_fn(Box::new(move |name_str| {
			native.get(name_str).map(|np| np.0)
		}));

		let module = Mutex::new(JITModule::new(builder));

		Self {
			module: Arc::new(JitModule {
				inner: MaybeUninit::new(module),
			}),
			functions: vec![],
			data: vec![],
		}
	}
}
