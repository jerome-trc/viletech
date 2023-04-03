//! "Asset" is the catch-all term for any data unit the catalog can store.

mod actor;
mod audio;
mod level;
mod visual;

use std::{
	marker::{PhantomData, PhantomPinned},
	mem::ManuallyDrop,
	sync::{Arc, Weak},
};

use super::AssetError;

pub use self::{actor::*, audio::*, level::*, visual::*};

/// A dynamically-typed storage for a single asset.
pub struct Record {
	/// Note to reader: leave this here, even if not doing any pinning.
	#[allow(unused)]
	pin: PhantomPinned,
	id: String,
	kind: AssetKind,
	pub(self) asset: AssetUnion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetKind {
	Audio,
	Blueprint,
	DamageType,
	Image,
	Level,
	Palettes,
	PolyModel,
	Species,
	VoxelModel,
}

union AssetUnion {
	#[allow(dead_code)]
	placeholder: (),

	audio: ManuallyDrop<Audio>,
	blueprint: ManuallyDrop<Blueprint>,
	damage_type: ManuallyDrop<DamageType>,
	image: ManuallyDrop<Image>,
	level: ManuallyDrop<Level>,
	palettes: ManuallyDrop<PaletteSet>,
	poly_model: ManuallyDrop<PolyModel>,
	species: ManuallyDrop<Species>,
	voxel_model: ManuallyDrop<VoxelModel>,
}

impl Record {
	#[must_use]
	pub(super) fn new<A: Asset>(id: String, asset: A) -> Self {
		let mut ret = Self {
			pin: PhantomPinned,
			id,
			kind: A::KIND,
			asset: AssetUnion { placeholder: () },
		};

		unsafe {
			let a = A::get_mut(&mut ret);
			let invalid = std::ptr::replace(a, asset);
			std::mem::forget(invalid);
		}

		ret
	}

	#[must_use]
	pub fn id(&self) -> &str {
		&self.id
	}

	#[must_use]
	pub fn kind(&self) -> AssetKind {
		self.kind
	}

	/// Check this record's storage type.
	#[must_use]
	pub fn is<A: Asset>(&self) -> bool {
		self.kind == A::KIND
	}

	/// Returns [`AssetError::TypeMismatch`] if the storage type isn't `A`.
	///
	/// [`AssetError::TypeMismatch`]: super::AssetError::TypeMismatch
	pub fn downcast<A: Asset>(&self) -> Result<&A, AssetError> {
		if self.is::<A>() {
			unsafe { Ok(A::get(self)) }
		} else {
			Err(AssetError::TypeMismatch {
				expected: self.kind,
				given: A::KIND,
			})
		}
	}

	/// Returns [`AssetError::TypeMismatch`] if the storage type isn't `A`.
	///
	/// [`AssetError::TypeMismatch`]: super::AssetError::TypeMismatch
	pub fn downcast_mut<A: Asset>(&mut self) -> Result<&mut A, AssetError> {
		if self.is::<A>() {
			unsafe { Ok(A::get_mut(self)) }
		} else {
			Err(AssetError::TypeMismatch {
				expected: self.kind,
				given: A::KIND,
			})
		}
	}

	/// Returns [`AssetError::TypeMismatch`] if the storage type isn't `A`.
	///
	/// [`AssetError::TypeMismatch`]: super::AssetError::TypeMismatch
	pub fn handle<A: Asset>(self: &Arc<Self>) -> Result<Handle<A>, AssetError> {
		if self.is::<A>() {
			Ok(Handle::from(self))
		} else {
			Err(AssetError::TypeMismatch {
				expected: self.kind,
				given: A::KIND,
			})
		}
	}
}

impl Drop for Record {
	fn drop(&mut self) {
		unsafe {
			match self.kind {
				AssetKind::Audio => ManuallyDrop::drop(&mut self.asset.audio),
				AssetKind::Blueprint => ManuallyDrop::drop(&mut self.asset.blueprint),
				AssetKind::DamageType => ManuallyDrop::drop(&mut self.asset.damage_type),
				AssetKind::Image => ManuallyDrop::drop(&mut self.asset.image),
				AssetKind::Level => ManuallyDrop::drop(&mut self.asset.level),
				AssetKind::Palettes => ManuallyDrop::drop(&mut self.asset.palettes),
				AssetKind::PolyModel => ManuallyDrop::drop(&mut self.asset.poly_model),
				AssetKind::Species => ManuallyDrop::drop(&mut self.asset.species),
				AssetKind::VoxelModel => ManuallyDrop::drop(&mut self.asset.voxel_model),
			}
		}
	}
}

impl std::fmt::Debug for Record {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut debug = f.debug_struct("Record");

