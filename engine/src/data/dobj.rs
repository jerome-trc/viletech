//! "Data objects". Things like levels, actor blueprints, and sounds.

mod actor;
mod audio;
mod level;
mod visual;

use std::{
	any::{Any, TypeId},
	collections::HashMap,
	sync::{Arc, Weak},
};

use once_cell::sync::Lazy;

pub use self::{actor::*, audio::*, level::*, visual::*};

use super::Catalog;

pub trait Datum: 'static + Any + Send + Sync + std::fmt::Debug {}

#[derive(Debug)]
pub struct Store<D: Datum> {
	id: String,
	inner: D,
}

impl<D: Datum> Store<D> {
	#[must_use]
	pub fn new(id: String, datum: D) -> Self {
		Self { id, inner: datum }
	}

	#[must_use]
	pub fn id(&self) -> &str {
		&self.id
	}

	#[must_use]
	pub fn inner(&self) -> &D {
		&self.inner
	}

	#[must_use]
	pub fn inner_mut(&mut self) -> &mut D {
		&mut self.inner
	}

	#[must_use]
	pub fn into_inner(self) -> D {
		self.inner
	}

	#[must_use]
	pub(super) fn _as_ptr(&mut self) -> *mut D {
		std::ptr::addr_of_mut!(self.inner)
	}
}

/// Adding some dynamic polymorphism to [`Store`] provides a few conveniences.
pub(super) trait DatumStore: 'static + Any + Send + Sync + std::fmt::Debug {
	#[must_use]
	fn id(&self) -> &str;

	#[must_use]
	fn datum_typeid(&self) -> TypeId;
}

impl<D: Datum> DatumStore for Store<D> {
	fn id(&self) -> &str {
		&self.id
	}

	fn datum_typeid(&self) -> TypeId {
		TypeId::of::<D>()
	}
}

#[derive(Debug, Clone, Copy)]
pub struct DataRef<'cat, D: Datum> {
	pub(super) catalog: &'cat Catalog,
	pub(super) arc: &'cat Arc<dyn DatumStore>,
	pub(super) store: &'cat Store<D>,
}

impl<'cat, D: Datum> DataRef<'cat, D> {
	#[must_use]
	pub(super) fn new(catalog: &'cat Catalog, arc: &'cat Arc<dyn DatumStore>) -> Self {
		Self {
			catalog,
			arc,
			// SAFETY: `DatumStore` has `Any` as a supertrait; these types are
			// essentially equivalent. Rust's dynamic type framework is just obstinate.
			store: unsafe {
				std::mem::transmute::<_, &Arc<dyn Any>>(arc)
					.downcast_ref()
					.unwrap()
			},
		}
	}

	#[must_use]
	pub fn catalog(&self) -> &Catalog {
		self.catalog
	}

	#[must_use]
	pub fn handle(&self) -> Handle<D> {
		// SAFETY: `DatumStore` meets all destination constraints, and this ref
		// could only have been created using the correct type.
		unsafe {
			let ret: Arc<dyn 'static + Send + Sync + Any> =
				std::mem::transmute::<_, _>(self.arc.clone());
			Handle::from(ret.downcast::<Store<D>>().unwrap_unchecked())
		}
	}
}

impl<D: Datum> std::ops::Deref for DataRef<'_, D> {
	type Target = Store<D>;

	fn deref(&self) -> &Self::Target {
		self.store
	}
}

impl<D: Datum> PartialEq for DataRef<'_, D> {
	/// Check if these are two references to the same datum and the same catalog.
	fn eq(&self, other: &Self) -> bool {
		std::ptr::eq(self.arc, other.arc) && std::ptr::eq(self.catalog, other.catalog)
	}
}

impl<D: Datum> Eq for DataRef<'_, D> {}

// Handle and InHandle /////////////////////////////////////////////////////////

/// Thin wrapper around an [`Arc`] pointing to a [`Datum`].
///
/// Attaching a generic type allows the pointer to be pre-downcast, so
/// dereferencing is as fast as with any other pointer with no unsafe code required.
#[derive(Debug)]
pub struct Handle<D: Datum>(Arc<Store<D>>);

impl<D: Datum> Handle<D> {
	#[must_use]
	pub fn id(&self) -> &str {
		self.0.id()
	}

	/// For use in inter-datum relationships.
	#[must_use]
	pub fn downgrade(&self) -> InHandle<D> {
		InHandle(Arc::downgrade(&self.0))
	}
}

impl<D: Datum> From<Arc<Store<D>>> for Handle<D> {
	fn from(value: Arc<Store<D>>) -> Self {
		Self(value)
	}
}

impl<D: Datum> From<&Arc<Store<D>>> for Handle<D> {
	fn from(value: &Arc<Store<D>>) -> Self {
		Self(value.clone())
	}
}

impl<D: Datum> std::ops::Deref for Handle<D> {
	type Target = D;

	#[inline]
	fn deref(&self) -> &Self::Target {
		self.0.inner()
	}
}

impl<D: Datum> Clone for Handle<D> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<D: Datum> PartialEq for Handle<D> {
	/// Check that these are two pointers to the same [`Datum`].
	fn eq(&self, other: &Self) -> bool {
		Arc::ptr_eq(&self.0, &other.0)
	}
}

impl<D: Datum> Eq for Handle<D> {}

/// Like [`Handle`] but [`Weak`], allowing inter-datum relationships (without
/// preventing in-place removal) in a way that can't leak.
#[derive(Debug)]
pub struct InHandle<D: Datum>(Weak<Store<D>>);

impl<D: Datum> InHandle<D> {
	#[must_use]
	pub fn upgrade(&self) -> Option<Handle<D>> {
		self.0.upgrade().map(Handle::from)
	}
}

impl<D: Datum> Clone for InHandle<D> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<D: Datum> PartialEq for InHandle<D> {
	/// Check that these are two pointers to the same [`Datum`].
	fn eq(&self, other: &Self) -> bool {
		Weak::ptr_eq(&self.0, &other.0)
	}
}

impl<D: Datum> Eq for InHandle<D> {}

macro_rules! impl_datum {
	($($datum_t:ty, $tname:literal);+) => {
		$(
			impl Datum for $datum_t {}
		)+

		pub(super) static DATUM_TYPE_NAMES: Lazy<HashMap<TypeId, &'static str>> = Lazy::new(|| {
			HashMap::from([
				$(
					(TypeId::of::<$datum_t>(), $tname),
				)+
			])
		});
	};
}

impl_datum! {
	Audio, "Audio";
	Blueprint, "Blueprint";
	DamageType, "Damage Type";
	Image, "Image";
	Level, "Level";
	LockDef, "Lock";
	PolyModel, "Poly Model";
	Species, "Species";
	VoxelModel, "Voxel Model"
}
