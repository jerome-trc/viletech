//! [`archery::SharedPointerKind`] implementation for [`triomphe::Arc`].

use std::{
	mem::ManuallyDrop,
	ops::{Deref, DerefMut},
};

use triomphe::Arc;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TArcK(ManuallyDrop<Arc<()>>);

impl TArcK {
	#[inline(always)]
	fn new_from_inner<T>(arc: Arc<T>) -> Self {
		Self(ManuallyDrop::new(unsafe { std::mem::transmute(arc) }))
	}

	#[inline(always)]
	unsafe fn take_inner<T>(self) -> Arc<T> {
		let arc: Arc<()> = ManuallyDrop::into_inner(self.0);

		std::mem::transmute(arc)
	}

	#[inline(always)]
	unsafe fn as_inner_ref<T>(&self) -> &Arc<T> {
		let arc_t: *const Arc<T> = (self.0.deref() as *const Arc<()>).cast::<Arc<T>>();

		// Static check to make sure we are not messing up the sizes.
		// This could happen if we allowed for `T` to be unsized, because it would need to be
		// represented as a wide pointer inside `Arc`.
		let _ = std::mem::transmute::<Arc<()>, Arc<T>>;

		&*arc_t
	}

	#[inline(always)]
	unsafe fn as_inner_mut<T>(&mut self) -> &mut Arc<T> {
		let arc_t: *mut Arc<T> = (self.0.deref_mut() as *mut Arc<()>).cast::<Arc<T>>();

		&mut *arc_t
	}
}

unsafe impl archery::SharedPointerKind for TArcK {
	fn new<T>(v: T) -> Self {
		Self({ ManuallyDrop::new(unsafe { std::mem::transmute(Arc::new(v)) }) })
	}

	fn from_box<T>(v: Box<T>) -> Self {
		Self({ ManuallyDrop::new(unsafe { std::mem::transmute(Arc::<T>::from(v)) }) })
	}

	unsafe fn as_ptr<T>(&self) -> *const T {
		Arc::as_ptr(self.as_inner_ref())
	}

	unsafe fn deref<T>(&self) -> &T {
		self.as_inner_ref::<T>().as_ref()
	}

	unsafe fn try_unwrap<T>(self) -> Result<T, Self> {
		Arc::try_unwrap(self.take_inner()).map_err(Self::new_from_inner)
	}

	unsafe fn get_mut<T>(&mut self) -> Option<&mut T> {
		Arc::get_mut(self.as_inner_mut())
	}

	unsafe fn make_mut<T: Clone>(&mut self) -> &mut T {
		Arc::make_mut(self.as_inner_mut())
	}

	unsafe fn strong_count<T>(&self) -> usize {
		Arc::count(self.as_inner_ref::<T>())
	}

	unsafe fn clone<T>(&self) -> Self {
		Self(ManuallyDrop::new(Arc::clone(self.as_inner_ref())))
	}

	unsafe fn drop<T>(&mut self) {
		std::ptr::drop_in_place::<Arc<T>>(self.as_inner_mut())
	}
}
