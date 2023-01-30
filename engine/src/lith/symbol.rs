use std::{
	any::TypeId,
	hash::{Hash, Hasher},
};

use fasthash::SeaHasher;

pub trait Symbol: private::Sealed {}

/// Thin wrapper around a hash generated from a symbol's fully-qualified name
/// and the type ID of its corresponding Rust structure. Only exists for use as
/// a key in a module's symbol map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SymbolKey(u64);

impl SymbolKey {
	#[must_use]
	pub(super) fn new<S: Symbol>(name: &str) -> Self {
		let mut hasher = SeaHasher::default();
		name.hash(&mut hasher);
		TypeId::of::<S>().hash(&mut hasher);
		Self(hasher.finish())
	}
}

mod private {
	use std::any::Any;

	pub trait Sealed: Any + Send + Sync + std::fmt::Debug {
		/// Boilerplate allowing upcasting from `Symbol` to `Any`.
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
