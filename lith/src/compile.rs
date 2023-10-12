//! Code that ties together the frontend, mid-section, and backend.

use parking_lot::Mutex;

use crate::{intern::NameInterner, issue::Issue};

/// State and context tying together the frontend, mid-section, and backend.
#[derive(Debug)]
pub struct Compiler {
	// State
	pub(crate) issues: Mutex<Vec<Issue>>,
	pub(crate) failed: bool,
	// Interning
	pub(crate) names: NameInterner,
}

impl Compiler {
	#[must_use]
	pub fn new() -> Self {
		Self {
			issues: Mutex::default(),
			failed: false,
			names: NameInterner::default(),
		}
	}
}
