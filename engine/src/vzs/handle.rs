use std::{
	marker::PhantomData,
	sync::{Arc, Weak},
};

use super::{abi::Abi, func::TFunc, Error, Function, Symbol};

/// Thin wrapper around an [`Arc`] pointing to an [`Symbol`]. Attaching a generic
/// type allows the pointer to be pre-downcast, so dereferencing is as fast as
/// with any other pointer with no unsafe code required.
#[derive(Debug)]
pub struct Handle<S: Symbol>(Arc<S>, PhantomData<S>);

impl Handle<Function> {
	pub fn downcast<A, R>(&self) -> Result<Handle<TFunc<A, R>>, Error>
	where
		A: Abi,
		R: Abi,
	{
		if self.has_signature::<A, R>() {
			Ok(Handle(
				Arc::new(TFunc {
					source: self.clone(),
					phantom: PhantomData,
				}),
				PhantomData,
			))
		} else {
			Err(Error::SignatureMismatch)
		}
	}
}

impl<S: 'static + Symbol> std::ops::Deref for Handle<S> {
	type Target = Arc<S>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<S: Symbol> PartialEq for Handle<S> {
	/// Check that these are two handles to the same symbol in the same module.
	fn eq(&self, other: &Self) -> bool {
		Arc::ptr_eq(&self.0, &other.0)
	}
}

impl<S: Symbol> Eq for Handle<S> {}

impl<S: Symbol> Clone for Handle<S> {
	fn clone(&self) -> Self {
		Self(self.0.clone(), PhantomData)
	}
}

impl<S: Symbol> From<&Arc<S>> for Handle<S> {
	fn from(value: &Arc<S>) -> Self {
		Handle(value.clone(), PhantomData)
	}
}

// SAFETY: See safety disclaimer for `Module`.
unsafe impl<S: Symbol> Send for Handle<S> {}
unsafe impl<S: Symbol> Sync for Handle<S> {}

/// Internal handle. Like [`Handle`] but [`Weak`], allowing inter-symbol
/// relationships (without preventing in-place mutation or removal) in a way
/// that can't leak.
#[derive(Debug, Clone)]
pub struct InHandle<S: Symbol>(Weak<S>, PhantomData<S>);

impl<S: Symbol> PartialEq for InHandle<S> {
	/// Check that these are two handles to the same symbol in the same module.
	fn eq(&self, other: &Self) -> bool {
		Weak::ptr_eq(&self.0, &other.0)
	}
}

impl<S: Symbol> Eq for InHandle<S> {}

// SAFETY: See safety disclaimer for `Module`.
unsafe impl<S: Symbol> Send for InHandle<S> {}
unsafe impl<S: Symbol> Sync for InHandle<S> {}
