//! A VZScript module is a single linkage unit.
//!
//! This is an equivalent concept to modules in LLVM, Rust, and Cranelift, but it
//! inherits the ZScript behavior of being compiled from an arbitrary number of
//! source files, rather than just one. Hence, `vzs` is a module (for language
//! support), `vtec` is a module for native engine functionality, et cetera.
//!
//! To get started, [create a `Builder`].
//!
//! [create a `Builder`]: Builder::new

use std::{
	any::TypeId,
	hash::{Hash, Hasher},
	marker::PhantomData,
	mem::MaybeUninit,
	sync::{Arc, Weak},
};

use cranelift_jit::{JITBuilder, JITModule};
use dashmap::DashMap;
use fasthash::SeaHasher;

use super::{abi::Abi, func::TFunc, tsys, Error, Function, TypeInfo};

/// A collection of modules and a table for accessing their symbols.
#[derive(Debug)]
pub struct Project {
	modules: Vec<Module>,
	/// In each value, `0` corresponds to an element in `Self::modules` and `1`
	/// correpsonds to an element in `Module::symbols`.
	symbols: DashMap<SymbolKey, (usize, usize)>,
}

impl Project {
	pub fn get<S: Symbol>(&self, name: &str) -> Option<&Arc<S>> {
		let key = SymbolKey::new::<S>(name);

		if let Some(kvp) = self.symbols.get(&key) {
			self.modules[kvp.0].symbols[kvp.1].as_any().downcast_ref()
		} else {
			None
		}
	}

	pub fn add_module(&mut self, module: Module) {
		for (i, symbol) in module.symbols.iter().enumerate() {
			let key = SymbolKey::new::<TypeInfo>(&symbol.header().name);
			self.symbols.insert(key, (0, i));
		}

		self.modules.push(module);
	}

	#[must_use]
	pub fn modules(&self) -> &[Module] {
		&self.modules
	}

	/// Applies [`Vec::truncate`] to the module array.
	pub fn truncate(&mut self, len: usize) {
		self.modules.truncate(len);
		self.symbols.retain(|_, val| val.0 < len);
	}
}

impl Default for Project {
	fn default() -> Self {
		let core = Module::core();

		let mut ret = Self {
			modules: vec![],
			symbols: DashMap::with_capacity(core.symbols.len() * 3),
		};

		ret.add_module(core);

		ret
	}
}

#[derive(Debug)]
pub struct Module {
	pub(super) name: String,
	/// Is `true` if this module had any native symbols loaded into it.
	/// Special rules are applied when performing semantic checks on source being
	/// compiled into a native module.
	pub(super) native: bool,
	#[allow(unused)]
	pub(super) jit: Arc<JitModule>,
	/// Functions, types, type aliases, et cetera.
	pub(super) symbols: Vec<Arc<dyn Symbol>>,
}

impl Module {
	#[must_use]
	pub fn name(&self) -> &str {
		&self.name
	}

	#[must_use]
	pub fn is_native(&self) -> bool {
		self.native
	}

	#[must_use]
	fn core() -> Self {
		let mut ret = Builder::new("vzscript".to_string(), false, false).build();

		for typeinfo in tsys::builtins() {
			ret.symbols.push(Arc::new(typeinfo));
		}

		ret
	}
}

// SAFETY:
// - Functions can only be hotswapped if there are no handles to them.
// - Data can not be modified while it is reachable by an existing runtime.
unsafe impl Send for Module {}
unsafe impl Sync for Module {}

pub trait Symbol: private::Sealed {
	#[must_use]
	fn header(&self) -> &SymbolHeader;
	#[must_use]
	fn header_mut(&mut self) -> &mut SymbolHeader;
}

impl<S: Symbol> From<&Arc<S>> for Handle<S> {
	fn from(value: &Arc<S>) -> Self {
		Handle(value.clone(), PhantomData)
	}
}

/// A storage implementation detail, exposed only so library users can
/// access common symbol metadata.
#[derive(Debug)]
pub struct SymbolHeader {
	pub name: String,
}

/// Thin wrapper around an [`Arc`] pointing to a [`Symbol`]'s storage. Attaching
/// a generic type allows the pointer to be safely downcast without any checks,
/// enabling safe, instant access to a symbol's data from anywhere in the engine.
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

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<S: Symbol> PartialEq for Handle<S> {
	/// Check that these are two handles to the same symbol in the same module.
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

// SAFETY: See safety disclaimer for `Module`.
unsafe impl<S: Symbol> Send for Handle<S> {}
unsafe impl<S: Symbol> Sync for Handle<S> {}

/// Internal handle. Like [`Handle`] but [`Weak`], allowing inter-symbol
/// relationships (without preventing in-place mutation or removal) in a way
/// that can't leak.
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct InHandle<S: Symbol>(Weak<S>, PhantomData<S>);

impl<S: Symbol> PartialEq for InHandle<S> {
	/// Check that these are two handles to the same symbol in the same module.
	fn eq(&self, other: &Self) -> bool {
		Weak::ptr_eq(&self.0, &other.0)
	}
}

impl<S: Symbol> Eq for InHandle<S> {}

// SAFETY: See safety disclaimer for `Module`.
unsafe impl<S: Symbol> Send for InHandle<S> {}
unsafe impl<S: Symbol> Sync for InHandle<S> {}

/// Used to prepare for building a [`Module`], primarily by registering native
/// functions to be callable by scripts.
pub struct Builder {
	name: String,
	native: bool,
	functions: Vec<Function>,
	types: Vec<TypeInfo>,
	jit: JITBuilder,
}

impl Builder {
	/// Also see [`JITBuilder::hotswap`].
	#[must_use]
	pub fn new(name: String, native: bool, hotswap: bool) -> Self {
		let mut builder = JITBuilder::new(cranelift_module::default_libcall_names())
			.expect("Failed to construct a Cranelift `JITBuilder`.");

		builder.hotswap(hotswap);

		Self {
			name,
			native,
			functions: vec![],
			types: vec![],
			jit: builder,
		}
	}

	#[must_use]
	pub fn build(self) -> Module {
		let mut symbols: Vec<Arc<dyn Symbol>> =
			Vec::with_capacity(self.functions.len() + self.types.len());

		for func in self.functions.into_iter() {
			symbols.push(Arc::new(func));
		}

		for tinfo in self.types.into_iter() {
			symbols.push(Arc::new(tinfo));
		}

		Module {
			name: self.name,
			native: self.native,
			jit: Arc::new(JitModule(MaybeUninit::new(JITModule::new(self.jit)))),
			symbols,
		}
	}
}

impl std::fmt::Debug for Builder {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Builder").field("name", &self.name).finish()
	}
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

/// Ensures proper JIT code de-allocation when all [`SymStore`]s
/// (and, by extension, all [`Handle`]s) drop their `Arc`s.
#[derive(Debug)]
pub(super) struct JitModule(pub(super) MaybeUninit<cranelift_jit::JITModule>);

impl Drop for JitModule {
	fn drop(&mut self) {
		unsafe {
			let i = std::mem::replace(&mut self.0, MaybeUninit::uninit());
			i.assume_init().free_memory();
		}
	}
}

mod private {
	use std::any::Any;

	pub trait Sealed: Any + Send + Sync + std::fmt::Debug {
		/// Boilerplate allowing upcasting from [`super::Symbol`] to [`Any`].
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