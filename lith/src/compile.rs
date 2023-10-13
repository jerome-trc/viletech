//! Code that ties together the frontend, mid-section, and backend.

use std::hash::BuildHasherDefault;

use parking_lot::Mutex;
use rustc_hash::FxHasher;

use crate::{
	data::{Location, SymPtr},
	filetree::FileTree,
	intern::{NameInterner, NameIx},
	issue::Issue,
	FxDashMap, Version,
};

/// State and context tying together the frontend, mid-section, and backend.
#[derive(Debug)]
pub struct Compiler {
	// Input
	pub(crate) cfg: Config,
	pub(crate) sources: Vec<LibSource>,
	// State
	pub(crate) issues: Mutex<Vec<Issue>>,
	pub(crate) failed: bool,
	// Storage
	/// One arena per thread in the Rayon global pool.
	pub(crate) arenas: Vec<Mutex<bumpalo::Bump>>,
	/// Each `Vec` in this field corresponds to an element in [`Self::sources`].
	/// Each element in a sub-vec corresponds to an element in [`FileTree::files`].
	pub(crate) containers: Vec<Vec<Scope>>,
	pub(crate) symbols: FxDashMap<Location, SymPtr>,
	// Interning
	pub(crate) names: NameInterner,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
	/// Are lints emitted?
	pub pedantic: bool,
}

#[derive(Debug)]
pub struct LibSource {
	pub name: String,
	pub version: Version,
	pub native: bool,
	pub filetree: FileTree,
}

impl Compiler {
	#[must_use]
	pub fn new(config: Config, sources: impl IntoIterator<Item = LibSource>) -> Self {
		let sources: Vec<_> = sources
			.into_iter()
			.map(|s| {
				assert!(s.filetree.valid(), "cannot compile due to parse errors");
				s
			})
			.collect();

		assert!(
			!sources.is_empty(),
			"`Compiler::new` needs at least one `LibSource`"
		);

		Self {
			cfg: config,
			sources,
			issues: Mutex::default(),
			failed: false,
			arenas: {
				let mut v = vec![];
				v.resize_with(rayon::current_num_threads(), Mutex::default);
				v
			},
			containers: vec![],
			symbols: FxDashMap::default(),
			names: NameInterner::default(),
		}
	}

	#[must_use]
	pub fn failed(&self) -> bool {
		self.failed
	}

	pub fn drain_issues(&mut self) -> impl Iterator<Item = Issue> + '_ {
		self.issues.get_mut().drain(..)
	}

	pub fn reset(&mut self) {
		self.containers.clear();

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

impl Drop for Compiler {
	fn drop(&mut self) {
		self.reset();
	}
}

pub(crate) type Scope = im::HashMap<NameIx, SymPtr, BuildHasherDefault<FxHasher>>;
