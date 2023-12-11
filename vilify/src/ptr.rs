//! Pointer types to ease working with arenas.
//!
//! None of these types have any safety guarantees of their own. Soundness of use
//! relies on the presumumption of correct usage by the transpiler code and a public
//! interface which prevents crate clients from being exposed to invalid pointer.

use std::{
	hash::{Hash, Hasher},
	ptr::NonNull,
};

use crossbeam::atomic::AtomicCell;

#[derive(Debug)]
pub(crate) struct Shared<T>(NonNull<T>);

unsafe impl<T: Send> Send for Shared<T> {}
unsafe impl<T: Send + Sync> Sync for Shared<T> {}

impl<T> std::ops::Deref for Shared<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe { self.0.as_ref() }
	}
}

impl<T> PartialEq for Shared<T> {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0
	}
}

impl<T> Eq for Shared<T> {}

impl<T> Clone for Shared<T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T> Copy for Shared<T> {}

impl<T> Hash for Shared<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[derive(Debug)]
pub(crate) struct Nullable<T>(AtomicCell<Option<Shared<T>>>);

#[derive(Debug)]
pub(crate) struct Owned<T>(Shared<T>);

impl<T> Owned<T> {
	#[must_use]
	pub(crate) fn alloc(arena: &bumpalo::Bump, obj: T) -> Self {
		let m = arena.alloc(obj);
		Self(Shared(NonNull::new(m as *mut T).unwrap()))
	}
}

impl<T> std::ops::Deref for Owned<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe { self.0.0.as_ref() }
	}
}

impl<T> std::borrow::Borrow<T> for Owned<T> {
	fn borrow(&self) -> &T {
		std::ops::Deref::deref(self)
	}
}

impl<T: PartialEq> PartialEq for Owned<T> {
    fn eq(&self, other: &Self) -> bool {
        std::ops::Deref::deref(self) == std::ops::Deref::deref(other)
    }
}

impl<T: PartialEq> Eq for Owned<T> {}

impl<T: Hash> Hash for Owned<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.0.hash(state);
    }
}

impl<'p, T> From<&'p Owned<T>> for Shared<T> {
	fn from(value: &'p Owned<T>) -> Self {
		Self(value.0.0)
	}
}

impl<T> Drop for Owned<T> {
	fn drop(&mut self) {
		unsafe { std::ptr::drop_in_place(self.0 .0.as_ptr()) }
	}
}

#[derive(Debug)]
pub(crate) struct Atomic<T>(Nullable<T>);

impl<T> Drop for Atomic<T> {
	fn drop(&mut self) {
		match self.0 .0.load() {
			Some(ptr) => unsafe { std::ptr::drop_in_place(ptr.0.as_ptr()) },
			None => todo!(),
		}
	}
}

const _STATIC_ASSERT_CONSTRAINTS: () = {
	assert!(AtomicCell::<Option<NonNull<u8>>>::is_lock_free());

	assert!(std::mem::size_of::<Shared<u8>>() == std::mem::size_of::<*mut u8>());
	assert!(std::mem::size_of::<Shared<u8>>() == std::mem::size_of::<Option<Shared<u8>>>(),);

	assert!(std::mem::size_of::<Nullable<u8>>() == std::mem::size_of::<*mut u8>());
};