		debug
			.field("pin", &self.pin)
			.field("id", &self.id)
			.field("kind", &self.kind);

		unsafe {
			match self.kind {
				AssetKind::Audio => debug.field("audio", &self.asset.audio),
				AssetKind::Blueprint => debug.field("blueprint", &self.asset.blueprint),
				AssetKind::DamageType => debug.field("damage_type", &self.asset.damage_type),
				AssetKind::Image => debug.field("image", &self.asset.image),
				AssetKind::Level => debug.field("level", &self.asset.level),
				AssetKind::Palettes => debug.field("palette", &self.asset.palettes),
				AssetKind::PolyModel => debug.field("poly_model", &self.asset.poly_model),
				AssetKind::Species => debug.field("species", &self.asset.species),
				AssetKind::VoxelModel => debug.field("voxel_model", &self.asset.voxel_model),
			};
		}

		debug.finish()
	}
}

pub trait Asset {
	const KIND: AssetKind;

	/// An internal implementation detail.
	///
	/// # Safety
	///
	/// None. The library user should never call this.
	unsafe fn get(record: &Record) -> &Self;

	/// An internal implementation detail.
	///
	/// # Safety
	///
	/// None. The library user should never call this.
	unsafe fn get_mut(record: &mut Record) -> &mut Self;
}

// Handle //////////////////////////////////////////////////////////////////////

/// Thin wrapper around an [`Arc`] pointing to a [`Record`]. Attaching a generic
/// asset type allows the asset pointer to be safely downcast without any checks,
/// enabling safe, instant access to an asset's data from anywhere in the engine.
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Handle<A: Asset>(Arc<Record>, PhantomData<A>);

impl<A: Asset> Handle<A> {
	/// For use in inter-asset relationships.
	#[must_use]
	pub fn downgrade(&self) -> InHandle<A> {
		InHandle(Arc::downgrade(&self.0), PhantomData)
	}
}

impl<A: Asset> From<Arc<Record>> for Handle<A> {
	/// This conversion panics if the asset type of the given record is not `A`.
	fn from(value: Arc<Record>) -> Self {
		let expected = A::KIND;
		let kind = value.kind;

		assert_eq!(
			expected, kind,
			"Expected asset type: {expected:#?}, but got: {kind:#?}",
		);

		Self(value, PhantomData)
	}
}

impl<A: Asset> From<&Arc<Record>> for Handle<A> {
	/// This conversion panics if the asset type of the given record is not `A`.
	fn from(value: &Arc<Record>) -> Self {
		let expected = A::KIND;
		let kind = value.kind;

		assert_eq!(
			expected, kind,
			"Expected asset type: {expected:#?}, but got: {kind:#?}",
		);

		Self(value.clone(), PhantomData)
	}
}

impl<A: 'static + Asset> std::ops::Deref for Handle<A> {
	type Target = A;

	#[inline]
	fn deref(&self) -> &Self::Target {
		// SAFETY: Type correctness was validated during handle acquisition.
		debug_assert!(self.0.kind == A::KIND);
		unsafe { A::get(&self.0) }
	}
}

impl<A: Asset> PartialEq for Handle<A> {
	/// Check that these are two handles to the same [`Record`].
	fn eq(&self, other: &Self) -> bool {
		Arc::ptr_eq(&self.0, &other.0)
	}
}

impl<A: Asset> Eq for Handle<A> {}

/// Internal handle. Like [`Handle`] but [`Weak`], allowing inter-asset
/// relationships (without preventing in-place mutation or removal) in a way
/// that can't leak.
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct InHandle<A: Asset>(Weak<Record>, PhantomData<A>);

impl<A: Asset> InHandle<A> {
	#[must_use]
	pub fn upgrade(&self) -> Option<Handle<A>> {
		self.0.upgrade().map(Handle::from)
	}
}

impl<A: Asset> PartialEq for InHandle<A> {
	/// Check that these are two handles to the same [`Record`].
	fn eq(&self, other: &Self) -> bool {
		Weak::ptr_eq(&self.0, &other.0)
	}
}

impl<A: Asset> Eq for InHandle<A> {}
