//! "Asset" is the catch-all term for any data unit the catalog can store.

mod audio;
mod gameplay;
mod map;
mod visual;

use std::{any::TypeId, marker::PhantomPinned, sync::Arc};

use dashmap::mapref::one::RefMut as DashMapRefMut;

pub use audio::*;
pub use gameplay::*;
pub use map::*;
pub use visual::*;

use super::{detail::AssetKey, AssetError, Handle};

/// A dynamically-typed storage for a single asset.
#[derive(Debug)]
pub struct Record {
	/// Note to reader: leave this here, even if not doing any pinning.
	#[allow(unused)]
	pin: PhantomPinned,
	pub(super) id: String,
	pub(super) data: Box<dyn Asset>,
	// Q: Could this safely and painlessly be made into a DST?
}

impl Record {
	#[must_use]
	pub fn id(&self) -> &str {
		&self.id
	}

	/// Check this record's storage type.
	#[must_use]
	pub fn is<T: 'static>(&self) -> bool {
		self.data.as_any().is::<T>()
	}

	/// Returns [`AssetError::TypeMismatch`] if the storage type isn't `T`.
	///
	/// [`AssetError::TypeMismatch`]: super::AssetError::TypeMismatch
	pub fn downcast<T: 'static>(&self) -> Result<&T, AssetError> {
		self.data
			.as_any()
			.downcast_ref::<T>()
			.ok_or_else(|| AssetError::TypeMismatch {
				expected: self.data.type_id(),
				given: TypeId::of::<T>(),
			})
	}

	/// Returns [`AssetError::TypeMismatch`] if the storage type isn't `T`.
	///
	/// [`AssetError::TypeMismatch`]: super::AssetError::TypeMismatch
	pub fn handle<T: 'static + Asset>(self: &Arc<Self>) -> Result<Handle<T>, AssetError> {
		if self.data.as_any().is::<T>() {
			Ok(Handle::new(self))
		} else {
			Err(AssetError::TypeMismatch {
				expected: self.data.as_any().type_id(),
				given: TypeId::of::<T>(),
			})
		}
	}
}

impl PartialEq for Record {
	fn eq(&self, other: &Self) -> bool {
		std::ptr::eq(self, other)
	}
}

impl Eq for Record {}

/// See [`Catalog::try_mutate`](super::Catalog::try_mutate).
pub struct RefMut<'cat>(pub(super) DashMapRefMut<'cat, AssetKey, Arc<Record>>);

// Newtype this so the record is mutable but the key is not

impl std::ops::Deref for RefMut<'_> {
	type Target = Record;

	fn deref(&self) -> &Self::Target {
		<Arc<Record> as AsRef<Record>>::as_ref(self.0.value())
	}
}

impl std::ops::DerefMut for RefMut<'_> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		Arc::get_mut(self.0.value_mut()).expect("An asset record mutation failed unexpectedly.")
	}
}

pub trait Asset: private::Sealed {}

mod private {
	use std::any::Any;

	pub trait Sealed: Any + Send + Sync + std::fmt::Debug {
		/// Boilerplate allowing upcasting from `Asset` to `Any`.
		#[must_use]
		fn as_any(&self) -> &dyn Any;
	}

	impl<T> Sealed for T
	where
		T: Any + Send + Sync + std::fmt::Debug,
	{
		#[inline]
		fn as_any(&self) -> &dyn Any {
			self
		}
	}
}
