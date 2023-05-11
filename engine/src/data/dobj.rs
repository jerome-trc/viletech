//! "Data objects". Things like levels, actor blueprints, and sounds.

mod actor;
mod audio;
mod level;
mod visual;

use std::sync::{Arc, Weak};

pub use self::{actor::*, audio::*, level::*, visual::*};

/// A storage implementation detail, exposed only so library users can
/// access common datum metadata.
#[derive(Debug)]
pub struct DatumHeader {
	pub id: String,
}

impl DatumHeader {
	#[must_use]
	pub fn nickname(&self) -> &str {
		self.id.split('/').last().unwrap()
	}
}

pub trait Datum: private::Sealed {
	#[must_use]
	fn header(&self) -> &DatumHeader;
	#[must_use]
	fn header_mut(&mut self) -> &mut DatumHeader;
	#[must_use]
	fn type_name(&self) -> &'static str;
}

// Handle and InHandle /////////////////////////////////////////////////////////

/// Thin wrapper around an [`Arc`] pointing to a [`Datum`].
///
/// Attaching a generic type allows the pointer to be pre-downcast, so
/// dereferencing is as fast as with any other pointer with no unsafe code required.
#[derive(Debug)]
pub struct Handle<D: Datum>(Arc<D>);

impl<D: Datum> Handle<D> {
	/// For use in inter-datum relationships.
	#[must_use]
	pub fn downgrade(&self) -> InHandle<D> {
		InHandle(Arc::downgrade(&self.0))
	}
}

impl<D: Datum> From<Arc<D>> for Handle<D> {
	fn from(value: Arc<D>) -> Self {
		Self(value)
	}
}

impl<D: Datum> From<&Arc<D>> for Handle<D> {
	fn from(value: &Arc<D>) -> Self {
		Self(value.clone())
	}
}

impl<D: Datum> std::ops::Deref for Handle<D> {
	type Target = D;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.0
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
pub struct InHandle<D: Datum>(Weak<D>);

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
	($($t:ty, $tname:literal);+) => {
		$(
			impl Datum for $t {
				fn header(&self) -> &DatumHeader {
					&self.header
				}

				fn header_mut(&mut self) -> &mut DatumHeader {
					&mut self.header
				}

				fn type_name(&self) -> &'static str {
					$tname
				}
			}
		)+
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

mod private {
	use std::any::Any;

	use super::*;

	pub trait Sealed: 'static + Any + Send + Sync + std::fmt::Debug {
		/// Boilerplate allowing upcasting from [`super::Datum`] to [`Any`].
		#[must_use]
		fn as_any(&self) -> &dyn Any;
	}

	macro_rules! impl_sealed {
		($($type:ty),+) => {
			$(
				impl Sealed for $type {
					fn as_any(&self) -> &dyn Any {
						self
					}
				}
			)+
		};
	}

	impl_sealed! {
		Audio,
		Blueprint,
		DamageType,
		Image,
		Level,
		LockDef,
		PolyModel,
		Species,
		VoxelModel
	}
}
