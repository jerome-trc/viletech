//! Types relevant to both the frontend and backend.

use std::{ffi::c_void, hash::Hash, sync::Arc};

use arc_swap::ArcSwap;
use parking_lot::Mutex;
use rustc_hash::FxHashMap;
use sharded_slab::Slab;
use util::rstring::RString;

use crate::{
	extend::{CompExt, NoExt},
	front::{self, Symbol, UndefKind},
	issue::Issue,
	FileTree, FxDashMap, Version,
};

/// Ties the frontend and backend together.
pub struct Compiler<E: CompExt> {
	pub(crate) cur_lib: usize,
	pub(crate) stage: Stage,
	pub(crate) interner: Interner,
	pub(crate) sources: Vec<LibSource<E>>,
	pub(crate) containers: FxDashMap<String, front::Container>,
	pub(crate) issues: Mutex<Vec<Issue>>,
	pub ext: E,
}

impl<E: CompExt> Compiler<E> {
	#[must_use]
	pub fn new(options: Options<E>, sources: impl IntoIterator<Item = LibSource<E>>) -> Self {
		let sources: Vec<_> = sources
			.into_iter()
			.map(|s| {
				assert!(!s.files.any_errors(), "cannot compile due to parse errors");
				s
			})
			.collect();

		assert!(
			!sources.is_empty(),
			"`Compiler::new` needs at least one `LibSource`"
		);

		Self {
			cur_lib: 0,
			stage: Stage::Declaration,
			interner: Interner::default(),
			sources,
			containers: FxDashMap::default(),
			issues: Mutex::default(),
			ext: options.ext,
		}
	}

	/// Prepare a series of checked libraries for code-gen.
	/// Will panic if not all libraries have been successfully checked.
	pub fn seal(&mut self) {
		assert!(self.cur_lib == self.sources.len());
		assert!(self.stage == Stage::Declaration);
		self.stage = Stage::CodeGen;
	}

	/// Panics if no container exists at the given path.
	pub fn native<F>(&self, path: &str, mut callback: F)
	where
		F: FnMut(&mut Scope),
	{
		assert_eq!(self.stage, Stage::Import);

		let Some(mut kvp) = self.containers.get_mut(path) else {
			panic!("failed to find container: `{path}`");
		};

		callback(&mut kvp.value_mut().decls);
	}

	/// Panics if the given function was not pre-declared in scope by the container's file.
	///
	/// # Safety
	///
	/// The given pointer must use the `extern "C"` ABI.
	pub unsafe fn native_fn(&self, name: &str, ptr: *mut c_void, scope: &Scope) {
		let iname = self.interner.intern(name);

		let symbol_ptr = scope
			.get(&iname)
			.unwrap_or_else(|| panic!("failed to find pre-declared function: `{name}`"));

		symbol_ptr.rcu(|p| {
			assert!(matches!(
				p.as_ref(),
				Symbol::Undefined {
					kind: UndefKind::Function,
					..
				}
			));

			Symbol::Function {
				iname,
				typedef: todo!(),
				body: None,
			}
		});
	}

	/// # Safety
	///
	/// `ptr` must not be to a function, and it must be valid to dereference
	/// as per Rust's rules on undefined behavior.
	pub unsafe fn native_data(&self, name: &str, ptr: *mut c_void, scope: &Scope) {
		let iname = self.interner.intern(name);

		let symbol_ptr = scope
			.get(&iname)
			.unwrap_or_else(|| panic!("failed to find pre-declared data object: `{name}`"));

		symbol_ptr.rcu(|p| {
			let Symbol::Undefined {
				kind: UndefKind::Value { mutable },
				..
			} = p.as_ref() else {
				panic!("symbol kind mismatch");
			};

			Symbol::Value {
				iname,
				typedef: todo!(),
				mutable: *mutable,
			}
		});
	}

	pub fn raise(&self, issue: Issue) {
		self.issues.lock().push(issue);
	}

	#[must_use]
	pub fn any_errors(&self) -> bool {
		let guard = self.issues.lock();
		guard.iter().any(|iss| iss.is_error())
	}

	#[must_use]
	pub fn interner(&self) -> &Interner {
		&self.interner
	}
}

