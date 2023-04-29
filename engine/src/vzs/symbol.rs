use std::{
	any::TypeId,
	hash::{Hash, Hasher},
};

use fasthash::SeaHasher;

pub trait Symbol: private::Sealed {
	#[must_use]
	fn header(&self) -> &SymbolHeader;
	#[must_use]
	fn header_mut(&mut self) -> &mut SymbolHeader;
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
pub(super) struct SymbolKey(u64);

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

	use crate::vzs::{abi::Abi, *};

	pub trait Sealed: Any + Send + Sync + std::fmt::Debug {
		/// Boilerplate allowing upcasting from [`super::Symbol`] to [`Any`].
		#[must_use]
		fn as_any(&self) -> &dyn Any;
	}

	impl Sealed for Function {
		fn as_any(&self) -> &dyn Any {
			self
		}
	}

	impl<A: Abi, R: Abi> Sealed for TFunc<A, R> {
		fn as_any(&self) -> &dyn Any {
			self
		}
	}

	impl Sealed for TypeInfo {
		fn as_any(&self) -> &dyn Any {
			self
		}
	}
}
