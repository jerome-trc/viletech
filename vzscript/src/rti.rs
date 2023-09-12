//! Runtime information storage and handle types.

use std::{
	any::{Any, TypeId},
	ffi::c_void,
	hash::{Hash, Hasher},
	marker::PhantomData,
	mem::ManuallyDrop,
	sync::{Arc, Weak},
};

use cranelift_module::{DataId, FuncId};
use rustc_hash::FxHasher;

use crate::{
	back::{AbiTypes, JitModule},
	native::Native,
	project::Project,
	tsys::{
		ArrayType, ClassType, FuncType, PrimitiveType, StructType, TypeDef, TypeHandle, TypeRef,
		UnionType,
	},
	zname::ZName,
};

pub trait RtInfo: 'static + Any + Send + Sync + std::fmt::Debug {}

#[derive(Debug)]
pub struct Store<R: RtInfo> {
	name: ZName,
	inner: R,
}

impl<R: RtInfo> Store<R> {
	#[must_use]
	pub(crate) fn new(name: ZName, symbol: R) -> Self {
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
	ptr: *const c_void,
	id: FuncId,
	sighash: SignatureHash,
	#[allow(unused)]
	module: Arc<JitModule>,
}

impl Function {
	#[must_use]
	pub fn downcast<A, R>(&self) -> Option<TFn<A, R>>
	where
		A: Native,
		R: Native,
	{
		let h = SignatureHash::new(A::repr(), R::repr());
		(h == self.sighash).then_some(TFn(self, PhantomData))
	}
}

/// A strongly-typed pointer to a compiled function.
///
/// See [`Function::downcast`].
pub struct TFn<'f, A, R>(&'f Function, PhantomData<fn(A) -> R>)
where
	A: Native,
	R: Native;

impl<A, R> TFn<'_, A, R>
where
	A: Native,
	R: Native,
{
	pub fn call(&self, args: A) -> R {
		unsafe { std::mem::transmute::<_, fn(A) -> R>(self.0.ptr)(args) }
	}
}

/// A strongly-typed [`Handle`] over a [`Function`].
///
/// Must be acquired using `Handle::<Function>::downcast`.
pub struct TFnHandle<A, R>(Handle<Function>, PhantomData<fn(A) -> R>)
where
	A: Native,
	R: Native;

