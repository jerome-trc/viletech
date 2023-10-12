//! Runtime information storage and handle types.

use std::{
	mem::ManuallyDrop,
	sync::atomic::{self, AtomicU32},
};

use crate::arena::APtr;

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

pub(crate) struct Record {
	pub(crate) tag: StoreTag,
	pub(crate) inner: StoreUnion,
}

/// Gets discriminated with [`StoreTag`].
pub(crate) union StoreUnion {
	// TODO: determine backend.
	pub(crate) func: ManuallyDrop<APtr<Store<()>>>,
	pub(crate) data: ManuallyDrop<APtr<Store<()>>>,
	pub(crate) typedef: ManuallyDrop<APtr<Store<()>>>,
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
				StoreTag::Function => ManuallyDrop::drop(&mut self.inner.func),
				StoreTag::Data => ManuallyDrop::drop(&mut self.inner.data),
				StoreTag::Type => ManuallyDrop::drop(&mut self.inner.typedef),
			}
		}
	}
}
