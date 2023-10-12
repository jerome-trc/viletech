//! Code that ties together the frontend, mid-section, and backend.

use std::hash::BuildHasherDefault;

use parking_lot::Mutex;
use rustc_hash::FxHasher;

use crate::{
	data::{Location, SymPtr},
	intern::{NameInterner, NameIx},
	issue::Issue,
	FxDashMap,
};

/// State and context tying together the frontend, mid-section, and backend.
#[derive(Debug)]
pub struct Compiler {
	// Input
	pub(crate) cfg: Config,
	// State
	pub(crate) issues: Mutex<Vec<Issue>>,
	pub(crate) failed: bool,
	// Storage
	pub(crate) symbols: FxDashMap<Location, SymPtr>,
	// Interning
	pub(crate) names: NameInterner,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
	/// Are lints emitted?
	pub pedantic: bool,
}

impl Compiler {
	#[must_use]
	pub fn new(config: Config) -> Self {
		Self {
			cfg: config,
			issues: Mutex::default(),
			failed: false,
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
		self.symbols.iter().for_each(|kvp| unsafe {
			if let Some(sym_ptr) = kvp.value().as_ptr() {
				std::ptr::drop_in_place(sym_ptr.as_ptr());
			}
		});
	}
}

pub(crate) type Scope = im::HashMap<NameIx, SymPtr, BuildHasherDefault<FxHasher>>;
