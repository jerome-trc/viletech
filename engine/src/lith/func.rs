//! Functions and everything related.

use std::{
	collections::hash_map::RandomState,
	ffi::c_void,
	hash::{BuildHasher, Hash, Hasher},
};

use cranelift_module::FuncId;

use super::{interop::NativeFnBox, Params, Returns};

/// Internal storage of the function code pointer and metadata.
#[derive(Debug)]
pub(super) struct FunctionInfo {
	/// This is a pointer to either a JIT-compiled function or a C trampoline
	/// for a native function. Either way, never try to deallocate it.
	pub(super) _code: *const c_void,
	pub(super) _flags: FunctionFlags,
	/// See [`hash_signature`].
	pub(super) _sig_hash: u64,
	/// The "source of truth" for native functions. Gets transmuted into two
	/// void pointers, each of which is stored as a global in the JIT binary,
	/// and then passed to a C-linkage trampoline.
	pub(super) _native: Option<NativeFnBox>,
	pub(super) _id: FuncId,
}

bitflags::bitflags! {
	pub struct FunctionFlags: u8 {
		/// This function has been marked as being compile-time evaluable.
		const CEVAL = 1 << 0;
	}
}

impl FunctionInfo {
	#[must_use]
	pub(crate) fn _is_native(&self) -> bool {
		self._native.is_some()
	}

	#[must_use]
	pub(crate) fn _is_ceval(&self) -> bool {
		self._flags.contains(FunctionFlags::CEVAL)
	}

	#[must_use]
	pub(crate) fn _has_signature<F, P, R, const PARAM_C: usize, const RET_C: usize>(&self) -> bool
	where
		F: 'static + Send + FnMut(P) -> R,
		P: Params<PARAM_C>,
		R: Returns<RET_C>,
	{
		hash_signature::<P, R, PARAM_C, RET_C>() == self._sig_hash
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
