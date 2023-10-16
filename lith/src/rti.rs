//! Runtime information storage and handle types.

mod data;
mod func;
mod types;

use std::sync::atomic::{self, AtomicU32};

use crate::arena::APtr;

pub use self::{data::*, func::*, types::*};

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

impl<R> PartialEq for Handle<R> {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0
	}
}

impl<R> Eq for Handle<R> {}

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

impl std::ops::Deref for Handle<DataObj> {
	type Target = DataObj;

	fn deref(&self) -> &Self::Target {
		&self.0.inner
	}
}

impl std::ops::Deref for Handle<Rtti> {
	type Target = Rtti;

	fn deref(&self) -> &Self::Target {
		&self.0.inner
	}
}

// Internal details ////////////////////////////////////////////////////////////

/// A unit of runtime information (e.g. RTTI, pointer to a compiled function,
/// pointer to a JIT data object) allocated in an arena.
#[derive(Debug)]
pub(crate) struct Store<R> {
	inner: R,
	handles: AtomicU32,
}
