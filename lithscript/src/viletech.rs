//! A decoupled wing of LithScript for serving the VileTech Engine's specific needs.

mod lith;

mod decorate;
mod zscript;

use std::{
	collections::{HashMap, VecDeque},
	hash::Hasher,
	marker::PhantomData,
	path::{Path, PathBuf},
	sync::Arc,
};

use crate::{
	compile::{self, JitModule, Precompile, QName},
	issue::{self, Issue, IssueLevel},
	lir::AtomicSymbol,
	parse, Syn, Version,
};
use ariadne::{Report, ReportKind};
use dashmap::{mapref::one::Ref, DashMap};
use doomfront::{
	rowan::GreenNode,
	zdoom::{decorate::Syn as Decorate, zscript::Syn as ZScript},
	LangExt, ParseTree,
};
use parking_lot::Mutex;
use rayon::prelude::*;
use vfs::FileRef;

/// For parsing from VileTech's [virtual file system](vfs).
#[derive(Debug)]
pub struct IncludeTree(HashMap<PathBuf, Result<GreenNode, parse::Error>>);

impl IncludeTree {
	/// Traverses a VFS directory, parsing every text file with the extension "lith".
	/// Panics if `root` is not a directory.
	#[must_use]
	pub fn new(root: FileRef) -> Self {
		fn recur(fref: FileRef, files: &Mutex<HashMap<PathBuf, Result<GreenNode, parse::Error>>>) {
			fref.children().unwrap().par_bridge().for_each(|child| {
				if child.is_dir() {
					recur(child, files);
				}

				if !child.is_text() {
					return;
				}

				if !child.path_extension().is_some_and(|ext| ext == "lith") {
					return;
				}

				let result = parse::file(child.read_str());
				files.lock().insert(child.path().to_path_buf(), result);
			});
		}

		assert!(
			root.is_dir(),
			"`IncludeTree::new` expects a virtual directory"
		);

		let ret = Mutex::new(HashMap::new());
		recur(root, &ret);
		Self(ret.into_inner())
	}

	#[must_use]
	pub fn any_errors(&self) -> bool {
		self.0.values().any(|result| result.is_err())
	}

	pub fn drain_errors(&mut self) -> impl Iterator<Item = (PathBuf, parse::Error)> + '_ {
		self.0
			.drain()
			.filter_map(|(path, result)| result.err().map(|err| (path, err)))
	}

	pub fn into_inner(self) -> impl Iterator<Item = (PathBuf, GreenNode)> {
		assert!(!self.any_errors(), "encountered one or more parse errors");

		self.0
			.into_iter()
			.map(|(path, result)| (path, result.unwrap()))
	}
}

/// Thin, strongly-typed wrapper around [`compile::ContainerSource`].
#[derive(Debug)]
#[repr(transparent)]
pub struct ContainerSource<L: LangExt>(compile::ContainerSource, PhantomData<L>);

impl<L: LangExt> ContainerSource<L> {
	#[must_use]
	pub fn new(path: impl AsRef<Path>, ptree: ParseTree<L>) -> Self {
		assert!(
			!ptree.any_errors(),
			"encountered one or more parsing errors"
		);

		Self(
			compile::ContainerSource {
				path: path.as_ref().to_string_lossy().to_string(),
				root: ptree.into_green(),
			},
			PhantomData,
		)
	}
}

impl<L: LangExt> std::ops::Deref for ContainerSource<L> {
	type Target = compile::ContainerSource;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

/// A superset of [`compile::LibBuilder`].
pub struct LibBuilder {
	pub inner: compile::LibBuilder,
	// (G)ZDoom
	pub decorate: Vec<ContainerSource<Decorate>>,
	pub zscript: Vec<ContainerSource<ZScript>>,
}

impl LibBuilder {
	#[must_use]
	fn finish(self) -> LibSource {
		let subset = self.inner.finish();

		LibSource {
			name: subset.name,
			version: subset.version,
			jit: subset.jit,
			// SAFETY: `ContainerSource<L>` is `repr(transparent)` over
			// `compile::ContainerSource`.
			lith: unsafe { std::mem::transmute::<_, _>(subset.sources) },
			decorate: self.decorate,
			zscript: self.zscript,
		}
	}
}

/// A superset of [`compile::LibSource`].
pub(crate) struct LibSource {
	pub(self) name: String,
	pub(self) version: Version,
	pub(self) jit: Arc<JitModule>,

