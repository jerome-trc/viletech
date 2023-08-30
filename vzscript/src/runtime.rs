//! The VZScript execution context: stack, heap, garbage collector, coroutines...

use std::sync::Arc;

use parking_lot::RwLock;

use crate::heap::Heap;

/// Context for VZScript execution.
///
/// Fully re-entrant; VZS has no global state.
pub struct Runtime {
	pub(super) heap: Heap,
}

impl Runtime {
	#[must_use]
	pub(crate) fn new() -> Arc<RwLock<Self>> {
		Arc::new(RwLock::new(Self {
			heap: Heap::default(),
		}))
	}
}
