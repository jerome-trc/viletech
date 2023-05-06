//! "Asset" is the catch-all term for any data unit the catalog can store.

mod actor;
mod audio;
mod level;
mod visual;

use std::{
	marker::PhantomData,
	sync::{Arc, Weak},
};

pub use self::{actor::*, audio::*, level::*, visual::*};

/// A storage implementation detail, exposed only so library users can
/// access common asset metadata.
#[derive(Debug)]
pub struct AssetHeader {
	pub(super) id: String,
}

impl AssetHeader {
	#[must_use]
	pub fn nickname(&self) -> &str {
		self.id.split('/').last().unwrap()
	}
}

pub trait Asset: private::Sealed {
	#[must_use]
	fn header(&self) -> &AssetHeader;
	#[must_use]
	fn header_mut(&mut self) -> &mut AssetHeader;
	#[must_use]
	fn type_name(&self) -> &'static str;
}

// Handle //////////////////////////////////////////////////////////////////////

/// Thin wrapper around an [`Arc`] pointing to an [`Asset`]. Attaching a generic
/// type allows the pointer to be pre-downcast, so dereferencing is as fast as
/// with any other pointer with no unsafe code required.
#[derive(Debug)]
pub struct Handle<A: Asset>(Arc<A>, PhantomData<A>);

impl<A: Asset> Handle<A> {
	/// For use in inter-asset relationships.
	#[must_use]
	pub fn downgrade(&self) -> InHandle<A> {
		InHandle(Arc::downgrade(&self.0), PhantomData)
	}
}

impl<A: Asset> From<Arc<A>> for Handle<A> {
	fn from(value: Arc<A>) -> Self {
		Self(value, PhantomData)
	}
}

impl<A: Asset> From<&Arc<A>> for Handle<A> {
	fn from(value: &Arc<A>) -> Self {
		Self(value.clone(), PhantomData)
	}
}

impl<A: Asset> std::ops::Deref for Handle<A> {
	type Target = A;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<A: Asset> Clone for Handle<A> {
	fn clone(&self) -> Self {
		Self(self.0.clone(), PhantomData)
	}
}

impl<A: Asset> PartialEq for Handle<A> {
	/// Check that these are two handles to the same [`Asset`].
	fn eq(&self, other: &Self) -> bool {
		Arc::ptr_eq(&self.0, &other.0)
	}
}

impl<A: Asset> Eq for Handle<A> {}

/// Internal handle. Like [`Handle`] but [`Weak`], allowing inter-asset
/// relationships (without preventing in-place removal) in a way that can't leak.
#[derive(Debug)]
pub struct InHandle<A: Asset>(Weak<A>, PhantomData<A>);

impl<A: Asset> InHandle<A> {
	#[must_use]
	pub fn upgrade(&self) -> Option<Handle<A>> {
		self.0.upgrade().map(Handle::from)
	}
}

impl<A: Asset> Clone for InHandle<A> {
	fn clone(&self) -> Self {
		Self(self.0.clone(), PhantomData)
	}
}

impl<A: Asset> PartialEq for InHandle<A> {
	/// Check that these are two handles to the same [`Asset`].
	fn eq(&self, other: &Self) -> bool {
		Weak::ptr_eq(&self.0, &other.0)
	}
}

impl<A: Asset> Eq for InHandle<A> {}

macro_rules! impl_asset {
	($($t:ty, $tname:literal);+) => {
		$(
			impl Asset for $t {
				fn header(&self) -> &AssetHeader {
					&self.header
				}

				fn header_mut(&mut self) -> &mut AssetHeader {
					&mut self.header
				}

				fn type_name(&self) -> &'static str {
					$tname
				}
			}
		)+
	};
}

impl_asset! {
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
		/// Boilerplate allowing upcasting from [`super::Asset`] to [`Any`].
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
