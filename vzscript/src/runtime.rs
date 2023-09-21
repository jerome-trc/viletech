//! The VZScript execution context: stack, heap, garbage collector, coroutines...

use std::pin::Pin;

use util::rstring::RString;

use crate::{heap::Heap, rti, FxDashMap, FxDashMapView, FxDashSet, ZName};

/// Context for VZScript execution.
///
/// Fully re-entrant; VZS has no global state.
#[derive(Debug)]
pub struct Runtime {
	pub(super) heap: Heap,
	pub(super) rtinfo: FxDashMapView<ZName, rti::Record>,
	pub(super) strings: FxDashSet<RString>,
}

/// This is the only way to interact with a `Runtime`.
pub type RuntimePtr = Pin<Box<Runtime>>;

impl Runtime {
	#[must_use]
	pub(crate) fn new(strings: FxDashSet<RString>) -> RuntimePtr {
		Box::pin(Self {
			heap: Heap::default(),
			rtinfo: FxDashMap::default().into_read_only(),
			strings,
		})
	}
}
