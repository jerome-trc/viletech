//! Runtime information storage and handle types.

use std::{
	any::{Any, TypeId},
	mem::ManuallyDrop,
	sync::{Arc, Weak},
};

use cranelift_module::{DataId, FuncId};
use util::rstring::RString;

use crate::{compile::JitModule, tsys::TypeDef};

pub trait RtInfo: 'static + Any + Send + Sync + std::fmt::Debug {}

#[derive(Debug)]
pub struct Store<R: RtInfo> {
	name: RString,
	inner: R,
}

impl<R: RtInfo> Store<R> {
	#[must_use]
	pub fn new(name: RString, symbol: R) -> Self {
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
	pub fn inner(&self) -> &R {
		&self.inner
	}

	#[must_use]
	pub fn inner_mut(&mut self) -> &mut R {
		&mut self.inner
	}

	#[must_use]
	pub fn into_inner(self) -> R {
		self.inner
	}
}

#[derive(Debug)]
pub struct Function {
	_ptr: *const u8,
	id: FuncId,
	#[allow(unused)]
	module: Arc<JitModule>,
}

impl Function {
	#[must_use]
	pub fn id(&self) -> FuncId {
		self.id
	}
}

// SAFETY: `Function::ptr` is to a JIT function, kept alive by `Function::module`.
unsafe impl Send for Function {}
unsafe impl Sync for Function {}

impl RtInfo for Function {}

#[derive(Debug)]
pub struct Data {
	ptr: *const u8,
	size: usize,
	id: DataId,
	#[allow(unused)]
	module: Arc<JitModule>,
	native_t: Option<TypeId>,
}

impl Data {
	#[must_use]
	pub fn id(&self) -> DataId {
		self.id
	}

	#[must_use]
	pub fn size(&self) -> usize {
		self.size
	}

	#[must_use]
	pub fn as_native_ref<T: 'static>(&self) -> Option<&T> {
		if self
			.native_t
			.is_some_and(|typeid| typeid == TypeId::of::<T>())
		{
			Some(unsafe { self.ptr.cast::<T>().as_ref().unwrap() })
		} else {
			None
		}
	}

	pub fn as_native_mut<T: 'static>(&mut self) -> Option<&mut T> {
		if self
			.native_t
			.is_some_and(|typeid| typeid == TypeId::of::<T>())
		{
			Some(unsafe { self.ptr.cast::<T>().cast_mut().as_mut().unwrap() })
		} else {
			None
		}
	}
}

// SAFETY: `Data::ptr` is to a JIT data object, kept alive by `Data::module`.
unsafe impl Send for Data {}
unsafe impl Sync for Data {}

impl RtInfo for Data {}

/// Thin wrapper around an [`Arc`].
///
/// Attaching a generic type allows the pointer to be pre-downcast, so dereferencing
/// is as fast as with any other pointer with no unsafe code required.
#[derive(Debug)]
pub struct Handle<R: RtInfo>(Arc<Store<R>>);

impl<R: 'static + RtInfo> std::ops::Deref for Handle<R> {
	type Target = R;

	fn deref(&self) -> &Self::Target {
		&self.0.inner
	}
}

impl<R: RtInfo> Handle<R> {
	#[must_use]
	pub fn name(&self) -> &str {
		self.0.name()
	}
}

impl<R: RtInfo> PartialEq for Handle<R> {
	/// Check that these are two pointers to the same symbol in the same module.
	fn eq(&self, other: &Self) -> bool {
		Arc::ptr_eq(&self.0, &other.0)
	}
}

impl<R: RtInfo> Eq for Handle<R> {}

impl<R: RtInfo> Clone for Handle<R> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<R: RtInfo> From<&Arc<Store<R>>> for Handle<R> {
	fn from(value: &Arc<Store<R>>) -> Self {
		Handle(value.clone())
	}
}

impl<R: RtInfo> From<Arc<Store<R>>> for Handle<R> {
	fn from(value: Arc<Store<R>>) -> Self {
		Handle(value)
	}
}

/// Internal handle. Like [`Handle`] but [`Weak`], allowing inter-symbol
/// relationships (without preventing in-place removal) in a way that can't leak.
#[derive(Debug)]
pub struct InHandle<S: RtInfo>(Weak<Store<S>>);

impl<R: RtInfo> InHandle<R> {
	#[must_use]
	pub fn upgrade(&self) -> Handle<R> {
		Handle(Weak::upgrade(&self.0).expect("failed to upgrade a symbol ARC"))
	}
}

impl<R: RtInfo> Clone for InHandle<R> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<R: RtInfo> From<&Arc<Store<R>>> for InHandle<R> {
	fn from(value: &Arc<Store<R>>) -> Self {
		Self(Arc::downgrade(value))
	}
}

impl<R: RtInfo> PartialEq for InHandle<R> {
	/// Check that these are two pointers to the same symbol in the same module.
	fn eq(&self, other: &Self) -> bool {
		Weak::ptr_eq(&self.0, &other.0)
	}
}

impl<R: RtInfo> Eq for InHandle<R> {}

pub(crate) struct Record {
	pub(crate) tag: StoreTag,
	pub(crate) data: StoreUnion,
}

/// Gets discriminated with [`StoreTag`].
pub(crate) union StoreUnion {
	func: ManuallyDrop<Arc<Store<Function>>>,
	data: ManuallyDrop<Arc<Store<Data>>>,
	typedef: ManuallyDrop<Arc<Store<TypeDef>>>,
}

/// Separated discriminant for [`StoreUnion`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum StoreTag {
	Function,
	Data,
	Type,
}

impl std::fmt::Debug for Record {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("S")
			.field("tag", &self.tag)
			.field("data", unsafe {
				match self.tag {
					StoreTag::Function => &self.data.func,
					StoreTag::Data => &self.data.data,
					StoreTag::Type => &self.data.typedef,
				}
			})
			.finish()
	}
}

impl Drop for Record {
	fn drop(&mut self) {
		unsafe {
			match self.tag {
				StoreTag::Function => ManuallyDrop::drop(&mut self.data.func),
				StoreTag::Data => ManuallyDrop::drop(&mut self.data.data),
				StoreTag::Type => ManuallyDrop::drop(&mut self.data.typedef),
			}
		}
	}
}
