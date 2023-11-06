//! The compiler toolchain's memory management code.

use std::{
	hash::{Hash, Hasher},
	ptr::NonNull,
};

use crossbeam::atomic::AtomicCell;

/// An "arena pointer".
///
/// This type has no safety guarantees on its own. Its soundness relies on the
/// presumption of correct usage by the compiler code.
///
/// Benefits from null pointer optimization.
#[derive(Debug)]
pub struct APtr<T>(NonNull<T>);

impl<T> APtr<T> {
	#[must_use]
	pub(crate) fn new(ptr: NonNull<T>) -> Self {
		Self(ptr)
	}
}

impl<T> PartialEq for APtr<T> {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0
	}
}

impl<T> Eq for APtr<T> {}

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

impl<T> std::borrow::Borrow<T> for APtr<T> {
	fn borrow(&self) -> &T {
		std::ops::Deref::deref(self)
	}
}

impl<T> Hash for APtr<T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.hash(state);
	}
}

impl<T> From<NPtr<T>> for APtr<T> {
	fn from(value: NPtr<T>) -> Self {
		Self(value.0.load().unwrap())
	}
}

unsafe impl<T: Send> Send for APtr<T> {}
unsafe impl<T: Send + Sync> Sync for APtr<T> {}

/// An atomic "nullable pointer" to an allocation in an [arena](bumpalo::Bump).
///
/// The multithreaded architecture used by LithC benefits heavily from immutability,
/// but some deferral of initialization is sometimes necessary. These pointers
/// facilitate that late initialization.
///
/// This type has no safety guarantees on its own. Its soundness relies on the
/// presumption of correct usage by the compiler code.
///
/// Does not benefit from null pointer optimization.
#[derive(Debug)]
pub(crate) struct NPtr<T>(AtomicCell<Option<NonNull<T>>>);

impl<T> NPtr<T> {
	#[must_use]
	pub(crate) fn new(aptr: APtr<T>) -> Self {
		Self(AtomicCell::new(Some(aptr.0)))
	}

	#[must_use]
	pub(crate) fn null() -> Self {
		Self(AtomicCell::new(None))
	}

	pub(crate) fn store(&self, new: APtr<T>) {
		self.0.store(Some(new.0));
	}

	/// Returns `None` if the pointer within is null.
	#[must_use]
	pub(crate) fn try_ref(&self) -> Option<&T> {
		unsafe { self.0.load().map(|nn| nn.as_ref()) }
	}

	/// Panics if the pointer within is null.
	#[must_use]
	pub(crate) fn as_ref(&self) -> &T {
		self.try_ref().unwrap()
	}

	#[must_use]
	pub(crate) fn as_ptr(&self) -> Option<NonNull<T>> {
		self.0.load()
	}
}

impl<T> PartialEq for NPtr<T> {
	fn eq(&self, other: &Self) -> bool {
		self.0.load() == other.0.load()
	}
}

impl<T> PartialEq<APtr<T>> for NPtr<T> {
	fn eq(&self, other: &APtr<T>) -> bool {
		self.0.load().is_some_and(|nn| nn == other.0)
	}
}

impl<T> Eq for NPtr<T> {}

impl<T> Clone for NPtr<T> {
	fn clone(&self) -> Self {
		Self(AtomicCell::new(self.0.load()))
	}
}

impl<T> Hash for NPtr<T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.load().hash(state);
	}
}

unsafe impl<T: Send> Send for NPtr<T> {}
unsafe impl<T: Send + Sync> Sync for NPtr<T> {}

/// Like [`APtr`] but "owning".
#[derive(Debug)]
pub struct OPtr<T>(NonNull<T>);

impl<T> OPtr<T> {
	#[must_use]
	pub(crate) fn alloc(arena: &bumpalo::Bump, obj: T) -> Self {
		let m = arena.alloc(obj);
		Self(NonNull::new(m as *mut T).unwrap())
	}

	#[must_use]
	pub(crate) unsafe fn read(self) -> T {
		std::ptr::read(self.0.as_ptr())
	}
}

impl<T> std::ops::Deref for OPtr<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe { self.0.as_ref() }
	}
}

impl<T> std::borrow::Borrow<T> for OPtr<T> {
	fn borrow(&self) -> &T {
		std::ops::Deref::deref(self)
	}
}

impl<'p, T> From<&'p OPtr<T>> for APtr<T> {
	fn from(value: &'p OPtr<T>) -> Self {
		Self(value.0)
	}
}

impl<T> From<APtr<T>> for OPtr<T> {
	fn from(value: APtr<T>) -> Self {
		Self(value.0)
	}
}

impl<T> PartialEq for OPtr<T> {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0
	}
}

impl<T> PartialEq<APtr<T>> for OPtr<T> {
	fn eq(&self, other: &APtr<T>) -> bool {
		self.0 == other.0
	}
}

impl<T> Eq for OPtr<T> {}

impl<T> Hash for OPtr<T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.hash(state);
	}
}

impl<T> Drop for OPtr<T> {
	fn drop(&mut self) {
		unsafe {
			std::ptr::drop_in_place(self.0.as_ptr());
		}
	}
}

unsafe impl<T> Send for OPtr<T> {}
unsafe impl<T> Sync for OPtr<T> {}

const _STATIC_ASSERT_CONSTRAINTS: () = {
	assert!(AtomicCell::<Option<NonNull<u8>>>::is_lock_free());

	assert!(std::mem::size_of::<APtr<u8>>() == std::mem::size_of::<*mut u8>());
	assert!(std::mem::size_of::<APtr<u8>>() == std::mem::size_of::<Option<APtr<u8>>>(),);

	assert!(std::mem::size_of::<NPtr<u8>>() == std::mem::size_of::<*mut u8>());
};
