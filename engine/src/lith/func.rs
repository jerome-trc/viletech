//! Functions and everything related.

/*

Copyright (C) 2022 ***REMOVED***

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <http://www.gnu.org/licenses/>.

*/

use std::ffi::c_void;

use cranelift_module::FuncId;

use super::interop::NativeFnBox;

/// Internal storage of the function code pointer and metadata.
pub(super) struct FunctionInfo {
	/// This is a pointer to either a JIT-compiled function or a C trampoline
	/// for a native function. Either way, never try to deallocate it.
	pub(super) _code: *const c_void,
	pub(super) _flags: FunctionFlags,
	/// The "source of truth" for native functions. Gets transmuted into two
	/// void pointers, each of which is stored as a global in the JIT binary,
	/// and then passed to a C-linkage trampoline.
	pub(super) _native: Option<NativeFnBox>,
	pub(super) _id: FuncId,
}

bitflags::bitflags! {
	pub struct FunctionFlags: u8 {
		/// This function has been marked as being compile-time evaluable.
		const CEVAL = 1 << 0;
	}
}

impl FunctionInfo {
	#[must_use]
	pub(crate) fn _is_native(&self) -> bool {
		self._native.is_some()
	}

	#[must_use]
	pub(crate) fn _is_ceval(&self) -> bool {
		self._flags.contains(FunctionFlags::CEVAL)
	}
}
