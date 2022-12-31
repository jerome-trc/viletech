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

use std::{
	any::{Any, TypeId},
	ffi::c_void,
};

use paste::paste;

use crate::count_tts;

use super::word::Word;

pub(super) type NativeFnPtr<const PARAM_C: usize, const RET_C: usize> =
	*mut (dyn 'static + Send + FnMut([Word; PARAM_C]) -> [Word; RET_C]);

pub(super) type NativeFnBox = Box<dyn 'static + Send + Any>;

macro_rules! c_trampoline {
	($param_n:tt, $ret_n:tt) => {
		paste! {
			unsafe extern "C" fn [<c_trampoline_a $param_n _r $ret_n>](
				function: (*const c_void, *const c_void),
				args: [Word; $param_n],
			) -> [Word; $ret_n] {
				let func = std::mem::transmute::<_, NativeFnPtr<{$param_n}, {$ret_n}>>(function);
				(*func)(args)
			}
		}
	};
}

macro_rules! c_trampolines {
	($($param_n:tt),+) => {
		$(
			c_trampoline! { $param_n, 0 }
			c_trampoline! { $param_n, 1 }
			c_trampoline! { $param_n, 2 }
			c_trampoline! { $param_n, 3 }
			c_trampoline! { $param_n, 4 }
		)+

		paste! {
			/// Can be indexed into with `RET_C * (MAX_RETS + 1) + PARAM_C`.
			pub(super) const C_TRAMPOLINES: &[*const c_void] = &[
				$(
					[<c_trampoline_a $param_n _r 0>] as *const c_void,
					[<c_trampoline_a $param_n _r 1>] as *const c_void,
					[<c_trampoline_a $param_n _r 2>] as *const c_void,
					[<c_trampoline_a $param_n _r 3>] as *const c_void,
					[<c_trampoline_a $param_n _r 4>] as *const c_void,
				)+
			];
		}
	};
}

c_trampolines!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);

/// Helper trait enabling tuples to be passed to and from LithScript functions.
///
/// Marked as `unsafe` since returning the wrong type IDs could easily lead to
/// UB when passing arguments to a function. The implementations generated
/// in this module are guaranteed to be safe; you should never need to implement
/// it yourself.
#[allow(clippy::missing_safety_doc)]
pub unsafe trait Params<const N: usize>: Sized {
	#[must_use]
	fn compose(self) -> [Word; N];

	#[must_use]
	fn decompose(args: [Word; N]) -> Self;

	#[must_use]
	fn type_ids() -> [TypeId; N];
}

/// Helper trait enabling tuples to be passed to and from LithScript functions.
///
/// Marked as `unsafe` since returning the wrong type IDs could easily lead to
/// UB when retrieving return values from a function. The implementations generated
/// in this module are guaranteed to be safe; you should never need to implement
/// it yourself.
#[allow(clippy::missing_safety_doc)]
pub unsafe trait Returns<const N: usize>: Sized {
	#[must_use]
	fn decompose(words: [Word; N]) -> Self;

	#[must_use]
	fn compose(self) -> [Word; N];

	#[must_use]
	fn type_ids() -> [TypeId; N];
}

macro_rules! impl_tuple_params {
	($($generic:ident),+;$($num:tt),+) => {
		unsafe impl<$($generic),+> Params<{ count_tts!($($generic) +) }> for ($($generic),+, )
		where
			$($generic: 'static + Into<Word> + From<Word>),+
		{
			fn compose(self) -> [Word; count_tts!($($generic) +)] {
				[
					$(self.$num.into()),+
				]
			}

			fn decompose(args: [Word; count_tts!($($generic) +)]) -> Self {
				($(args[$num].into()),+)
			}

			fn type_ids() -> [TypeId; count_tts!($($generic) +)] {
				[
					$(std::any::TypeId::of::<$generic>()),+
				]
			}
		}
	};
}

impl_tuple_params! {
	A;
	0
}
impl_tuple_params! {
	A, B;
	0, 1
}
impl_tuple_params! {
	A, B, C;
	0, 1, 2
}
impl_tuple_params! {
	A, B, C, D;
	0, 1, 2, 3
}
impl_tuple_params! {
	A, B, C, D, E;
	0, 1, 2, 3, 4
}
impl_tuple_params! {
	A, B, C, D, E, F;
	0, 1, 2, 3, 4, 5
}
impl_tuple_params! {
	A, B, C, D, E, F, G;
	0, 1, 2, 3, 4, 5, 6
}
impl_tuple_params! {
	A, B, C, D, E, F, G, H;
	0, 1, 2, 3, 4, 5, 6, 7
}
impl_tuple_params! {
	A, B, C, D, E, F, G, H, I;
	0, 1, 2, 3, 4, 5, 6, 7, 8
}
impl_tuple_params! {
	A, B, C, D, E, F, G, H, I, J;
	0, 1, 2, 3, 4, 5, 6, 7, 8, 9
}
impl_tuple_params! {
	A, B, C, D, E, F, G, H, I, J, K;
	0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10
}
impl_tuple_params! {
	A, B, C, D, E, F, G, H, I, J, K, L;
	0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11
}

macro_rules! impl_tuple_rets {
	($($generic:ident),+;$($num:tt),+) => {
		unsafe impl<$($generic),+> Returns<{ count_tts!($($generic) +) }> for ($($generic),+, )
		where
			$($generic: 'static + From<Word> + Into<Word>),+
		{
			fn decompose(rets: [Word; count_tts!($($generic) +)]) -> Self {
				($(rets[$num].into()),+)
			}

			fn compose(self) -> [Word; count_tts!($($generic) +)] {
				[
					$(self.$num.into()),+
				]
			}

			fn type_ids() -> [TypeId; count_tts!($($generic) +)] {
				[
					$(std::any::TypeId::of::<$generic>()),+
				]
			}
		}
	};
}

impl_tuple_rets! {
	A;
	0
}
impl_tuple_rets! {
	A, B;
	0, 1
}
impl_tuple_rets! {
	A, B, C;
	0, 1, 2
}
impl_tuple_rets! {
	A, B, C, D;
	0, 1, 2, 3
}
