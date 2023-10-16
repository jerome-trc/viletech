//! Code that ties together the frontend, mid-section, and backend.

#[cfg(test)]
mod test;

use std::{cmp::Ordering, hash::BuildHasherDefault};

use cranelift::prelude::settings::OptLevel;
use parking_lot::Mutex;
use rustc_hash::FxHasher;

use crate::{
	data::{Location, SymPtr},
	filetree::{self, FileIx, FileTree},
	intern::{NameInterner, NameIx},
	issue::Issue,
	Error, FxDashMap, ValVec, Version,
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
	pub(crate) symbols: FxDashMap<Location, SymPtr>,
	// Interning
	pub(crate) names: NameInterner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
	pub opt: OptLevel,
	/// Whether the JIT backend should allow function re-definition.
	pub hotswap: bool,
}

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
				v.resize_with(rayon::current_num_threads(), Mutex::default);
				v
			},
			scopes: FxDashMap::default(),
			symbols: FxDashMap::default(),
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
			if let Some(sym_ptr) = kvp.value().as_ptr() {
				std::ptr::drop_in_place(sym_ptr.as_ptr());
			}
		});

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

pub type CEvalNative = fn(ValVec) -> ValVec;

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

pub(crate) type Scope = im::HashMap<NameIx, LutSym, BuildHasherDefault<FxHasher>>;
