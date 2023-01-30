//! Function wrappers and information, as well as interop details.

use std::{
	ffi::c_void,
	hash::{Hash, Hasher},
	marker::PhantomData,
};

use cranelift_module::FuncId;
use fasthash::SeaHasher;

use super::{abi::Abi, Handle, Symbol};

/// Pointer to a function, whether native or compiled.
#[derive(Debug)]
pub struct Function {
	/// Never try to de-allocate this.
	pub(super) code: *const c_void,
	pub(super) flags: Flags,
	pub(super) _id: FuncId,
	/// See [`Self::hash_signature`].
	pub(super) sig_hash: u64,
}

bitflags::bitflags! {
	/// Qualifiers and annotations.
	pub struct Flags: u8 {
		/// This function has been marked as being compile-time evaluable.
		const CEVAL = 1 << 0;
	}
}

impl Function {
	#[must_use]
	pub fn has_signature<A, R>(&self) -> bool
	where
		A: Abi,
		R: Abi,
	{
		Self::hash_signature::<A, R>() == self.sig_hash
	}

	#[must_use]
	pub fn flags(&self) -> Flags {
		self.flags
	}

	#[must_use]
	fn hash_signature<A, R>() -> u64
	where
		A: Abi,
		R: Abi,
	{
		let mut hasher = SeaHasher::default();
		A::type_id_hash(&mut hasher);
		R::type_id_hash(&mut hasher);
		hasher.finish()
	}
}

// SAFETY: See safety disclaimer for `Module`
unsafe impl Send for Function {}
unsafe impl Sync for Function {}

impl Symbol for Function {}

/// Typed function.
pub struct TFunc<A: Abi, R: Abi> {
	/// Copied from the source [`Function`].
	/// *Definitely* never try to de-allocate this.
	pub(super) code: *const c_void,
	#[allow(unused)]
	pub(super) source: Handle<Function>,
	#[allow(unused)]
	pub(super) phantom: PhantomData<fn(A) -> R>,
}

impl<A, R> TFunc<A, R>
where
	A: Abi,
	R: Abi,
{
	pub fn call(&self, args: A) -> R {
		let a = args.to_words();
		// SAFETY: For the public interface to have gotten to this point, it must
		// have performed a check that signature types match
		let func = unsafe { std::mem::transmute::<_, fn(A::Repr) -> R::Repr>(self.code) };
		let r = func(a);
		R::from_words(r)
	}
}

impl<A, R> std::fmt::Debug for TFunc<A, R>
where
	A: Abi,
	R: Abi,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("TFunc").field("code", &self.code).finish()
	}
}

// SAFETY: See safety disclaimer for `Module`
unsafe impl<A, R> Send for TFunc<A, R>
where
	A: Abi,
	R: Abi,
{
}
unsafe impl<A, R> Sync for TFunc<A, R>
where
	A: Abi,
	R: Abi,
{
}

impl<A, R> Symbol for TFunc<A, R>
where
	A: Abi,
	R: Abi,
{
}
