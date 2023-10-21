//! Code that ties together the frontend, mid-section, and backend.

#[cfg(test)]
mod test;

use std::cmp::Ordering;

use cranelift::{
	codegen::ir,
	prelude::{settings::OptLevel, AbiParam, TrapCode},
};
use cranelift_module::FuncId;
use parking_lot::Mutex;
use util::pushvec::PushVec;

use crate::{
	back::JitModule,
	data::{Location, SymbolId},
	filetree::{self, FileIx, FileTree},
	intern::NameInterner,
	interop::JitFn,
	issue::Issue,
	runtime,
	types::{FxDashMap, FxIndexMap, Scope, SymNPtr, SymPtr},
	Error, ValVec, Version,
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
	pub(crate) symbols: FxDashMap<SymbolId, SymPtr>,
	/// Gets filled in upon success of the [sema phase](crate::sema).
	pub(crate) module: Option<JitModule>,
	pub(crate) ir: PushVec<(FuncId, ir::Function)>,
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
		Self {
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
			module: None,
			ir: PushVec::new(),
			native: NativeSymbols::default(),
			sym_cache: SymCache::default(),
			names: NameInterner::default(),
		}
	}

	pub fn register_lib<F>(&mut self, meta: LibMeta, mut sourcer: F) -> Result<(), Vec<Error>>
	where
		F: FnMut(&mut FileTree) -> Result<FileIx, Vec<Error>>,
	{
		assert_eq!(self.stage, Stage::Registration);

		assert!(!meta.name.is_empty(), "`LibMeta::name` cannot be empty");

		let prev_ftn_count = self.ftree.graph.node_count();

		let lib_root = match sourcer(&mut self.ftree) {
			Ok(l) => l,
			Err(errs) => {
				self.failed = true;
				return Err(errs);
			}
		};

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

	pub fn reset(&mut self) {
		self.ftree.reset();
		self.libs.clear();
		self.scopes.clear();

		self.symbols.iter().for_each(|kvp| unsafe {
			kvp.drop_in_place();
		});

		self.ir = PushVec::new();

		for arena in &mut self.arenas {
			arena.get_mut().reset();
		}

		self.issues.get_mut().clear();
		self.failed = false;
		self.stage = Stage::Declaration;
	}

	// Internal ////////////////////////////////////////////////////////////////

	pub(crate) fn raise(&self, issue: Issue) {
		let mut guard = self.issues.lock();
		guard.push(issue);
	}

	#[must_use]
	pub(crate) fn any_errors(&self) -> bool {
		let guard = self.issues.lock();
		guard.iter().any(|iss| iss.is_error())
	}
}

impl Drop for Compiler {
	fn drop(&mut self) {
		self.reset();
	}
}

#[derive(Debug, Default)]
pub struct NativeSymbols {
	pub(crate) functions: FxIndexMap<&'static str, NativeFn>,
}

#[derive(Debug)]
pub(crate) struct NativeFn {
	pub(crate) rt: Option<RuntimeNative>,
	pub(crate) ceval: Option<CEvalNative>,
}

#[derive(Debug)]
pub(crate) struct RuntimeNative {
	pub(crate) ptr: extern "C" fn(*mut runtime::Context, ...),
	pub(crate) params: &'static [AbiParam],
	pub(crate) returns: &'static [AbiParam],
}

pub type CEvalNative = fn(ValVec) -> Result<ValVec, TrapCode>;

impl NativeSymbols {
	/// # Safety
	///
	/// `runtime` must use the `extern "C"` ABI.
	pub unsafe fn register<F: JitFn>(
		&mut self,
		name: &'static str,
		runtime: Option<F>,
		ceval: Option<CEvalNative>,
	) {
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
	pub(crate) void_t: SymNPtr,
	pub(crate) bool_t: SymNPtr,
	pub(crate) i8_t: SymNPtr,
	pub(crate) u8_t: SymNPtr,
	pub(crate) i16_t: SymNPtr,
	pub(crate) u16_t: SymNPtr,
	pub(crate) i32_t: SymNPtr,
	pub(crate) u32_t: SymNPtr,
	pub(crate) i64_t: SymNPtr,
	pub(crate) u64_t: SymNPtr,
	pub(crate) i128_t: SymNPtr,
	pub(crate) u128_t: SymNPtr,
	pub(crate) f32_t: SymNPtr,
	pub(crate) f64_t: SymNPtr,
}

impl Default for SymCache {
	fn default() -> Self {
		Self {
			void_t: SymNPtr::null(),
			bool_t: SymNPtr::null(),
			i8_t: SymNPtr::null(),
			u8_t: SymNPtr::null(),
			i16_t: SymNPtr::null(),
			u16_t: SymNPtr::null(),
			i32_t: SymNPtr::null(),
			u32_t: SymNPtr::null(),
			i64_t: SymNPtr::null(),
			u64_t: SymNPtr::null(),
			i128_t: SymNPtr::null(),
			u128_t: SymNPtr::null(),
			f32_t: SymNPtr::null(),
			f64_t: SymNPtr::null(),
		}
	}
}
