//! Code that ties together the frontend, mid-section, and backend.

pub(crate) mod baselib;
mod module;

#[cfg(test)]
mod test;

use std::{any::TypeId, cmp::Ordering};

use cranelift::{
	codegen::ir::UserExternalName,
	prelude::{settings::OptLevel, AbiParam, TrapCode},
};
use crossbeam::channel::{Receiver, Sender};
use parking_lot::Mutex;

use crate::{
	back::FunctionIr,
	filetree::{self, FileIx, FileTree},
	front::{
		sema::{CEval, MonoKey, MonoSig},
		sym::{Location, SymbolId},
	},
	intern::NameInterner,
	interop::Interop,
	issue::Issue,
	types::{FxDashMap, FxDashSet, FxIndexMap, IrPtr, Scope, SymOPtr, SymPtr, TypeNPtr, TypeOPtr},
	Error, ValVec, Version,
};

pub(crate) use module::*;

pub use crate::{
	back::finalize,
	front::{decl::declare_symbols, import::resolve_imports, sema::semantic_check},
};

/// State and context tying together the frontend, mid-section, and backend.
#[derive(Debug)]
pub struct Compiler {
	// Input
	pub(crate) cfg: Config,
	pub(crate) ftree: FileTree,
	pub(crate) libs: Vec<(LibMeta, FileIx)>,
	// State
	pub(crate) stage: Stage,
	pub(crate) issues: Mutex<Vec<Issue>>,
	pub(crate) failed: bool,
	// Storage
	/// One arena per thread in the Rayon global pool.
	pub(crate) arenas: Vec<Mutex<bumpalo::Bump>>,
	/// Scopes for symbols as well as containers.
	/// Container scopes are keyed via [`Location::full_file`].
	pub(crate) scopes: FxDashMap<Location, Scope>,
	pub(crate) symbols: FxDashMap<SymbolId, SymOPtr>,
	pub(crate) types: FxDashSet<TypeOPtr>,
	/// Gets filled in upon success of the [sema phase](crate::sema).
	pub(crate) module: Option<JitModule>,
	pub(crate) ir: FxDashMap<UserExternalName, FunctionIr>,
	pub(crate) mono: FxDashMap<MonoSig, (Sender<IrPtr>, Receiver<IrPtr>)>,
	pub(crate) memo: FxDashMap<MonoKey, CEval>,
	pub(crate) native: NativeSymbols,
	pub(crate) sym_cache: SymCache,
	// Interning
	pub(crate) names: NameInterner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
	pub opt: OptLevel,
	/// Whether the JIT backend should allow function re-definition.
	pub hotswap: bool,
}

/// Note that a Lithica library is *not* a compilation unit.
/// An entire sequence of Lithica libraries is treated as one compilation unit.
#[derive(Debug)]
pub struct LibMeta {
	pub name: String,
	pub version: Version,
	pub native: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stage {
	#[default]
	Registration,
	Declaration,
	Import,
	Sema,
	CodeGen,
}

impl Compiler {
	const TOTAL_ALLOC_LIMIT: usize = 1024 * 1024 * 64;

	#[must_use]
	pub fn new(config: Config) -> Self {
		let mut ret = Self {
			cfg: config,
			ftree: FileTree::default(),
			libs: vec![],
			stage: Stage::default(),
			issues: Mutex::default(),
			failed: false,
			arenas: {
				let mut v = vec![];

				let threads = rayon::current_num_threads();
				let alloc_limit = Self::TOTAL_ALLOC_LIMIT / threads;

				for _ in 0..threads {
					let arena = bumpalo::Bump::new();
					arena.set_allocation_limit(Some(alloc_limit));
					v.push(Mutex::new(arena));
				}

				v
			},
			scopes: FxDashMap::default(),
			symbols: FxDashMap::default(),
			types: FxDashSet::default(),
			module: None,
			ir: FxDashMap::default(),
			memo: FxDashMap::default(),
			mono: FxDashMap::default(),
			native: NativeSymbols {
				user_ctx_t: TypeId::of::<()>(),
				functions: FxIndexMap::default(),
			},
			sym_cache: SymCache::default(),
			names: NameInterner::default(),
		};

		ret.libs.push(ret.baselib_meta());

		ret
	}

