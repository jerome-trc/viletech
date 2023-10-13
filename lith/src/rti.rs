//! Runtime information storage and handle types.

use std::{
	ffi::c_void,
	hash::Hasher,
	marker::PhantomData,
	mem::ManuallyDrop,
	sync::atomic::{self, AtomicU32},
};

use cranelift_module::{DataId, FuncId};
use rustc_hash::FxHasher;

use crate::{arena::APtr, interop::JitFn};

// Public interface ////////////////////////////////////////////////////////////

/// A reference-counted handle to a unit of runtime information.
///
/// Note that "reference-counted" in this context is not the same as with a
/// [`std::sync::Arc`]! The stored info is not dropped when all handles drop.
/// The reference count is only used to prevent holding pointers to the arenas
/// used by the compiler; if the compiled runtime drops along with all its arenas
/// before all handles drop, the program will panic to prevent use-after-free.
#[derive(Debug)]
pub struct Handle<R>(APtr<Store<R>>);

impl<R> Clone for Handle<R> {
	fn clone(&self) -> Self {
		self.0.handles.fetch_add(1, atomic::Ordering::Relaxed);
		Self(self.0)
	}
}

impl<R> Drop for Handle<R> {
	fn drop(&mut self) {
		self.0.handles.fetch_sub(1, atomic::Ordering::Release);
	}
}

impl std::ops::Deref for Handle<Function> {
	type Target = Function;

	fn deref(&self) -> &Self::Target {
		&self.0.inner
	}
}

#[derive(Debug)]
pub struct Function {
	pub(crate) ptr: *const c_void,
	pub(crate) id: FuncId,
	pub(crate) sig_hash: u64,
}

impl Function {
	#[must_use]
	pub fn downcast<F: JitFn>(&self) -> Option<TFn<F>> {
		let mut hasher = FxHasher::default();
		F::sig_hash(&mut hasher);

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

#[derive(Debug)]
pub struct Data {
	ptr: *const u8,
	size: usize,
	id: DataId,
	immutable: bool,
}

// Private details /////////////////////////////////////////////////////////////

/// A unit of runtime information (e.g. RTTI, pointer to a compiled function,
/// pointer to a JIT data object) allocated in an arena.
#[derive(Debug)]
pub(crate) struct Store<R> {
	pub(crate) inner: R,
	pub(crate) handles: AtomicU32,
}

impl<R> Drop for Store<R> {
	fn drop(&mut self) {
		assert_eq!(self.handles.load(atomic::Ordering::Acquire), 0);
	}
}

pub(crate) struct Record {
	pub(crate) tag: StoreTag,
	pub(crate) inner: StoreUnion,
}

/// Gets discriminated with [`StoreTag`].
pub(crate) union StoreUnion {
	pub(crate) func: APtr<Store<Function>>,
	pub(crate) data: APtr<Store<Data>>,
	pub(crate) typedef: APtr<Store<()>>,
}

/// Separated discriminant for [`StoreUnion`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StoreTag {
	Function,
	Data,
	Type,
}

impl std::fmt::Debug for Record {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Record")
			.field("tag", &self.tag)
			.field("data", unsafe {
				match self.tag {
					StoreTag::Function => &self.inner.func,
					StoreTag::Data => &self.inner.data,
					StoreTag::Type => &self.inner.typedef,
				}
			})
			.finish()
	}
}

impl Drop for Record {
	fn drop(&mut self) {
		unsafe {
			match self.tag {
				StoreTag::Function => self.inner.func.drop_in_place(),
				StoreTag::Data => self.inner.data.drop_in_place(),
				StoreTag::Type => self.inner.typedef.drop_in_place(),
			}
		}
	}
}
