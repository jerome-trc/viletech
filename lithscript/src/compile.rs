//! Types relevant to both the frontend and backend.

use std::{collections::HashMap, mem::MaybeUninit, path::Path, sync::Arc};

use arc_swap::ArcSwap;
use cranelift::prelude::settings::OptLevel;
use cranelift_jit::{JITBuilder, JITModule};
use dashmap::{mapref::one::Ref, DashMap};
use doomfront::{rowan::GreenNode, ParseTree};
use parking_lot::Mutex;

use crate::{
	issue::Issue,
	lir::{self, AtomicSymbol, LibSymbol},
	Syn, Version,
};

// QName ///////////////////////////////////////////////////////////////////////

/// A fully-qualified name with built-in namespacing.
///
/// `Value` is for functions and types; `Container` is for imports. A well-formed
/// Lith symbol name is made of two parts: a container path and a "resolver".
/// The path is "virtual"; always with a root separator and no extension.
/// The resolver is a series of identifiers starting with and separated by `::`.
///
/// As an example: `/lith/collect::TArray::new`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QName {
	Value(Box<str>),
	Container(Box<str>),
}

impl QName {
	/// Note that this automatically inserts a `::`
	/// between `container_path` and the final composed resolver.
	#[must_use]
	pub fn new_value_name<'s>(
		container_path: &str,
		resolver_parts: impl IntoIterator<Item = &'s str>,
	) -> Self {
		Self::Value(Self::compose(container_path, resolver_parts))
	}

	/// Note that this automatically inserts a `::`
	/// between `container_path` and the final composed resolver.
	#[must_use]
	pub fn new_container_name<'s>(
		container_path: &str,
		resolver_parts: impl IntoIterator<Item = &'s str>,
	) -> Self {
		Self::Container(Self::compose(container_path, resolver_parts))
	}

	#[must_use]
	fn compose<'s>(
		container_path: &str,
		resolver_parts: impl IntoIterator<Item = &'s str>,
	) -> Box<str> {
		let mut ret = String::new();
		ret.push_str(container_path);

		for part in resolver_parts.into_iter() {
			ret.push_str("::");
			ret.push_str(part);
		}

		ret.into_boxed_str()
	}

	#[must_use]
	pub fn as_str(&self) -> &str {
		match self {
			Self::Value(string) | Self::Container(string) => string,
		}
	}
}

// LibBuilder //////////////////////////////////////////////////////////////////

/// It is at this point when all native functions and data that JIT scripts can
/// call must be registered.
pub struct LibBuilder {
	jit: JITBuilder,
	pub name: String,
	pub version: Version,
	pub sources: Vec<ContainerSource>,
}

impl LibBuilder {
	#[must_use]
	pub fn new(
		name: String,
		version: Version,
		sources: Vec<ContainerSource>,
		opt: OptLevel,
	) -> Self {
		Self {
			jit: {
				let o_lvl = match opt {
					OptLevel::None => "none",
					OptLevel::Speed => "speed",
					OptLevel::SpeedAndSize => "speed_and_size",
				};

				JITBuilder::with_flags(
					&[
						("use_colocated_libcalls", "false"),
						("is_pic", "false"),
						("opt_level", o_lvl),
					],
					cranelift_module::default_libcall_names(),
				)
				.expect("JIT module builder creation failed")
			},
			name,
			version,
			sources,
		}
	}

	/// See [`JITBuilder::symbol`].
	pub fn symbol(&mut self, name: impl Into<String>, ptr: *const u8) {
		self.jit.symbol(name, ptr);
	}

	/// See [`JITBuilder::symbols`].
	pub fn symbols<I, N>(&mut self, symbols: I)
	where
		I: IntoIterator<Item = (N, *const u8)>,
		N: Into<String>,
	{
		self.jit.symbols(symbols);
	}

	#[must_use]
	pub(crate) fn finish(self) -> LibSource {
		LibSource {
			name: self.name,
			version: self.version,
			jit: Arc::new(JitModule(Mutex::new(MaybeUninit::new(JITModule::new(
				self.jit,
			))))),
			sources: self.sources,
		}
	}
}

// ContainerSource /////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ContainerSource {
	pub(crate) path: String,
	pub(crate) root: GreenNode,
}

impl ContainerSource {
	/// Panics if:
	/// - `ptree` has any errors attached.
	/// - `path` is extended with anything other than ".lith" (ASCII case-insensitively).
	#[must_use]
	pub fn new(path: impl AsRef<Path>, ptree: ParseTree<Syn>) -> Self {
		assert!(
			!ptree.any_errors(),
			"encountered one or more parsing errors"
		);

		let path = path.as_ref();

		assert!(
			path.extension()
				.is_some_and(|ext| ext.eq_ignore_ascii_case("lith")),
			"`path` must have the `.lith` extension"
		);

		let mut p = path.to_string_lossy().to_string();
		p.truncate(p.len() - 5);

		Self {
			path: p,
			root: ptree.into_inner(),
		}
	}
}

// LibSource ///////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub(crate) struct LibSource {
	pub(crate) name: String,
	pub(crate) version: Version,
	pub(crate) jit: Arc<JitModule>,
	pub(crate) sources: Vec<ContainerSource>,
}

// SymbolTable /////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct SymbolTable(pub(crate) DashMap<QName, AtomicSymbol>);

impl SymbolTable {
	pub(crate) fn declare(&self, name: QName, ix_lib: usize) -> AtomicSymbol {
		let ret = AtomicSymbol(Arc::new(LibSymbol {
			inner: ArcSwap::new(Arc::new(lir::Symbol::Unknown)),
			ix_lib,
		}));

		self.0.insert(name, ret.clone());

		ret
	}

	#[must_use]
	pub(crate) fn get(&self, name: &QName) -> Option<Ref<QName, AtomicSymbol>> {
		self.0.get(name)
	}
}

// ScopeStack //////////////////////////////////////////////////////////////////

/// Keys are parameterizable to support languages with case-insensitive names.
#[derive(Debug, Default)]
pub(crate) struct ScopeStack<N = QName>(pub(crate) Vec<HashMap<N, Variable>>);

#[derive(Debug)]
pub(crate) struct Variable {
	pub(crate) _mutable: bool,
}

// Precompile //////////////////////////////////////////////////////////////////

/// The output of lowering source to LIR, to be passed to [`crate::codegen`].
#[derive(Debug)]
pub struct Precompile {
	pub(crate) module: Arc<JitModule>,
	pub(crate) lib_name: String,
	pub(crate) lib_vers: Version,
	pub(crate) issues: Vec<Issue>,
}

impl Precompile {
	/// Of the compiler issues raised, were any fatal?
	#[must_use]
	pub fn any_errors(&self) -> bool {
		self.issues.iter().any(|iss| iss.is_error())
	}
}

// JitModule ///////////////////////////////////////////////////////////////////

/// Ensures proper JIT code de-allocation (from behind an [`Arc`]).
#[derive(Debug)]
pub struct JitModule(pub(crate) Mutex<MaybeUninit<JITModule>>);

impl Drop for JitModule {
	fn drop(&mut self) {
		unsafe {
			std::mem::replace(&mut self.0, Mutex::new(MaybeUninit::uninit()))
				.into_inner()
				.assume_init()
				.free_memory();
		}
	}
}

// TODO: Prove that this is safe when codegen machinery is in working order.
unsafe impl Send for JitModule {}
unsafe impl Sync for JitModule {}
