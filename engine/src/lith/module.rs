//! A LithScript module is a single linkage unit.
//!
//! This is an equivalent concept to modules in LLVM, Rust, and Cranelift, but it
//! inherits the ZScript behavior of being compiled from an arbitrary number of
//! source files, rather than just one. Hence, `lith` is a module (for language
//! support), `vile` is a module for native engine functionality, et cetera.
//!
//! To get started, [create a `Builder`]. Register all native functions and data
//! objects with it, and then use it to emit an [`OpenModule`]. This can then
//! have script source compiled into it. When you're ready to start running code,
//! close the `OpenModule` to get a [`Module`].
//!
//! [create a `Builder`]: Builder::new

use std::{ffi::c_void, marker::PhantomData, sync::Arc};

use cranelift::prelude::{types as ClTypes, AbiParam};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module as CraneliftModule};
use indexmap::IndexMap;

use crate::lith::{interop::C_TRAMPOLINES, MAX_RETS};

use super::{
	func::{Function, FunctionFlags, FunctionInfo},
	interop::NativeFnBox,
	word::Word,
	Error, Params, Returns,
};

/// This can be cheaply cloned since it wraps an `Arc`. The actual compiled data
/// is accessed lock-free, since it's stored immutably.
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Module(Arc<Inner>);

impl Module {
	/// "Native" modules are those with symbols pre-registered upon construction.
	#[must_use]
	pub fn is_native(&self) -> bool {
		self.0.native
	}

	/// Re-open the module, re-enabling modification to its functions and data.
	/// If this is not the only outstanding handle to this module's data, then
	/// `Err(Self)` will be returned.
	pub fn open(self) -> Result<OpenModule, Self> {
		match Arc::try_unwrap(self.0) {
			Ok(inner) => Ok(OpenModule(inner)),
			Err(arc) => Err(Self(arc)),
		}
	}

	pub fn get_function<P, R, const PARAM_C: usize, const RET_C: usize>(
		&self,
		name: &str,
	) -> Result<Function<P, R, PARAM_C, RET_C>, Error>
	where
		P: Params<PARAM_C>,
		R: Returns<RET_C>,
	{
		match self.0.functions.get_full(name) {
			Some((index, _, func)) => {
				if super::func::hash_signature::<P, R, PARAM_C, RET_C>() == func.sig_hash {
					Ok(Function::new(self.0.clone(), index))
				} else {
					Err(Error::SignatureMismatch)
				}
			}
			None => Err(Error::UnknownIdentifier),
		}
	}
}

impl PartialEq for Module {
	/// Check that these two objects are backed by the same Cranelift module.
	fn eq(&self, other: &Self) -> bool {
		Arc::ptr_eq(&self.0, &other.0)
	}
}

// Safety: Lith makes the following guarantees:
// - An `OpenModule` is mutable, but it can not be safely moved across threads.
// - A closed `Module` can be safely moved across threads, and its functions can
// be called, but not in any way that mutates its inner state.
unsafe impl Send for Module {}
unsafe impl Sync for Module {}

impl std::fmt::Debug for OpenModule {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Module")
			.field("name", &self.0.name)
			.field("native", &self.0.native)
			.field("functions", &self.0.functions)
			.finish()
	}
}

/// A module has to be "open" for compiling code into it; it must be "closed" via
/// [`close`](OpenModule::close) in order to be used by a runtime.
#[repr(transparent)]
pub struct OpenModule(Inner);

impl OpenModule {
	#[must_use]
	pub fn close(self) -> Module {
		Module(Arc::new(self.0))
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
	pub fn add_function<F, P, R, const PARAM_C: usize, const RET_C: usize>(
		mut self,
		mut function: F,
		name: String,
	) -> Self
	where
		F: 'static + Send + FnMut(P) -> R,
		P: Params<PARAM_C>,
		R: Returns<RET_C>,
	{
		assert!(
			!self.native_fns.iter().any(|nfn| name == nfn.name),
			"Tried to clobber native function: {name}"
		);

		let wrapper = Box::new(move |args: [Word; PARAM_C]| {
			let rs_args = P::decompose(args);
			let rs_rets = function(rs_args);
			rs_rets.compose()
		});

		let (fat0, fat1) =
			unsafe { std::mem::transmute_copy::<_, (*const u8, *const u8)>(&wrapper) };
		let trampoline = C_TRAMPOLINES[RET_C * (MAX_RETS + 1) + PARAM_C];

		self.native_fns.push(NativeFn {
			name: name.clone(),
			wrapper,
			trampoline,
			sig_hash: super::func::hash_signature::<P, R, PARAM_C, RET_C>(),
			param_len: PARAM_C,
			ret_len: RET_C,
		});

		self.inner.symbol(format!("__{name}_FAT0__"), fat0);
		self.inner.symbol(format!("__{name}_FAT1__"), fat1);
		self.inner.symbol(name, trampoline as *mut u8);

		self
	}

	#[must_use]
	pub fn build(mut self) -> OpenModule {
		let mut module = JITModule::new(self.inner);
		let mut functions = IndexMap::with_capacity(self.native_fns.len());

		let native = !self.native_fns.is_empty();

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
					flags: FunctionFlags::empty(),
					sig_hash: nfn.sig_hash,
					native: Some(nfn.wrapper),
					_id: id,
				},
			);
		}

		OpenModule(Inner {
			name: self.name,
			native,
			inner: Some(module),
			functions,
		})
	}
}

impl std::fmt::Debug for Builder {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Builder")
			.field("name", &self.name)
			.field("native_fns", &self.native_fns)
			.finish()
	}
}

pub(super) struct Inner {
	pub(super) name: String,
	/// Is `true` if this module had any native symbols loaded into it.
	/// Special rules are applied when performing semantic checks on source being
	/// compiled into a native module.
	pub(super) native: bool,
	/// Freeing the module's memory requires pass-by-value, but [`Drop::drop`]
	/// takes a mutable reference, so an indirection is needed here to allow
	/// removing the module upon destruction. It is never `None`, ever.
	pub(super) inner: Option<JITModule>,
	/// Indices are guaranteed to be stable, since this map is append-only.
	pub(super) functions: IndexMap<String, FunctionInfo>,
}

impl std::fmt::Debug for Inner {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Inner")
			.field("name", &self.name)
			.field("native", &self.native)
			.field("functions", &self.functions)
			.finish()
	}
}

impl Drop for Inner {
	fn drop(&mut self) {
		let inner = self.inner.take();

		debug_assert!(inner.is_some());

		unsafe {
			inner.unwrap_unchecked().free_memory();
		}
	}
}

/// Proxy for access to some kind of symbol in a LithScript [`Module`].
///
/// Wraps an [`Arc`], much like the module itself, so it's easy to store it far
/// from the module itself. If you need to re-open the module for alteration later,
/// all outstanding handles pointing it will need to be dropped first.
#[derive(Debug, Clone)]
pub struct Handle<T> {
	pub(self) module: Arc<Inner>,
	pub(self) index: usize,
	_phantom: PhantomData<T>,
}

impl<T> PartialEq for Handle<T> {
	/// Check that these two handles point to the same object in the same module.
	fn eq(&self, other: &Self) -> bool {
		Arc::ptr_eq(&self.module, &other.module) && self.index == other.index
	}
}

/// Intermediate storage from [`Builder`] to [`Module`].
#[derive(Debug)]
struct NativeFn {
	name: String,
	wrapper: NativeFnBox,
	trampoline: *const c_void,
	param_len: usize,
	ret_len: usize,
	sig_hash: u64,
}
