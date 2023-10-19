use parking_lot::Mutex;
use rustc_hash::FxHashMap;

use crate::{back::JitModule, rti};

/// Context for Lithica execution.
///
/// Fully re-entrant; Lith has no global state.
#[derive(Debug)]
pub struct Runtime {
	pub(crate) function_rti: FxHashMap<String, rti::Store<rti::Function>>,
	pub(crate) data_rti: FxHashMap<String, rti::Store<rti::DataObj>>,
	pub(crate) type_rti: FxHashMap<String, rti::Store<rti::Rtti>>,
	/// Left untouched by the runtime; just needs to be here so that its
	/// memory does not get freed until it has no more users.
	#[allow(unused)]
	pub(crate) module: JitModule,
	/// Comes from [`crate::compile::Compiler::arenas`].
	/// Every pointer in the RTI maps references this memory.
	///
	/// Left untouched by the runtime; they just need to be here so that their
	/// memory does not get freed until it has no more users.
	#[allow(unused)]
	pub(crate) arenas: Vec<Mutex<bumpalo::Bump>>,
}

/// A pointer to a structure of this type gets weaved through all Lith calls.
#[derive(Debug)]
#[repr(C)]
pub struct Context {
	rt: *mut Runtime,
	user: *mut (),
}