pub struct Options<E: CompExt> {
	pub ext: E,
	pub optimization: Optimization,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stage {
	/// A newly-constructed [`Compiler`] is in this state.
	/// It should be passed to [`front::declare_symbols`].
	Declaration,
	/// The compiler is in this state after calling [`front::declare_symbols`].
	/// It should be passed to [`front::resolve_imports`].
	Import,
	/// The compiler is in this state after calling [`front::resolve_imports`].
	/// It should be passed to [`front::resolve_names`]
	Resolution,
	/// The compiler is in this state after calling [`front::resolve_names`].
	/// It should be passed to [`front::semantic_checks`].
	/// After that, the compiler is reset to [`Stage::Declaration`]
	/// to check the next library, if there are any left.
	Checking,
	/// The compiler is in this state after calling [`Compiler::seal`].
	/// At this point, it should be given to a middle stage or a backend.
	CodeGen,
}

/// To pass to [`Compiler::new`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Optimization {
	/// No optimization; only register allocation and machine code generation.
	L0,
	/// Additional code selection. Code is more compact and faster than [`Optimization::L0`],
	/// with practically the same compilation speed.
	L1,
	/// Common sub-expression elimination and sparse conditional constant propagation.
	/// Good for if the input MIR code is not pre-optimized.
	/// [`Optimization::L1`] is about 50% faster to compile than this.
	/// This is the default.
	#[default]
	L2,
	/// Additional register renaming and loop invariant code motion.
	/// [`Optimization::L2`] is about 50% faster to compile than this.
	L3,
}

/// See also [`Compiler`] and [`IName`].
#[derive(Debug, Default)]
pub struct Interner {
	pub(crate) slab: Slab<RString>,
	/// Values are indices into `slab`.
	pub(crate) map: FxDashMap<RString, IName>,
	// TODO: Probably possible to go a little faster than this...somehow.
}

impl Interner {
	#[must_use]
	pub fn resolve(&self, iname: IName) -> Option<sharded_slab::Entry<RString>> {
		self.slab.get(iname.0 as usize)
	}

	#[must_use]
	pub fn intern(&self, string: &str) -> IName {
		if let Some(kvp) = self.map.get(string) {
			return *kvp.value();
		}

		let rstring = RString::new(string);
		let ix = self.slab.insert(rstring.clone()).unwrap();
		let ret = IName(ix as u32);
		self.map.insert(rstring, ret);
		ret
	}
}

/// "Interned name". An index into [`Interner`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IName(u32);

pub type SymbolPtr = Arc<ArcSwap<front::Symbol>>;
pub type Scope<N = IName> = FxHashMap<N, SymbolPtr>;

#[derive(Debug)]
pub(crate) struct StackedScope<'s, N: Copy + Eq + Hash> {
	pub(crate) inner: &'s Scope<N>,
	pub(crate) is_addendum: bool,
}

impl<N: Copy + Eq + Hash> std::ops::Deref for StackedScope<'_, N> {
	type Target = Scope<N>;

	fn deref(&self) -> &Self::Target {
		self.inner
	}
}

/// Names are parameterizable to support frontends for third-party languages
/// with different rules around namespacing and case-sensitivity.
#[derive(Debug)]
pub(crate) struct ScopeStack<'s, N: Copy + Eq + Hash = IName>(Vec<StackedScope<'s, N>>);

impl<N: Copy + Eq + Hash> ScopeStack<'_, N> {
	pub(crate) fn lookup(&self, key: N) -> Option<&SymbolPtr> {
		self.0.iter().rev().find_map(|scope| scope.inner.get(&key))
	}
}

impl<'s, N: 'static + Copy + Eq + Hash> std::ops::Deref for ScopeStack<'s, N> {
	type Target = Vec<StackedScope<'s, N>>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<N: 'static + Copy + Eq + Hash> std::ops::DerefMut for ScopeStack<'_, N> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl<'s, N: 'static + Copy + Eq + Hash> std::ops::Index<usize> for ScopeStack<'s, N> {
	type Output = StackedScope<'s, N>;

	fn index(&self, index: usize) -> &Self::Output {
		&self.0[index]
	}
}

impl<N: 'static + Copy + Eq + Hash> std::ops::IndexMut<usize> for ScopeStack<'_, N> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.0[index]
	}
}

impl<N: Copy + Eq + Hash> Default for ScopeStack<'_, N> {
	fn default() -> Self {
		Self(vec![])
	}
}

// LibSource ///////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LibSource<E: CompExt = NoExt> {
	pub name: String,
	/// Note: this is the Lith version this library is bound to,
	/// not the version of the library itself.
	pub version: Version,
	pub files: FileTree,
	pub ext: E::Input,
}