	pub(self) lith: Vec<ContainerSource<Syn>>,
	pub(self) decorate: Vec<ContainerSource<Decorate>>,
	pub(self) zscript: Vec<ContainerSource<ZScript>>,
}

/// A superset of [`compile::SymbolTable`].
#[derive(Debug, Default)]
pub(self) struct SymbolTable {
	pub(self) symbols: compile::SymbolTable,
	pub(self) znames: DashMap<ZName, AtomicSymbol>,
}

#[derive(Debug)]
#[must_use]
pub enum Outcome {
	Ok {
		precomps: VecDeque<Precompile>,
		symtab: compile::SymbolTable,
	},
	Fail {
		lib_name: String,
		issues: Vec<Issue>,
	},
}

pub fn compile(builders: impl IntoIterator<Item = LibBuilder>) -> Outcome {
	let symtab = SymbolTable::default();
	let mut precomps = VecDeque::new();

	for (i, source) in builders.into_iter().map(|b| b.finish()).enumerate() {
		let issues = Mutex::new(vec![]);

		[lith::pass1, decorate::pass1, zscript::pass1]
			.par_iter()
			.for_each(|function| {
				let pass = Pass1 {
					src: &source,
					symtab: &symtab,
					ix_lib: i,
				};

				function(pass);
			});

		[zscript::pass2].par_iter().for_each(|function| {
			let pass = Pass2 {
				src: &source,
				symtab: &symtab,
				issues: &issues,
			};

			function(pass);
		});

		if issues.lock().iter().any(|iss| iss.is_error()) {
			return Outcome::Fail {
				lib_name: source.name,
				issues: issues.into_inner(),
			};
		}

		precomps.push_front(Precompile {
			module: source.jit,
			lib_name: source.name,
			lib_vers: source.version,
			issues: issues.into_inner(),
		});
	}

	Outcome::Ok {
		precomps,
		symtab: symtab.symbols,
	}
}

// Pass 1 //////////////////////////////////////////////////////////////////////

/// Context for the first pass of LithV compilation.
///
/// Pass 1 involves declaring all "ZDoom-reachable" names; this means:
/// - container-level types and type aliases
/// - types and type aliases nested within container-level types
///
/// Due to the weakness of the compilation models of ZScript and DECORATE, it is
/// impossible to perform name resolution without completing these tables.
pub(self) struct Pass1<'s> {
	pub(self) src: &'s LibSource,
	pub(self) symtab: &'s SymbolTable,
	pub(self) ix_lib: usize,
}

impl Pass1<'_> {
	pub(self) fn declare<'s>(
		&self,
		ctr_path: impl AsRef<str>,
		resolver_parts: impl IntoIterator<Item = &'s str> + Copy,
	) {
		let name = QName::new_value_name(ctr_path.as_ref(), resolver_parts);

		let asym = self.symtab.symbols.declare(name, self.ix_lib);

		self.symtab
			.znames
			.insert(ZName(QName::new_value_name("", resolver_parts)), asym);
	}
}

/// Context for the second pass of LithV compilation.
///
/// Pass 2 contains all semantic checks. Now that all names are known, we can do
/// things like sanity checking ZScript inheritance hierarchies and compile-time
/// evaluation.
pub(self) struct Pass2<'s> {
	pub(self) src: &'s LibSource,
	pub(self) symtab: &'s SymbolTable,
	pub(self) issues: &'s Mutex<Vec<Issue>>,
}

impl Pass2<'_> {
	pub(self) fn get_z(&self, id_chain: &str) -> Option<Ref<ZName, AtomicSymbol>> {
		let qname = QName::Value(id_chain.replace('.', "::").into_boxed_str());
		self.symtab.znames.get(&ZName(qname))
	}

	pub(self) fn raise(&self, issue: Issue) {
		self.issues.lock().push(issue);
	}
}

// Details /////////////////////////////////////////////////////////////////////

/// A counterpart to [`QName`] for (G)ZDoom,
/// with ASCII case-insensitive equality comparison and hashing.
///
/// Note that these have different qualification rules names from ZScript and
/// DECORATE have no container equivalent, so a class has only its identifier;
/// if that class then declares an enum, the name is `::MyClass::MyEnum`.
#[derive(Debug)]
pub(self) struct ZName(QName);

impl ZName {
	#[must_use]
	pub fn as_str(&self) -> &str {
		self.0.as_str()
	}
}

impl PartialEq for ZName {
	fn eq(&self, other: &Self) -> bool {
		self.0.as_str().eq_ignore_ascii_case(other.0.as_str())
	}
}

impl Eq for ZName {}

impl std::hash::Hash for ZName {
	fn hash<H: Hasher>(&self, state: &mut H) {
		for c in self.0.as_str().chars() {
			c.to_ascii_lowercase().hash(state);
		}
	}
}

impl std::borrow::Borrow<str> for ZName {
	fn borrow(&self) -> &str {
		self.0.as_str()
	}
}

impl From<&QName> for ZName {
	fn from(value: &QName) -> Self {
		Self(value.clone())
	}
}
