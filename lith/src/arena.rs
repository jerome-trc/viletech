//! The compiler toolchain's memory management code.

use std::{
	hash::{Hash, Hasher},
	ptr::NonNull,
};

use crossbeam::atomic::AtomicCell;

/// An "arena pointer".
///
/// Benefits from null pointer optimization.
#[derive(Debug)]
pub(crate) struct APtr<T>(NonNull<T>);

impl<T> APtr<T> {
	pub(crate) unsafe fn drop_in_place(self) {
		std::ptr::drop_in_place(self.0.as_ptr());
	}
}

impl<T> Clone for APtr<T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T> Copy for APtr<T> {}

impl<T> std::ops::Deref for APtr<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe { self.0.as_ref() }
	}
}

unsafe impl<T: Send> Send for APtr<T> {}
unsafe impl<T: Send + Sync> Sync for APtr<T> {}

/// A "compiler pointer" to an allocation in an [arena](bumpalo::Bump).
///
/// This type has no safety guarantees on its own. Its soundness relies on the
/// presumption of correct usage by the compiler code.
///
/// Does not benefit from null pointer optimization.
#[derive(Debug)]
pub(crate) struct CPtr<T>(
	// (RAT): It's weird that there's no way to get
	// null-pointer optimization for an `AtomicPtr`, isn't it?
	pub(crate) AtomicCell<Option<NonNull<T>>>,
);

impl<T> CPtr<T> {
	#[must_use]
	pub(crate) fn null() -> Self {
		Self(AtomicCell::new(None))
	}

	#[must_use]
	pub(crate) fn alloc(arena: &bumpalo::Bump, obj: T) -> Self {
		let m = arena.alloc(obj);
		let ret = CPtr::<T>::null();
		ret.store(NonNull::new(m as *mut T).unwrap());
		ret
	}

	pub(crate) fn store(&self, new: NonNull<T>) {
		self.0.store(Some(new));
	}

	/// Returns `None` if the pointer within is null.
	#[must_use]
	pub(crate) fn as_ref(&self) -> Option<&T> {
		unsafe { self.0.load().map(|nn| nn.as_ref()) }
	}

	#[must_use]
	pub(crate) fn as_ptr(&self) -> Option<NonNull<T>> {
		self.0.load()
	}

	#[must_use]
	pub(crate) fn into_inner(self) -> APtr<T> {
		APtr(self.0.load().unwrap())
	}
}

impl<T> PartialEq for CPtr<T> {
	fn eq(&self, other: &Self) -> bool {
		self.0.load() == other.0.load()
	}
}

impl<T> Eq for CPtr<T> {}

impl<T> Clone for CPtr<T> {
	fn clone(&self) -> Self {
		Self(AtomicCell::new(self.0.load()))
	}
}

impl<T> Hash for CPtr<T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.load().hash(state);
	}
}

unsafe impl<T: Send> Send for CPtr<T> {}
unsafe impl<T: Send + Sync> Sync for CPtr<T> {}

const _STATIC_ASSERT_APTR_CONSTRAINTS: () = {
	assert!(std::mem::size_of::<APtr<()>>() == std::mem::size_of::<*mut ()>());
	assert!(AtomicCell::<Option<NonNull<()>>>::is_lock_free());
};
