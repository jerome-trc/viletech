//! The VZScript execution context: stack, heap, garbage collector, coroutines...

use std::sync::Arc;

use parking_lot::RwLock;
use util::rstring::RString;

use crate::{heap::Heap, FxDashSet};

/// Context for VZScript execution.
///
/// Fully re-entrant; VZS has no global state.
pub struct Runtime {
	pub(super) heap: Heap,
	pub(super) strings: FxDashSet<RString>,
}

impl Runtime {
	#[must_use]
	pub(crate) fn new(strings: FxDashSet<RString>) -> Arc<RwLock<Self>> {
		Arc::new(RwLock::new(Self {
			heap: Heap::default(),
			strings,
		}))
	}
}
