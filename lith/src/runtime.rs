use parking_lot::Mutex;

use crate::{back::JitModule, rti, FxDashView};

/// Context for Lithica execution.
///
/// Fully re-entrant; Lith has no global state.
#[derive(Debug)]
pub struct Runtime {
	pub(crate) rtinfo: FxDashView<String, rti::Record>,
	/// Left untouched by the runtime; just needs to be here so that its
	/// memory does not get freed until it has no more users.
	pub(crate) module: JitModule,
	/// Comes from [`crate::compile::Compiler::arenas`]. Every pointer in
	/// [`Self::rtinfo`] references this memory.
	///
	/// Left untouched by the runtime; they just need to be here so that their
	/// memory does not get freed until it has no more users.
	pub(crate) arenas: Vec<Mutex<bumpalo::Bump>>,
}

/// A pointer to a structure of this type gets weaved through all Lith calls.
#[derive(Debug)]
#[repr(C)]
pub struct Context<U: Sized> {
	rt: *mut Runtime,
	user: *mut U,
}
