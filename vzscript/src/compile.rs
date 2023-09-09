//! Code that ties together the frontend, mid-section, and backend.

pub(crate) mod intern;
pub(crate) mod symbol;

use append_only_vec::AppendOnlyVec;
use doomfront::zdoom::{decorate, inctree::IncludeTree};
use parking_lot::Mutex;
use rustc_hash::FxHashMap;
use util::rstring::RString;

use crate::{issue::Issue, rti, tsys::TypeDef, zname::ZName, FxDashSet, Project, Version};

use self::{
	intern::{NameInterner, NameIx, NsName, SymbolIx},
	symbol::{Definition, Location, Symbol},
};

pub type NativeSymbolTable = FxHashMap<&'static str, *const u8>;

#[derive(Debug)]
pub struct LibSource {
	pub name: String,
	pub version: Version,
	pub native: bool,
	pub inctree: crate::IncludeTree,
	pub decorate: Option<IncludeTree<decorate::Syn>>,
}

#[derive(Debug)]
pub struct Compiler {
	// Input
	pub(crate) sources: Vec<LibSource>,
	// State
	pub(crate) stage: Stage,
	pub(crate) issues: Mutex<Vec<Issue>>,
	pub(crate) failed: bool,
	// Storage
	pub(crate) project: Project,
	pub(crate) builtins: Builtins,
	pub(crate) globals: Scope,
	pub(crate) defs: AppendOnlyVec<Definition>,
	pub(crate) native: FxHashMap<&'static str, NativePtr>,
	/// One for each library, parallel to [`Self::sources`].
	pub(crate) namespaces: Vec<Scope>,
	pub(crate) symbols: AppendOnlyVec<Symbol>,
	// Interning
	pub(crate) strings: FxDashSet<RString>,
	pub(crate) names: NameInterner,
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

		#[must_use]
		fn register_builtin(
			project: &mut Project,
			qname: &'static str,
			tdef: &TypeDef,
		) -> rti::Handle<TypeDef> {
			let zname = ZName::from(RString::new(qname));
			let store = rti::Store::new(zname.clone(), tdef.clone());
			let record = rti::Record::new_type(store);
			let handle = record.handle_type();
			project.rti.insert(zname, record);
			handle
		}

		let builtins = Builtins {
			void_t: register_builtin(&mut project, "vzs.void", &TypeDef::BUILTIN_VOID),
			bool_t: register_builtin(&mut project, "vzs.bool", &TypeDef::BUILTIN_BOOL),
			int32_t: register_builtin(&mut project, "vzs.int32", &TypeDef::BUILTIN_INT32),
			uint32_t: register_builtin(&mut project, "vzs.uint32", &TypeDef::BUILTIN_UINT32),
			int64_t: register_builtin(&mut project, "vzs.int64", &TypeDef::BUILTIN_INT64),
			uint64_t: register_builtin(&mut project, "vzs.uint64", &TypeDef::BUILTIN_UINT64),
			float32_t: register_builtin(&mut project, "vzs.float32", &TypeDef::BUILTIN_FLOAT32),
			float64_t: register_builtin(&mut project, "vzs.float64", &TypeDef::BUILTIN_FLOAT64),
			iname_t: register_builtin(&mut project, "vzs.iname", &TypeDef::BUILTIN_INAME),
			string_t: register_builtin(&mut project, "vzs.string", &TypeDef::BUILTIN_STRING),
		};

		Self {
			sources,
			issues: Mutex::default(),
			stage: Stage::Declaration,
			failed: false,
			project,
			builtins,
			globals: Scope::default(),
			defs: AppendOnlyVec::new(),
			native: FxHashMap::default(),
			namespaces: vec![],
			symbols: AppendOnlyVec::new(),
			strings: FxDashSet::default(),
			names: NameInterner::default(),
		}
	}

	/// This is provided as a separate method from [`Self::new`] to:
	/// - isolate unsafe behavior
	/// - allow building the given map in parallel to the declaration pass
	///
	/// # Safety
	///
	/// - Dereferencing a data object pointer or calling a function pointer must
	/// never invoke any thread-unsafe behavior.
	pub unsafe fn native(&mut self, symbols: NativeSymbolTable) {
		assert!(matches!(self.stage, Stage::Declaration | Stage::Semantic));
		// SAFETY: `NativePtr` is `repr(transparent)` over `*const u8`.
		self.native = std::mem::transmute::<_, _>(symbols);
	}

	#[must_use]
	pub fn failed(&self) -> bool {
		self.failed
	}

	pub fn drain_issues(&mut self) -> impl Iterator<Item = Issue> + '_ {
		self.issues.get_mut().drain(..)
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

	#[must_use]
	pub(crate) fn get_corelib_type(&self, name: &str) -> &Symbol {
		let nsname = NsName::Type(self.names.intern_str(name));
		let &sym_ix = self.namespaces[0].get(&nsname).unwrap();
		self.symbol(sym_ix)
	}

	#[must_use]
	pub(crate) fn resolve_path(&self, location: Location) -> &str {
		let libsrc = &self.sources[location.lib_ix as usize];
		libsrc.inctree.files[location.file_ix as usize].path()
	}

	#[must_use]
	pub(crate) fn symbol(&self, ix: SymbolIx) -> &Symbol {
		&self.symbols[ix.0 as usize]
	}

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

pub(crate) type Scope = FxHashMap<NsName, SymbolIx>;

/// Cache handles to types which will be commonly referenced
/// to keep hash table lookups down.
#[derive(Debug)]
pub(crate) struct Builtins {
	pub(crate) void_t: rti::Handle<TypeDef>,
	pub(crate) bool_t: rti::Handle<TypeDef>,
	pub(crate) int32_t: rti::Handle<TypeDef>,
	pub(crate) uint32_t: rti::Handle<TypeDef>,
	pub(crate) int64_t: rti::Handle<TypeDef>,
	pub(crate) uint64_t: rti::Handle<TypeDef>,
	pub(crate) float32_t: rti::Handle<TypeDef>,
	pub(crate) float64_t: rti::Handle<TypeDef>,
	pub(crate) iname_t: rti::Handle<TypeDef>,
	pub(crate) string_t: rti::Handle<TypeDef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub(crate) struct NativePtr(pub(crate) *const u8);

// SAFETY: Caller of `Compiler::native` provides guarantees about given pointers.
unsafe impl Send for NativePtr {}
unsafe impl Sync for NativePtr {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stage {
	Declaration,
	Semantic,
	CodeGen,
}
