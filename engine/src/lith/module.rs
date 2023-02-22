//! A LithScript module is a single linkage unit.
//!
//! This is an equivalent concept to modules in LLVM, Rust, and Cranelift, but it
//! inherits the ZScript behavior of being compiled from an arbitrary number of
//! source files, rather than just one. Hence, `lith` is a module (for language
//! support), `vile` is a module for native engine functionality, et cetera.
//!
//! To get started, [create a `Builder`].
//!
//! [create a `Builder`]: Builder::new

use std::{
	any::{Any, TypeId},
	collections::HashMap,
	marker::PhantomData,
	mem::MaybeUninit,
	sync::{Arc, Weak},
};

use cranelift_jit::{JITBuilder, JITModule};

use super::{
	abi::Abi,
	func::TFunc,
	symbol::{Symbol, SymbolKey},
	tsys, Error, Function, TypeInfo,
};

#[derive(Debug, Clone)]
pub struct Module {
	pub(super) name: String,
	/// Is `true` if this module had any native symbols loaded into it.
	/// Special rules are applied when performing semantic checks on source being
	/// compiled into a native module.
	pub(super) native: bool,
	#[allow(unused)]
	pub(super) inner: Arc<JitModule>,
	/// Functions, types, type aliases, et cetera.
	pub(super) symbols: HashMap<SymbolKey, Arc<SymStore>>,
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

	pub fn get<S: Symbol>(&self, name: &str) -> Result<Handle<S>, Error> {
		let key = SymbolKey::new::<S>(name);
		let sym = self.symbols.get(&key).ok_or(Error::UnknownIdentifier)?;

		if sym.data.as_any().is::<S>() {
			Ok(Handle(sym.clone(), PhantomData))
		} else {
			Err(Error::TypeMismatch {
				expected: sym.data.type_id(),
				given: TypeId::of::<S>(),
			})
		}
	}

	/// Returns the `lith` module.
	#[must_use]
	pub fn core() -> Self {
		let mut ret = Builder::new("lith".to_string(), false).build();

		for (name, typeinfo) in tsys::builtins() {
			let key = SymbolKey::new::<TypeInfo>(&name);

			ret.symbols.insert(
				key,
				Arc::new(SymStore {
					name,
					data: Box::new(typeinfo),
				}),
			);
		}

		ret
	}
}

// SAFETY:
// - Functions can only be hotswapped if there are no handles to them.
// - Data can not be modified while it is reachable by an existing runtime.
unsafe impl Send for Module {}
unsafe impl Sync for Module {}

#[derive(Debug)]
pub(super) struct SymStore {
	pub(super) name: String,
	pub(super) data: Box<dyn Symbol>,
}

/// Thin wrapper around an [`Arc`] pointing to a [`Symbol`]'s storage. Attaching
/// a generic type allows the pointer to be safely downcast without any checks,
/// enabling safe, instant access to a symbol's data from anywhere in the engine.
#[derive(Debug)]
#[repr(transparent)]
pub struct Handle<S: Symbol>(Arc<SymStore>, PhantomData<S>);

impl<S: Symbol> Handle<S> {
	#[must_use]
	pub fn name(&self) -> &str {
		&self.0.name
	}
}

impl Handle<Function> {
	pub fn downcast<A, R>(&self) -> Result<Handle<TFunc<A, R>>, Error>
	where
		A: Abi,
		R: Abi,
	{
		if self.has_signature::<A, R>() {
			Ok(Handle(
				Arc::new(SymStore {
					name: self.name().to_string(),
					data: Box::new(TFunc::<A, R> {
						source: self.clone(),
						phantom: PhantomData,
					}),
				}),
				PhantomData,
			))
		} else {
			Err(Error::SignatureMismatch)
		}
	}
}

impl<S: 'static + Symbol> std::ops::Deref for Handle<S> {
	type Target = S;

	#[inline]
	fn deref(&self) -> &Self::Target {
		// SAFETY: Type correctness was already validated during handle acquisition.
		// Q: `downcast_ref_unchecked` when it stabilizes?
		unsafe { self.0.data.as_any().downcast_ref::<S>().unwrap_unchecked() }
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

// SAFETY: See safety disclaimer for `Module`
unsafe impl<S: Symbol> Send for Handle<S> {}
unsafe impl<S: Symbol> Sync for Handle<S> {}

/// Internal handle. Like [`Handle`] but [`Weak`], allowing inter-symbol
/// relationships (without preventing in-place mutation or removal) in a way
/// that can't leak.
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct InHandle<S: Symbol>(Weak<SymStore>, PhantomData<S>);

impl<S: Symbol> PartialEq for InHandle<S> {
	/// Check that these are two handles to the same symbol in the same module.
	fn eq(&self, other: &Self) -> bool {
		Weak::ptr_eq(&self.0, &other.0)
	}
}

impl<S: Symbol> Eq for InHandle<S> {}

// SAFETY: See safety disclaimer for `Module`
unsafe impl<S: Symbol> Send for InHandle<S> {}
unsafe impl<S: Symbol> Sync for InHandle<S> {}

/// Used to prepare for building a [`Module`], primarily by registering native
/// functions to be callable by scripts.
pub struct Builder {
	name: String,
	inner: JITBuilder,
}

impl Builder {
	/// Also see [`JITBuilder::hotswap`].
	#[must_use]
	pub fn new(name: String, hotswap: bool) -> Self {
		let mut builder = JITBuilder::new(cranelift_module::default_libcall_names())
			.expect("Failed to construct a Cranelift `JITBuilder`.");

		builder.hotswap(hotswap);

		Self {
			name,
			inner: builder,
		}
	}

	#[must_use]
	pub fn build(self) -> Module {
		Module {
			name: self.name,
			native: true,
			inner: Arc::new(JitModule(MaybeUninit::new(JITModule::new(self.inner)))),
			symbols: HashMap::default(),
		}
	}
}

/// Ensures proper JIT code de-allocation when all [`SymStore`]s
/// (and, by extension, all [`Handle`]s) drop their `Arc`s.
#[derive(Debug)]
#[repr(transparent)]
pub(super) struct JitModule(MaybeUninit<cranelift_jit::JITModule>);

impl Drop for JitModule {
	fn drop(&mut self) {
		unsafe {
			let i = std::mem::replace(&mut self.0, MaybeUninit::uninit());
			i.assume_init().free_memory();
		}
	}
}
