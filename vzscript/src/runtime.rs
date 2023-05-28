//! The VZScript execution context: stack, heap, garbage collector, coroutines...

use cranelift_interpreter::interpreter::{Interpreter, InterpreterState};

use crate::heap::Heap;

/// Context for VZScript execution.
///
/// Fully re-entrant; VZS has no global state.
pub struct Runtime {
	pub(super) heap: Heap,
	pub(super) _interpreter: Interpreter<'static>,
}

impl std::fmt::Debug for Runtime {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Runtime")
			.field("interpreter", &"<cranelift_interpreter::Interpreter>")
			.finish()
	}
}

impl Default for Runtime {
	fn default() -> Self {
		let istate = InterpreterState::default();

		Self {
			heap: Heap::default(),
			_interpreter: Interpreter::new(istate),
		}
	}
}
