//! Implementing the Rust/LithScript language boundary.

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

#![allow(improper_ctypes_definitions)]

use std::{any::Any, ffi::c_void};

use paste::paste;

use super::word::Word;

pub(super) type NativeFnPtr<const ARG_C: usize, const RET_C: usize> =
	*mut (dyn 'static + Send + FnMut([Word; ARG_C]) -> [Word; RET_C]);

pub(super) type NativeFnBox = Box<dyn 'static + Send + Any>;

macro_rules! c_trampoline {
	($arg_n:tt, $ret_n:tt) => {
		paste! {
			unsafe extern "C" fn [<c_trampoline_a $arg_n _r $ret_n>](
				function: (*const c_void, *const c_void),
				args: [Word; $arg_n],
			) -> [Word; $ret_n] {
				let func = std::mem::transmute::<_, NativeFnPtr<{$arg_n}, {$ret_n}>>(function);
				(*func)(args)
			}
		}
	};
}

macro_rules! c_trampolines {
	($($arg_n:tt),+) => {
		$(
			c_trampoline! { $arg_n, 0 }
			c_trampoline! { $arg_n, 1 }
			c_trampoline! { $arg_n, 2 }
			c_trampoline! { $arg_n, 3 }
			c_trampoline! { $arg_n, 4 }
		)+

		paste! {
			/// Can be indexed into with `RET_C * (MAX_RETS + 1) + ARG_C`.
			pub(super) const C_TRAMPOLINES: &[*const c_void] = &[
				$(
					[<c_trampoline_a $arg_n _r 0>] as *const c_void,
					[<c_trampoline_a $arg_n _r 1>] as *const c_void,
					[<c_trampoline_a $arg_n _r 2>] as *const c_void,
					[<c_trampoline_a $arg_n _r 3>] as *const c_void,
					[<c_trampoline_a $arg_n _r 4>] as *const c_void,
				)+
			];
		}
	};
}

c_trampolines!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
