//! "Symbols" are things like type and function definitions.
//!
//! These are tracked in rich detail by LithScript so that they can be easily
//! introspected by applications embedding the language.

use std::{
	alloc::Layout,
	any::{Any, TypeId},
	borrow::Borrow,
	hash::{Hash, Hasher},
	sync::{Arc, Weak},
};

use rustc_hash::FxHasher;

use crate::{
	project::Project,
	tsys::{self, TypeInfo},
};

pub trait Symbol: 'static + Any + Send + Sync + std::fmt::Debug {
	type HashInput<'i>: ?Sized + Hash;

	#[must_use]
	fn key(&self) -> SymbolKey;
}

#[derive(Debug)]
pub struct Store<S: Symbol> {
	name: String,
	inner: S,
}

impl<S: Symbol> Store<S> {
	#[must_use]
	pub fn new(name: String, symbol: S) -> Self {
		Self {
			name,
			inner: symbol,
		}
	}

	#[must_use]
	pub fn name(&self) -> &str {
		&self.name
	}

	#[must_use]
	pub fn inner(&self) -> &S {
		&self.inner
	}

	#[must_use]
	pub fn inner_mut(&mut self) -> &mut S {
		&mut self.inner
	}

	#[must_use]
	pub fn into_inner(self) -> S {
		self.inner
	}
}

#[derive(Debug, Clone, Copy)]
pub struct SymbolRef<'p, S: Symbol> {
	pub(super) project: &'p Project,
	pub(super) arc: &'p Arc<dyn SymbolStore>,
	pub(super) store: &'p Store<S>,
}

impl<'p, S: Symbol> SymbolRef<'p, S> {
	#[must_use]
	pub(super) fn _new(project: &'p Project, arc: &'p Arc<dyn SymbolStore>) -> Self {
		Self {
			project,
			arc,
			// SAFETY: `SymbolStore` has `Any` as a supertrait; these types are
			// essentially equivalent. Rust's dynamic type framework is just obstinate.
			store: unsafe {
				std::mem::transmute::<_, &Arc<dyn Any>>(arc)
					.downcast_ref()
					.unwrap()
			},
		}
	}

	#[must_use]
	pub fn project(&self) -> &Project {
		self.project
	}

	#[must_use]
	pub fn handle(&self) -> Handle<S> {
		// SAFETY: `SymbolStore` meets all destination constraints, and `self`
		// could only have been created using the correct type.
		unsafe {
			let ret: Arc<dyn 'static + Send + Sync + Any> =
				std::mem::transmute::<_, _>(self.arc.clone());

			Handle::from(ret.downcast::<Store<S>>().unwrap_unchecked())
		}
	}
}

impl<S: Symbol> std::ops::Deref for SymbolRef<'_, S> {
	type Target = Store<S>;

	fn deref(&self) -> &Self::Target {
		self.store
	}
}

// Handle and InHandle /////////////////////////////////////////////////////////

/// Thin wrapper around an [`Arc`] pointing to an [`Symbol`].
///
/// Attaching a generic type allows the pointer to be pre-downcast, so dereferencing
/// is as fast as with any other pointer with no unsafe code required.
#[derive(Debug)]
pub struct Handle<S: Symbol>(Arc<Store<S>>);

impl<S: 'static + Symbol> std::ops::Deref for Handle<S> {
	type Target = S;

	fn deref(&self) -> &Self::Target {
		&self.0.inner
	}
}

impl<S: Symbol> Handle<S> {
	#[must_use]
	pub fn name(&self) -> &str {
		self.0.name()
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
		Self(self.0.clone())
	}
}

impl<S: Symbol> From<&Arc<Store<S>>> for Handle<S> {
	fn from(value: &Arc<Store<S>>) -> Self {
		Handle(value.clone())
	}
}

impl<S: Symbol> From<Arc<Store<S>>> for Handle<S> {
	fn from(value: Arc<Store<S>>) -> Self {
		Handle(value)
	}
}

// SAFETY: See safety disclaimer for `Module`.
unsafe impl<S: Symbol> Send for Handle<S> {}
unsafe impl<S: Symbol> Sync for Handle<S> {}

/// Internal handle. Like [`Handle`] but [`Weak`], allowing inter-symbol
/// relationships (without preventing in-place removal) in a way that can't leak.
#[derive(Debug)]
pub struct InHandle<S: Symbol>(Weak<Store<S>>);

impl<S: Symbol> InHandle<S> {
	#[must_use]
	pub fn upgrade(&self) -> Handle<S> {
		Handle(Weak::upgrade(&self.0).expect("failed to upgrade a symbol ARC"))
	}
}

impl<S: Symbol> Clone for InHandle<S> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<S: Symbol> From<&Arc<Store<S>>> for InHandle<S> {
	fn from(value: &Arc<Store<S>>) -> Self {
		Self(Arc::downgrade(value))
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
	pub fn name(&self) -> &str {
		match &self {
			Self::Void(tinfo) => tinfo.0.name(),
			Self::Numeric(tinfo) => tinfo.0.name(),
			Self::Array(tinfo) => tinfo.0.name(),
		}
	}

	#[must_use]
	pub fn layout(&self) -> Layout {
		match &self {
			Self::Void(tinfo) => tinfo.0.inner.layout(),
			Self::Numeric(tinfo) => tinfo.0.inner.layout(),
			Self::Array(tinfo) => tinfo.0.inner.layout(),
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
			Self::_Void(tinfo) => tinfo.0.upgrade().unwrap().inner.layout(),
			Self::_Numeric(tinfo) => tinfo.0.upgrade().unwrap().inner.layout(),
			Self::_Array(tinfo) => tinfo.0.upgrade().unwrap().inner.layout(),
		}
	}
}

macro_rules! type_handle_converters {
	($($subtype:ty, $variant:ident);+) => {
		$(
			impl From<&Arc<Store<TypeInfo<$subtype>>>> for TypeHandle {
				fn from(value: &Arc<Store<TypeInfo<$subtype>>>) -> Self {
					Self::$variant(Handle(value.clone()))
				}
			}

			impl From<Handle<TypeInfo<$subtype>>> for TypeHandle {
				fn from(value: Handle<TypeInfo<$subtype>>) -> Self {
					Self::$variant(value)
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

// Details /////////////////////////////////////////////////////////////////////

/// An implementation detail, exposed for the benefit of [`Symbol`].
/// Field `1` is a hash of the symbol's name string.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SymbolKey(TypeId, u64);

impl SymbolKey {
	#[must_use]
	pub(super) fn new<'i, S: Symbol>(input: impl Borrow<S::HashInput<'i>>) -> Self {
		let mut hasher = FxHasher::default();
		input.borrow().hash(&mut hasher);
		Self(TypeId::of::<S>(), hasher.finish())
	}
}

/// Adding some dynamic polymorphism to [`Store`] provides a few conveniences.
pub(super) trait SymbolStore: 'static + Any + Send + Sync + std::fmt::Debug {
	#[must_use]
	fn name(&self) -> &str;

	#[must_use]
	fn symbol_typeid(&self) -> TypeId;
}

impl<S: Symbol> SymbolStore for Store<S> {
	fn name(&self) -> &str {
		&self.name
	}

	fn symbol_typeid(&self) -> TypeId {
		TypeId::of::<S>()
	}
}
