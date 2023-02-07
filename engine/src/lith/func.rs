//! Function wrappers and information, as well as interop details.

use std::{
	hash::{Hash, Hasher},
	marker::PhantomData,
	sync::Arc,
};

use fasthash::SeaHasher;

use super::{abi::Abi, inode, Handle, Runtime, Symbol};

/// Pointer to a function, whether native or compiled.
#[derive(Debug)]
pub struct Function {
	pub(super) code: Arc<inode::Tree>,
	pub(super) flags: Flags,
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

/// Typed function handle.
pub struct TFunc<A: Abi, R: Abi> {
	pub(super) source: Handle<Function>,
	#[allow(unused)]
	pub(super) phantom: PhantomData<fn(A) -> R>,
}

impl<A: Abi, R: Abi> std::fmt::Debug for TFunc<A, R> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("TFunc")
			.field("source", &self.source)
			.finish()
	}
}

impl<A, R> TFunc<A, R>
where
	A: Abi,
	R: Abi,
{
	pub fn call(&self, runtime: &mut Runtime, args: A) -> R {
		self.source.code.eval(runtime, args)
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