	/// The file index returned by `sourcer` must be a [`filetree::Node::Folder`].
	/// Otherwise, this function will panic.
	pub fn register_lib<F>(&mut self, meta: LibMeta, mut sourcer: F) -> Result<(), Vec<Error>>
	where
		F: FnMut(&mut FileTree) -> Result<FileIx, Vec<Error>>,
	{
		assert_eq!(self.stage, Stage::Registration);

		assert!(!meta.name.is_empty(), "`LibMeta::name` cannot be empty");

		if meta.name.eq_ignore_ascii_case("lith") || meta.name.eq_ignore_ascii_case("lithica") {
			panic!("`lithica` and `lith` are reserved library names");
		}

		let prev_ftn_count = self.ftree.graph.node_count();

		let lib_root = match sourcer(&mut self.ftree) {
			Ok(l) => l,
			Err(errs) => {
				self.failed = true;
				return Err(errs);
			}
		};

		assert!(matches!(
			self.ftree.graph[lib_root],
			filetree::Node::Folder { .. }
		));

		for i in prev_ftn_count..self.ftree.graph.node_count() {
			let ix = FileIx::new(i);

			let ftn = &self.ftree.graph[ix];

			let filetree::Node::File { ptree, .. } = ftn else {
				continue;
			};

			if ptree.any_errors() {
				self.failed = true;
				return Err(vec![Error::Parse]);
			}
		}

		self.libs.push((meta, lib_root));
		Ok(())
	}

	/// Panics if:
	/// - this compiler state has already moved on past the library registration stage.
	/// - [`Self::register_lib`] was never called.
	pub fn finish_registration(&mut self) {
		assert_eq!(self.stage, Stage::Registration);

		assert!(
			!self.libs.is_empty(),
			"compilation requires at least one valid registered library"
		);

		for (path, ptree) in self.ftree.files() {
			if ptree.any_errors() {
				panic!("library registration failed: file {path} has parse errors");
			}
		}

		assert!(
			!self.failed,
			"compilation cannot continue due to parsing errors"
		);

		self.stage = Stage::Declaration;
	}

	/// This is provided as a separate method from [`Self::new`] to:
	/// - isolate unsafe behavior
	/// - allow building a map in parallel to the library registration pass if desired
	///
	/// # Safety
	///
	/// See safety sections under [`NativeSymbols`].
	pub unsafe fn register_native(&mut self, native: NativeSymbols) {
		assert_eq!(self.stage, Stage::Declaration);
		self.native = native;
	}

	/// Have any fatal [issues](Issue) been encountered thus far?
	/// Attempting to send this compiler state to the next phase in the pipeline
	/// will panic if this is `true`.
	#[must_use]
	pub fn failed(&self) -> bool {
		self.failed
	}

	/// Provided so that a new buffer does not have to be allocated to sort the
	/// output of [`Self::drain_issues`].
	pub fn sort_issues<F>(&mut self, comparator: F)
	where
		F: FnMut(&Issue, &Issue) -> Ordering,
	{
		self.issues.get_mut().sort_by(comparator)
	}

	pub fn drain_issues(&mut self) -> impl Iterator<Item = Issue> + '_ {
		self.issues.get_mut().drain(..)
	}

	#[must_use]
	pub fn file_tree(&self) -> &FileTree {
		&self.ftree
	}

	/// Frees all memory (excluding interned strings) and set the library back
	/// to its file registration stage, bringing this state back to where it was
	/// when [`Self::new`] was called. This allows repeated use of existing allocations
	/// for re-try upon compiler error.
	pub fn reset(&mut self) {
		// TODO: Check if parallelizing the heaviest `clear` calls would be
		// faster in the general case here.
		self.ftree.reset();
		self.libs.clear();
		self.libs.push(self.baselib_meta());
		self.symbols.clear();
		self.types.clear();
		self.ir.clear();
		self.scopes.clear();
		self.module = None;
		self.memo.clear();
		self.mono.clear();
		self.sym_cache = SymCache::default();
		self.native.functions.clear();

		for arena in &mut self.arenas {
			arena.get_mut().reset();
		}

		self.issues.get_mut().clear();
		self.failed = false;
		self.stage = Stage::default();
	}

	#[must_use]
	pub fn arena_mem_usage(&self) -> usize {
		let mut ret = 0;

		for arena in &self.arenas {
			ret += arena.lock().allocated_bytes_including_metadata();
		}

		ret
	}
}

/// Internal details.
impl Compiler {
	pub(crate) fn raise(&self, issue: Issue) {
		let mut guard = self.issues.lock();
		guard.push(issue);
	}

