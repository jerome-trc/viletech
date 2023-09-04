//! Types relevant to both the frontend and backend.

use std::{
	ffi::c_void,
	hash::{Hash, Hasher},
	mem::ManuallyDrop,
	ops::Deref,
	sync::Arc,
};

use arc_swap::ArcSwap;
use doomfront::zdoom::{decorate, inctree::IncludeTree};
use parking_lot::Mutex;
use rustc_hash::FxHashMap;
use util::rstring::RString;

use crate::{
	front::{self, Symbol},
	issue::Issue,
	rti::{self},
	tsys::{TypeDef, TypeDefType},
	zname::ZName,
	FxDashMap, FxDashSet, Project, Version,
};

#[derive(Debug)]
pub struct LibSource {
	pub name: String,
	pub version: Version,
	pub inctree: crate::IncludeTree,
	pub decorate: Option<IncludeTree<decorate::Syn>>,
}

#[derive(Debug)]
pub struct Compiler {
	pub(crate) project: Project,
	pub(crate) builtins: Builtins,
	pub(crate) globals: Scope,
	pub(crate) names: FxDashSet<ZName>,
	pub(crate) paths: FxDashSet<RString>,
	pub(crate) strings: FxDashSet<RString>,
	pub(crate) sources: Vec<LibSource>,
	pub(crate) issues: Mutex<Vec<Issue>>,
	pub(crate) decl_done: bool,
}

impl Compiler {
	#[must_use]
	pub fn new(sources: impl IntoIterator<Item = LibSource>) -> Self {
		let sources: Vec<_> = sources
			.into_iter()
			.map(|s| {
				assert!(
					!s.inctree.any_errors(),
					"cannot compile due to parse errors"
				);

				assert!(
					s.inctree.missing.is_empty(),
					"cannot compile due to missing includes"
				);

				s
			})
			.collect();

		assert!(
			!sources.is_empty(),
			"`Compiler::new` needs at least one `LibSource`"
		);

		let mut project = Project::default();

		for (qname, builtin) in TypeDef::BUILTINS {
			let zname = ZName::from(RString::new(qname));
			let store = rti::Store::new(zname.clone(), builtin.clone());
			project.rti.insert(zname, rti::Record::new_type(store));
		}

		let builtins = Builtins {
			bool_t: project.get::<TypeDef>("vzscript.bool").unwrap().handle(),
			int32_t: project.get::<TypeDef>("vzscript.int32").unwrap().handle(),
			float64_t: project.get::<TypeDef>("vzscript.float64").unwrap().handle(),
			iname_t: project.get::<TypeDef>("vzscript.iname").unwrap().handle(),
			string_t: project.get::<TypeDef>("vzscript.string").unwrap().handle(),
		};

		Self {
			project,
			builtins,
			names: FxDashSet::default(),
			paths: FxDashSet::default(),
			strings: FxDashSet::default(),
			globals: Scope::default(),
			sources,
			issues: Mutex::default(),
			decl_done: false,
		}
	}

	#[must_use]
	pub fn any_errors(&self) -> bool {
		let guard = self.issues.lock();
		guard.iter().any(|iss| iss.is_error())
	}

	#[must_use]
	pub fn native(&mut self) -> NativeRegistrar {
		assert!(self.decl_done);
		assert!(!self.any_errors());

		// Circumventing the borrow checker via hacks as usual.
		let globals = std::mem::take(&mut self.globals);

		NativeRegistrar {
			compiler: self,
			globals,
		}
	}

	#[must_use]
	pub(crate) fn intern_name(&self, name: &str) -> ZName {
		if let Some(ret) = self.names.get(name) {
			return ret.clone();
		}

		let ret = ZName::from(RString::new(name));
		let _ = self.names.insert(ret.clone());
		ret
	}

	#[must_use]
	pub(crate) fn intern_path(&self, path: &str) -> RString {
		if let Some(ret) = self.paths.get(path) {
			return ret.clone();
		}

		let ret = RString::new(path);
		let _ = self.paths.insert(ret.clone());
		ret
	}

	#[must_use]
	pub(crate) fn intern_string(&self, string: &str) -> RString {
		if let Some(ret) = self.strings.get(string) {
			return ret.clone();
		}

		let ret = RString::new(string);
		let _ = self.strings.insert(ret.clone());
		ret
	}

	/// If `Err` is returned, it gives back `symbol` along with the pointer to the
	/// symbol that would have been clobbered.
	pub(crate) fn declare(
		&self,
		scope: &mut Scope,
		iname: ZName,
		symbol: Symbol,
	) -> Result<SymbolPtr, (Symbol, SymbolPtr)> {
		use std::collections::hash_map;

		match scope.entry(iname) {
			hash_map::Entry::Vacant(vac) => {
				let ret = SymbolPtr::from(symbol);
				vac.insert(ret.clone());
				Ok(ret)
			}
			hash_map::Entry::Occupied(occ) => Err((symbol, occ.get().clone())),
		}
	}

	pub(crate) fn raise(&self, issues: impl IntoIterator<Item = Issue>) {
		let mut guard = self.issues.lock();

		for issue in issues.into_iter() {
			guard.push(issue);
		}
	}
}

pub(crate) type SymbolPtr = Arc<ArcSwap<front::Symbol>>;
pub(crate) type SymbolTable = FxHashMap<ZName, SymbolPtr>;

#[derive(Debug, Default, Clone)]
pub(crate) struct Scope {
	pub(crate) inner: SymbolTable,
	/// Nicknames used for transpilation.
	pub(crate) nicks: SymbolTable,
}

impl std::ops::Deref for Scope {
	type Target = SymbolTable;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl std::ops::DerefMut for Scope {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

#[derive(Debug)]
pub struct NativeRegistrar<'c> {
	pub compiler: &'c mut Compiler,
	globals: Scope,
}

impl NativeRegistrar<'_> {
	/// # Safety
	///
	/// TODO: Finalize this API, determine invariants.
	pub unsafe fn function(&mut self, name: &str, ptr: *mut c_void) {
		let iname = self.compiler.intern_name(name);
		unimplemented!()
	}

	/// # Safety
	///
	/// TODO: Finalize this API, determine invariants.
	pub unsafe fn data(&mut self, name: &str, ptr: *mut c_void) {
		let iname = self.compiler.intern_name(name);
		unimplemented!()
	}
}

impl Drop for NativeRegistrar<'_> {
	fn drop(&mut self) {
		self.compiler.globals = std::mem::take(&mut self.globals);
	}
}

/// Cache handles to types which will be commonly referenced
/// to keep hash table lookups down.
#[derive(Debug)]
pub(crate) struct Builtins {
	pub(crate) bool_t: rti::Handle<TypeDef>,
	pub(crate) int32_t: rti::Handle<TypeDef>,
	pub(crate) float64_t: rti::Handle<TypeDef>,
	pub(crate) iname_t: rti::Handle<TypeDef>,
	pub(crate) string_t: rti::Handle<TypeDef>,
}
