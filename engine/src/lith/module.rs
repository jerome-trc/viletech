//! A LithScript module is a single linkage unit.
//!
//! This is an equivalent concept to LLVM's module, Rust's module, Cranelift's
//! module...

use std::ffi::c_void;

use cranelift::prelude::{types as ClTypes, AbiParam};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module as CraneliftModule};
use indexmap::IndexMap;

use crate::lith::{interop::C_TRAMPOLINES, MAX_RETS};

use super::{
	func::{FunctionFlags, FunctionInfo},
	interop::NativeFnBox,
	word::Word,
};

/// Start by [creating a `Builder`] and populating it with native functions and
/// data that you want to allow scripts to be able to access, and then call
/// [`Builder::build`].
///
/// [creating a `Builder`]: Builder::new
pub struct Module {
	_name: String,
	/// Freeing the module's memory requires pass-by-value, but [`Drop::drop`]
	/// takes a mutable reference, so an indirection is needed here to allow
	/// removing the module upon destruction. It is never `None`, ever.
	inner: Option<JITModule>,
	/// Indices are guaranteed to be stable, since this map is append-only.
	_functions: IndexMap<String, FunctionInfo>,
}

impl Drop for Module {
	fn drop(&mut self) {
		let inner = self.inner.take();

		debug_assert!(inner.is_some());

		unsafe {
			inner.unwrap_unchecked().free_memory();
		}
	}
}

/// Used to prepare for building a [`Module`], primarily by registering native
/// functions to be callable by scripts.
pub struct Builder {
	name: String,
	inner: JITBuilder,
	native_fns: Vec<NativeFn>,
}

impl Builder {
	/// Also see [`JITBuilder::hotswap`].
	#[must_use]
	pub fn new(name: String, hotswap: bool) -> Self {
		let mut builder = JITBuilder::new(cranelift_module::default_libcall_names())
			.expect("Failed to construct a Cranelift `JITBuilder`.");

		builder.hotswap(hotswap);

		Self {
			name,
			inner: builder,
			native_fns: Default::default(),
		}
	}

	/// See [`JITBuilder::symbol_lookup_fn`].
	///
	/// When declaring data to the JIT module, symbol lookup will use these, in
	/// the reverse order that they were pushed in. The last function to try will
	/// always try to find the symbol in the system's C runtime.
	///
	/// If this is undesirable, it is recommended that you add a lookup function
	/// which alerts you to the lookup and returns a null pointer, or panics outright.
	#[must_use]
	pub fn symbol_lookup_fn<F: 'static + Fn(&str) -> Option<*const u8>>(
		mut self,
		function: F,
	) -> Self {
		self.inner.symbol_lookup_fn(Box::new(function));
		self
	}

	/// Note that this is the only way to register native functions with a module.
	/// Panics if a native function with the same name has already been registered.
	pub fn add_function<F, const PARAM_C: usize, const RET_C: usize>(
		mut self,
		function: F,
		name: String,
	) -> Self
	where
		F: 'static + Send + FnMut([Word; PARAM_C]) -> [Word; RET_C],
	{
		assert!(
			!self.native_fns.iter().any(|nfn| name == nfn.name),
			"Tried to clobber native function: {name}"
		);

		let callback = Box::new(function);
		let (fat0, fat1) =
			unsafe { std::mem::transmute_copy::<_, (*const u8, *const u8)>(&callback) };
		let trampoline = C_TRAMPOLINES[RET_C * (MAX_RETS + 1) + PARAM_C];

		self.native_fns.push(NativeFn {
			name: name.clone(),
			param_len: PARAM_C,
			ret_len: RET_C,
			callback,
			trampoline,
		});

		self.inner.symbol(format!("__{name}_FAT0__"), fat0);
		self.inner.symbol(format!("__{name}_FAT1__"), fat1);
		self.inner.symbol(name, trampoline as *mut u8);

		self
	}

	#[must_use]
	pub fn build(mut self) -> Module {
		let mut module = JITModule::new(self.inner);
		let mut functions = IndexMap::with_capacity(self.native_fns.len());

		for nfn in self.native_fns.drain(..) {
			let mut sig = module.make_signature();

			sig.params = Vec::with_capacity(nfn.param_len);
			sig.params
				.resize(nfn.param_len, AbiParam::new(ClTypes::F32X4));
			sig.returns = Vec::with_capacity(nfn.ret_len);
			sig.returns
				.resize(nfn.ret_len, AbiParam::new(ClTypes::F32X4));

			let id = module
				.declare_function(&nfn.name, Linkage::Export, &sig)
				.expect("Failed to declare native function");

			let _ = functions.insert(
				nfn.name,
				FunctionInfo {
					_code: nfn.trampoline as *const c_void,
					_flags: FunctionFlags::empty(),
					_native: Some(nfn.callback),
					_id: id,
				},
			);
		}

		Module {
			_name: self.name,
			inner: Some(module),
			_functions: functions,
		}
	}
}

/// Intermediate storage from [`Builder`] to [`Module`].
struct NativeFn {
	name: String,
	param_len: usize,
	ret_len: usize,
	callback: NativeFnBox,
	trampoline: *const c_void,
}
