use std::any::TypeId;

use rustc_hash::FxHashMap;

use crate::{compile::module::JitModule, rti};

/// Context for Lithica execution.
///
/// Fully re-entrant; Lith has no global state.
#[derive(Debug)]
pub struct Runtime {
	pub(crate) _function_rti: FxHashMap<String, rti::Store<rti::Function>>,
	pub(crate) _data_rti: FxHashMap<String, rti::Store<rti::DataObj>>,
	pub(crate) _type_rti: FxHashMap<String, rti::Store<rti::Rtti>>,
	/// Left untouched by the runtime; just needs to be here so that its
	/// memory does not get freed until it has no more users.
	#[allow(unused)]
	pub(crate) module: JitModule,

	pub(crate) userdata: *mut (),
	pub(crate) userdata_t: TypeId,
}

// SAFETY: these impls are only needed for `userdata`, which is never dereferenced
// by the runtime state itself. The caller provides guarantees that dereferencing
// the pointer is thread-safe.
unsafe impl Send for Runtime {}
unsafe impl Sync for Runtime {}

impl Runtime {
	/// Panics if `T` is not the current type of the userdata pointer stored using
	/// [`Self::set_userdata`].
	#[must_use]
	pub fn userdata<T: 'static>(&self) -> *mut T {
		assert_eq!(TypeId::of::<T>(), self.userdata_t);
		self.userdata.cast()
	}

	/// # Safety
	///
	/// The most recent call to [`Self::set_userdata`] must have provided a
	/// pointer to a `T`. Panics if `T` is not the current type of the userdata
	/// pointer stored using [`Self::set_userdata`], but only in debug mode.
	#[must_use]
	pub unsafe fn userdata_unchecked<T: 'static>(&self) -> *mut T {
		debug_assert_eq!(TypeId::of::<T>(), self.userdata_t);
		self.userdata.cast()
	}

	/// If this method has not yet been called for this `Runtime` instance,
	/// the userdata type will for the unit type (`()`) and the pointer will
	/// be null (and thus unsound to dereference!).
	pub fn set_userdata<T: 'static>(&mut self, ptr: *mut T) {
		self.userdata = ptr.cast();
		self.userdata_t = TypeId::of::<T>();
	}
}
