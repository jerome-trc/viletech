//! A "symbol" is anything about VZScript that can be introspected, such as a type.

use std::{
	alloc::Layout,
	any::TypeId,
	borrow::Borrow,
	hash::{Hash, Hasher},
	marker::PhantomData,
	sync::{Arc, Weak},
};

use fasthash::SeaHasher;

use super::{
	abi::Abi,
	tsys::{self, TypeInfo},
	Error, Function, TFunc,
};

/// A "symbol" is anything about VZScript that can be introspected, such as a type.
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

// Handle and InHandle /////////////////////////////////////////////////////////

/// "Reference-counted symbol" pointer.
///
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
	/// Check that these are two pointers to the same symbol in the same module.
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
/// relationships (without preventing in-place removal) in a way that can't leak.
#[derive(Debug)]
pub struct InHandle<S: Symbol>(Weak<S>, PhantomData<S>);

impl<S: Symbol> InHandle<S> {
	#[must_use]
	pub fn upgrade(&self) -> Handle<S> {
		Handle(
			Weak::upgrade(&self.0).expect("Failed to upgrade a symbol ARC."),
			PhantomData,
		)
	}
}

impl<S: Symbol> Clone for InHandle<S> {
	fn clone(&self) -> Self {
		Self(self.0.clone(), PhantomData)
	}
}

impl<S: Symbol> From<&Arc<S>> for InHandle<S> {
	fn from(value: &Arc<S>) -> Self {
		Self(Arc::downgrade(value), PhantomData)
	}
}

impl<S: Symbol> PartialEq for InHandle<S> {
	/// Check that these are two pointers to the same symbol in the same module.
	fn eq(&self, other: &Self) -> bool {
		Weak::ptr_eq(&self.0, &other.0)
	}
}

impl<S: Symbol> Eq for InHandle<S> {}

// SAFETY: See safety disclaimer for `Module`.
unsafe impl<S: Symbol> Send for InHandle<S> {}
unsafe impl<S: Symbol> Sync for InHandle<S> {}

/// A specialized alternative to [`Handle`] that can point to any kind of [`TypeInfo`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeHandle {
	Void(Handle<TypeInfo<()>>),
	Numeric(Handle<TypeInfo<tsys::Numeric>>),
	Array(Handle<TypeInfo<tsys::Array>>),
}

impl TypeHandle {
	#[must_use]
	pub fn header(&self) -> &SymbolHeader {
		match &self {
			Self::Void(tinfo) => tinfo.0.header(),
			Self::Numeric(tinfo) => tinfo.0.header(),
			Self::Array(tinfo) => tinfo.0.header(),
		}
	}

	#[must_use]
	pub fn layout(&self) -> Layout {
		match &self {
			Self::Void(tinfo) => tinfo.0.layout(),
			Self::Numeric(tinfo) => tinfo.0.layout(),
			Self::Array(tinfo) => tinfo.0.layout(),
		}
	}
}

/// A specialized alternative to [`InHandle`] that can point to any kind of [`TypeInfo`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeInHandle {
	_Void(InHandle<TypeInfo<()>>),
	_Numeric(InHandle<TypeInfo<tsys::Numeric>>),
	_Array(InHandle<TypeInfo<tsys::Array>>),
}

impl TypeInHandle {
	#[must_use]
	pub fn upgrade(&self) -> TypeHandle {
		match &self {
			Self::_Void(tinfo) => TypeHandle::Void(tinfo.upgrade()),
			Self::_Numeric(tinfo) => TypeHandle::Numeric(tinfo.upgrade()),
			Self::_Array(tinfo) => TypeHandle::Array(tinfo.upgrade()),
		}
	}

	#[must_use]
	pub fn layout(&self) -> Layout {
		match &self {
			Self::_Void(tinfo) => tinfo.0.upgrade().unwrap().layout(),
			Self::_Numeric(tinfo) => tinfo.0.upgrade().unwrap().layout(),
			Self::_Array(tinfo) => tinfo.0.upgrade().unwrap().layout(),
		}
	}
}

macro_rules! type_handle_converters {
	($($subtype:ty, $variant:ident);+) => {
		$(
			impl From<&Arc<TypeInfo<$subtype>>> for TypeHandle {
				fn from(value: &Arc<TypeInfo<$subtype>>) -> Self {
					Self::$variant(Handle(value.clone(), PhantomData))
				}
			}
		)+
	};
}

type_handle_converters! {
	(), Void;
	tsys::Numeric, Numeric;
	tsys::Array, Array
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