impl<A, R> TFnHandle<A, R>
where
	A: Native,
	R: Native,
{
	pub fn call(&self, args: A) -> R {
		unsafe { std::mem::transmute::<_, fn(A) -> R>(self.0.ptr)(args) }
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SignatureHash(u64);

impl SignatureHash {
	#[must_use]
	pub(crate) fn new(params: AbiTypes, rets: AbiTypes) -> Self {
		let mut fxh = FxHasher::default();
		params.hash(&mut fxh);
		rets.hash(&mut fxh);
		Self(fxh.finish())
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
	native_t: Option<TypeId>,
	immutable: bool,
	#[allow(unused)]
	module: Arc<JitModule>,
}

impl Data {
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

	/// Returns `None` if `T` is not the type of this data object,
	/// or if the data object was declared to the backend as being immutable.
	pub fn as_native_mut<T: 'static>(&mut self) -> Option<&mut T> {
		if self.immutable {
			return None;
		}

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
	/// Check that these are two pointers to the same RTI object in the same module.
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

impl Handle<Function> {
	#[must_use]
	pub fn downcast<A, R>(&self) -> Option<TFnHandle<A, R>>
	where
		A: Native,
		R: Native,
	{
		self.0
			.inner
			.downcast::<A, R>()
			.map(|_| TFnHandle(self.clone(), PhantomData))
	}
}

impl Handle<TypeDef> {
	#[must_use]
	pub fn downcast<T: 'static>(self) -> Option<TypeHandle<T>> {
		match self.inner() {
			TypeRef::Array(_) => (TypeId::of::<T>() == TypeId::of::<ArrayType>())
				.then(|| TypeHandle(self, PhantomData)),
			TypeRef::Class(_) => (TypeId::of::<T>() == TypeId::of::<ClassType>())
				.then(|| TypeHandle(self, PhantomData)),
			TypeRef::Function(_) => (TypeId::of::<T>() == TypeId::of::<FuncType>())
				.then(|| TypeHandle(self, PhantomData)),
			TypeRef::Primitive(_) => (TypeId::of::<T>() == TypeId::of::<PrimitiveType>())
				.then(|| TypeHandle(self, PhantomData)),
			TypeRef::Struct(_) => (TypeId::of::<T>() == TypeId::of::<StructType>())
				.then(|| TypeHandle(self, PhantomData)),
			TypeRef::Union(_) => (TypeId::of::<T>() == TypeId::of::<UnionType>())
				.then(|| TypeHandle(self, PhantomData)),
		}
	}
}

/// Internal handle. Like [`Handle`] but [`Weak`], allowing inter-RTI
/// relationships (without preventing in-place removal) in a way that can't leak.
#[derive(Debug)]
pub struct InHandle<S: RtInfo>(Weak<Store<S>>);

impl<R: RtInfo> InHandle<R> {
	#[must_use]
	pub fn upgrade(&self) -> Handle<R> {
		Handle(Weak::upgrade(&self.0).expect("failed to upgrade an RTI ARC"))
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
	pub(crate) inner: StoreUnion,
}

impl Record {
	#[must_use]
	pub(crate) fn new_data(store: Store<Data>) -> Self {
		Self {
			tag: StoreTag::Data,
			inner: StoreUnion {
				data: ManuallyDrop::new(Arc::new(store)),
			},
		}
	}

	#[must_use]
	pub(crate) fn new_func(store: Store<Function>) -> Self {
		Self {
			tag: StoreTag::Function,
			inner: StoreUnion {
				func: ManuallyDrop::new(Arc::new(store)),
			},
		}
	}

	#[must_use]
	pub(crate) fn new_type(store: Store<TypeDef>) -> Self {
		Self {
			tag: StoreTag::Type,
			inner: StoreUnion {
				typedef: ManuallyDrop::new(Arc::new(store)),
			},
		}
	}

	#[must_use]
	pub(crate) fn handle_data(&self) -> Handle<Data> {
		assert_eq!(self.tag, StoreTag::Data);
		Handle(Arc::clone(unsafe { &self.inner.data }))
	}

	#[must_use]
	pub(crate) fn handle_func(&self) -> Handle<Function> {
		assert_eq!(self.tag, StoreTag::Function);
		Handle(Arc::clone(unsafe { &self.inner.func }))
	}

	#[must_use]
	pub(crate) fn handle_type(&self) -> Handle<TypeDef> {
		assert_eq!(self.tag, StoreTag::Type);
		Handle(Arc::clone(unsafe { &self.inner.typedef }))
	}
}

/// Gets discriminated with [`StoreTag`].
pub(crate) union StoreUnion {
	pub(crate) func: ManuallyDrop<Arc<Store<Function>>>,
	pub(crate) data: ManuallyDrop<Arc<Store<Data>>>,
	pub(crate) typedef: ManuallyDrop<Arc<Store<TypeDef>>>,
}

/// Separated discriminant for [`StoreUnion`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StoreTag {
	Function,
	Data,
	Type,
}

impl std::fmt::Debug for Record {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Record")
			.field("tag", &self.tag)
			.field("data", unsafe {
				match self.tag {
					StoreTag::Function => &self.inner.func,
					StoreTag::Data => &self.inner.data,
					StoreTag::Type => &self.inner.typedef,
				}
			})
			.finish()
	}
}

impl Drop for Record {
	fn drop(&mut self) {
		unsafe {
			match self.tag {
				StoreTag::Function => ManuallyDrop::drop(&mut self.inner.func),
				StoreTag::Data => ManuallyDrop::drop(&mut self.inner.data),
				StoreTag::Type => ManuallyDrop::drop(&mut self.inner.typedef),
			}
		}
	}
}

/// The primary interface for reading runtime info. Get one using [`Project::get`].
///
/// Can be turned into a longer-lasting [`Handle`] (for the cost of one atomic
/// reference count increment) using [`Self::handle`].
#[derive(Debug)]
pub struct Ref<'p, R: RtInfo> {
	project: &'p Project,
	store: &'p Arc<Store<R>>,
}

impl<'p, R: RtInfo> Ref<'p, R> {
	#[must_use]
	pub fn project(&self) -> &Project {
		self.project
	}

	#[must_use]
	pub fn handle(&self) -> Handle<R> {
		self.store.into()
	}

	#[must_use]
	pub(crate) fn new(project: &'p Project, store: &'p Arc<Store<R>>) -> Self {
		Self { project, store }
	}
}

impl<R: RtInfo> std::ops::Deref for Ref<'_, R> {
	type Target = Store<R>;

	fn deref(&self) -> &Self::Target {
		self.store.deref()
	}
}