	#[must_use]
	pub(crate) fn any_errors(&self) -> bool {
		let guard = self.issues.lock();
		guard.iter().any(|iss| iss.is_error())
	}

	#[must_use]
	pub(crate) fn baselib_meta(&self) -> (LibMeta, FileIx) {
		let meta = LibMeta {
			name: "lith".to_string(),
			version: Version::V0_0_0,
			native: true,
		};

		let file_ix = self.ftree.find_child(self.ftree.root(), "lith").unwrap();

		(meta, file_ix)
	}
}

impl Drop for Compiler {
	fn drop(&mut self) {
		self.reset();
	}
}

#[derive(Debug)]
pub struct NativeSymbols {
	pub(crate) user_ctx_t: TypeId,
	pub(crate) functions: FxIndexMap<&'static str, NativeFn>,
}

#[derive(Debug)]
pub(crate) struct NativeFn {
	pub(crate) rt: Option<RuntimeNative>,
	pub(crate) ceval: Option<CEvalNative>,
}

#[derive(Debug)]
pub(crate) struct RuntimeNative {
	pub(crate) ptr: *const u8,
	pub(crate) params: &'static [AbiParam],
	pub(crate) returns: &'static [AbiParam],
}

unsafe impl Send for RuntimeNative {}
unsafe impl Sync for RuntimeNative {}

pub type CEvalNative = fn(ValVec) -> Result<ValVec, TrapCode>;

impl NativeSymbols {
	#[must_use]
	pub fn new<U: 'static>() -> Self {
		Self {
			user_ctx_t: TypeId::of::<U>(),
			functions: FxIndexMap::default(),
		}
	}

	/// # Safety
	///
	/// `runtime` must use the `extern "C"` ABI.
	pub unsafe fn register<U: 'static, F: Interop>(
		&mut self,
		name: &'static str,
		runtime: Option<F>,
		ceval: Option<CEvalNative>,
	) {
		assert_eq!(TypeId::of::<U>(), self.user_ctx_t);
		assert_eq!(std::mem::size_of::<F>(), std::mem::size_of::<fn()>());

		self.functions.insert(
			name,
			NativeFn {
				rt: runtime.map(|f| {
					let ptr = std::mem::transmute_copy(&f);

					RuntimeNative {
						ptr,
						params: F::PARAMS,
						returns: F::RETURNS,
					}
				}),
				ceval,
			},
		);
	}
}

/// "Look-up table symbol".
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LutSym {
	pub(crate) inner: SymPtr,
	pub(crate) imported: bool,
}

impl std::ops::Deref for LutSym {
	type Target = SymPtr;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

/// For use by [`crate::sema`].
#[derive(Debug)]
pub(crate) struct SymCache {
	pub(crate) void_t: TypeNPtr,
	pub(crate) bool_t: TypeNPtr,
	pub(crate) i8_t: TypeNPtr,
	pub(crate) u8_t: TypeNPtr,
	pub(crate) i16_t: TypeNPtr,
	pub(crate) u16_t: TypeNPtr,
	pub(crate) i32_t: TypeNPtr,
	pub(crate) u32_t: TypeNPtr,
	pub(crate) i64_t: TypeNPtr,
	pub(crate) u64_t: TypeNPtr,
	pub(crate) i128_t: TypeNPtr,
	pub(crate) u128_t: TypeNPtr,
	pub(crate) f32_t: TypeNPtr,
	pub(crate) f64_t: TypeNPtr,
	pub(crate) iname_t: TypeNPtr,
	pub(crate) never_t: TypeNPtr,
}

impl Default for SymCache {
	fn default() -> Self {
		Self {
			void_t: TypeNPtr::null(),
			bool_t: TypeNPtr::null(),
			i8_t: TypeNPtr::null(),
			u8_t: TypeNPtr::null(),
			i16_t: TypeNPtr::null(),
			u16_t: TypeNPtr::null(),
			i32_t: TypeNPtr::null(),
			u32_t: TypeNPtr::null(),
			i64_t: TypeNPtr::null(),
			u64_t: TypeNPtr::null(),
			i128_t: TypeNPtr::null(),
			u128_t: TypeNPtr::null(),
			f32_t: TypeNPtr::null(),
			f64_t: TypeNPtr::null(),
			iname_t: TypeNPtr::null(),
			never_t: TypeNPtr::null(),
		}
	}
}
