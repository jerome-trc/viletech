//! A VZScript module is a single linkage unit.
//!
//! This is an equivalent concept to modules in LLVM, Rust, and Cranelift, but it
//! inherits the ZScript behavior of being compiled from an arbitrary number of
//! source files, rather than just one. Hence, `vzscript` (namespace `vzs`) is a
//! module (for language support), `viletech` (namespace `vtec`) is a module for
//! native engine functionality, et cetera.
//!
//! To get started, [create a `Builder`].
//!
//! [create a `Builder`]: Builder::new

use std::{hash::Hash, mem::MaybeUninit, sync::Arc};

use cranelift_jit::{JITBuilder, JITModule};
use slotmap::SlotMap;

use super::{tsys, Symbol};

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
	pub(super) symbols: SlotMap<SymbolSlotKey, Arc<dyn Symbol>>,
}

slotmap::new_key_type! {
	pub(super) struct ModuleSlotKey;
	pub(super) struct SymbolSlotKey;
}

impl Module {
	pub(super) const CORELIB_NAME: &str = "vzscript";

	#[must_use]
	pub fn name(&self) -> &str {
		&self.name
	}

	#[must_use]
	pub fn is_native(&self) -> bool {
		self.native
	}

	#[must_use]
	pub(super) fn core() -> Self {
		let mut ret = Builder::new(Self::CORELIB_NAME.to_string(), false, false).build();

		for typeinfo in tsys::builtins() {
			ret.symbols.insert(Arc::new(typeinfo));
		}

		ret
	}
}

// SAFETY:
// - Functions can only be hotswapped if there are no handles to them.
// - Data can not be modified while it is reachable by an existing runtime.
unsafe impl Send for Module {}
unsafe impl Sync for Module {}

/// Used to prepare for building a [`Module`], primarily by registering native
/// functions to be callable by scripts.
pub struct Builder {
	name: String,
	native: bool,
	symbols: SlotMap<SymbolSlotKey, Arc<dyn Symbol>>,
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
			symbols: SlotMap::default(),
			jit: builder,
		}
	}

	#[must_use]
	pub(super) fn _name(&self) -> &str {
		&self.name
	}

	#[must_use]
	pub(super) fn build(self) -> Module {
		Module {
			name: self.name,
			native: self.native,
			jit: Arc::new(JitModule(MaybeUninit::new(JITModule::new(self.jit)))),
			symbols: self.symbols,
		}
	}
}

impl std::fmt::Debug for Builder {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Builder").field("name", &self.name).finish()
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
