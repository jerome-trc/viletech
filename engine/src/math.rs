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
along with this program.  If not, see <http://www.gnu.org/licenses/>.

*/

use std::ops::{AddAssign, DivAssign, MulAssign, Rem, RemAssign, SubAssign};

pub trait Numeric:
	Sized + Copy + num::Num + AddAssign + MulAssign + DivAssign + SubAssign + Rem + RemAssign
{
}

impl<T> Numeric for T where
	T: Sized + Copy + num::Num + AddAssign + MulAssign + DivAssign + SubAssign + Rem + RemAssign
{
}

/// Utility function for SSE SIMD operations.
/// [`core::arch::x86_84::_MM_SHUFFLE_`] is unstable; use this in the meantime.
#[must_use]
pub const fn mm_shuffle(e3: u32, e1: u32, e0: u32, e4: u32) -> i32 {
	((e3 << 6) | (e1 << 4) | (e0 << 2) | e4) as i32
}

/// Utility function for SSE SIMD operations.
#[must_use]
pub const fn mm_shuffle_rev(e4: u32, e3: u32, e2: u32, e1: u32) -> i32 {
	mm_shuffle(e4, e3, e2, e1)
}

/// Utility function for SSE SIMD operations.
#[must_use]
pub const fn mm_shuffle_fwd(e1: u32, e2: u32, e3: u32, e4: u32) -> i32 {
	mm_shuffle(e4, e3, e2, e1)
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Rect4<T>
where
	T: Numeric,
{
	left: T,
	top: T,
	width: T,
	height: T,
}

impl<T> Rect4<T>
where
	T: Numeric,
{
	#[must_use]
	pub fn right(&self) -> T {
		self.left + self.width
	}

	#[must_use]
	pub fn bottom(&self) -> T {
		self.top + self.height
	}

	#[must_use]
	pub fn perimeter(&self) -> T {
		self.width + self.width + self.height + self.height
	}

	#[must_use]
	pub fn area(&self) -> T {
		self.width * self.height
	}

	pub fn offset(&mut self, x: T, y: T) {
		self.left += x;
		self.top += y;
	}
}

pub type URect8 = Rect4<u8>;
pub type URect16 = Rect4<u16>;
pub type URect32 = Rect4<u32>;
pub type URect64 = Rect4<u64>;

pub type IRect8 = Rect4<i8>;
pub type IRect16 = Rect4<i16>;
pub type IRect32 = Rect4<i32>;
pub type IRect64 = Rect4<i64>;

pub type FRect32 = Rect4<f32>;
pub type FRect64 = Rect4<f64>;
