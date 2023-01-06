//! Functions and everything related.

use std::{
	collections::hash_map::RandomState,
	ffi::c_void,
	hash::{BuildHasher, Hash, Hasher},
	marker::PhantomData,
	sync::Arc,
};

use cranelift_module::FuncId;

use super::{interop::NativeFnBox, module, Params, Returns};

/// Internal storage of the function code pointer and metadata.
#[derive(Debug)]
pub(super) struct FunctionInfo {
	/// This is a pointer to either a JIT-compiled function or a C trampoline
	/// for a native function. Either way, never try to deallocate it.
	pub(super) _code: *const c_void,
	pub(super) flags: FunctionFlags,
	/// See [`hash_signature`].
	pub(super) sig_hash: u64,
	/// The "source of truth" for native functions. Gets transmuted into two
	/// void pointers, each of which is stored as a global in the JIT binary,
	/// and then passed to a C-linkage trampoline.
	pub(super) native: Option<NativeFnBox>,
	pub(super) _id: FuncId,
}

bitflags::bitflags! {
	pub struct FunctionFlags: u8 {
		/// This function has been marked as being compile-time evaluable.
		const CEVAL = 1 << 0;
	}
}

/// Take all parameter type IDs in order and feed them to a hasher.
/// Then, take all return type IDs in order and feed them to that same hasher.
#[must_use]
pub(super) fn hash_signature<P, R, const PARAM_C: usize, const RET_C: usize>() -> u64
where
	P: Params<PARAM_C>,
	R: Returns<RET_C>,
{
	let mut hasher = RandomState::default().build_hasher();

	for t_id in P::type_ids() {
		t_id.hash(&mut hasher);
	}

	for t_id in R::type_ids() {
		t_id.hash(&mut hasher);
	}

	hasher.finish()
}

/// This is a type similar in purpose to [`Handle`], but separate to enable
/// generic parameterization.
///
/// [`Handle`]: super::module::Handle
pub struct Function<P, R, const PARAM_C: usize, const RET_C: usize>
where
	P: Params<PARAM_C>,
	R: Returns<RET_C>,
{
	module: Arc<module::Inner>,
	index: usize,
	#[allow(unused)]
	phantom: PhantomData<Box<dyn 'static + Send + FnMut(P) -> R>>,
}

impl<P: Params<PARAM_C>, R: Returns<RET_C>, const PARAM_C: usize, const RET_C: usize>
	Function<P, R, PARAM_C, RET_C>
{
	pub fn call(&self, _: P) -> R {
		unimplemented!()
	}

	#[must_use]
	pub fn is_native(&self) -> bool {
		self.inner().native.is_some()
	}

	#[must_use]
	pub fn is_ceval(&self) -> bool {
		self.inner().flags.contains(FunctionFlags::CEVAL)
	}

	#[must_use]
	pub(super) fn inner(&self) -> &FunctionInfo {
		unsafe {
			// Note that `unwrap_unchecked` has an internal debug assertion
			self.module
				.functions
				.get_index(self.index)
				.unwrap_unchecked()
				.1
		}
	}

	#[must_use]
	pub(super) fn new(module: Arc<module::Inner>, index: usize) -> Self {
		Self {
			module,
			index,
			phantom: PhantomData,
		}
	}
}
