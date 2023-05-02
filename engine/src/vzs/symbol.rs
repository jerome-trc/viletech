use std::{
	any::TypeId,
	borrow::Borrow,
	hash::{Hash, Hasher},
};

use fasthash::SeaHasher;

pub trait Symbol: private::Sealed {
	#[must_use]
	fn header(&self) -> &SymbolHeader;
	#[must_use]
	fn header_mut(&mut self) -> &mut SymbolHeader;
	#[must_use]
	fn key(&self) -> SymbolKey;
}

/// Detail trait extending [`Symbol`] with necessary non-object-safe information.
pub trait SymbolHash<'i>: Symbol {
	type HashInput: ?Sized + Hash;
}

/// A storage implementation detail, exposed only so library users can
/// access common symbol metadata.
#[derive(Debug)]
pub struct SymbolHeader {
	pub name: String,
}

/// Thin wrapper around a hash generated from a symbol's fully-qualified name
/// and the type ID of its corresponding Rust structure (in that order).
/// Only exists for use as a key in the [`Project`] symbol map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolKey(u64);

impl SymbolKey {
	#[must_use]
	pub(super) fn new<'i, S: SymbolHash<'i>>(input: impl Borrow<S::HashInput>) -> Self {
		let mut hasher = SeaHasher::default();
		TypeId::of::<S>().hash(&mut hasher);
		input.borrow().hash(&mut hasher);
		Self(hasher.finish())
	}
}

pub(crate) mod private {
	use std::any::Any;

	use crate::vzs::{abi::Abi, tsys::TypeInfo, *};

	pub trait Sealed: Any + Send + Sync + std::fmt::Debug {
		/// Boilerplate allowing upcasting from [`super::Symbol`] to [`Any`].
		#[must_use]
		fn as_any(&self) -> &dyn Any;
	}

	impl<A: Abi, R: Abi> Sealed for TFunc<A, R> {
		fn as_any(&self) -> &dyn Any {
			self
		}
	}

	macro_rules! impl_sealed {
		($($type:ty), +) => {
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
		Function,
		TypeInfo<()>,
		TypeInfo<tsys::Numeric>,
		TypeInfo<tsys::Array>
	}
}
