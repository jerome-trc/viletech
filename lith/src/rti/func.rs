//! Runtime function information.

use std::{
	hash::{Hash, Hasher},
	marker::PhantomData,
};

use cranelift_module::FuncId;
use rustc_hash::FxHasher;

use crate::interop::JitFn;

use super::Handle;

#[derive(Debug)]
pub struct Function {
	pub(crate) ptr: *const (),
	pub(crate) id: FuncId,
	pub(crate) sig_hash: u64,
}

impl Function {
	#[must_use]
	pub fn id(&self) -> FuncId {
		self.id
	}

	#[must_use]
	pub fn downcast<F: JitFn>(&self) -> Option<TFn<F>> {
		let mut hasher = FxHasher::default();

		F::PARAMS.hash(&mut hasher);
		F::RETURNS.hash(&mut hasher);

		if self.sig_hash == hasher.finish() {
			return Some(TFn(self, PhantomData));
		}

		None
	}
}

/// A strongly-typed reference to a [JIT function pointer](Function).
#[derive(Debug)]
pub struct TFn<'f, F: JitFn>(&'f Function, PhantomData<F>);

impl<F: JitFn> std::ops::Deref for TFn<'_, F> {
	type Target = F;

	fn deref(&self) -> &Self::Target {
		// SAFETY: the type of the function behind this reference was already verified.
		unsafe { &*self.0.ptr.cast::<F>() }
	}
}

/// A strongly-typed [handle](Handle) to a [JIT function pointer](Function).
#[derive(Debug)]
pub struct TFnHandle<F: JitFn>(Handle<Function>, PhantomData<F>);

impl<F: JitFn> std::ops::Deref for TFnHandle<F> {
	type Target = F;

	fn deref(&self) -> &Self::Target {
		// SAFETY: the type of the function behind this handle was already verified.
		unsafe { &*self.0.ptr.cast::<F>() }
	}
}
