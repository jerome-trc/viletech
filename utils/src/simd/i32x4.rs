#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use std::marker::PhantomData;

use super::F32X4;

/// An `__m128i` used for holding 4 32-bit integers, signed or unsigned.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct I32X4<T: Int32>(pub(super) __m128i, PhantomData<T>);

impl<T: Int32 + std::fmt::Debug> std::fmt::Debug for I32X4<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let tup = (self.e0(), self.e1(), self.e2(), self.e3());

		f.debug_tuple("I32X4").field(&tup).field(&self.1).finish()
	}
}

impl<T: Int32> I32X4<T> {
	/// `e0` a.k.a. `x` a.k.a. `r`; `e1` a.k.a. `y` a.k.a. `g`;
	/// `e2` a.k.a. `z` a.k.a. `b`; `e3` a.k.a. `w` a.k.a. `a`
	#[must_use]
	pub fn new(e0: T, e1: T, e2: T, e3: T) -> Self {
		Self(
			unsafe { _mm_set_epi32(e0.to_i32(), e1.to_i32(), e2.to_i32(), e3.to_i32()) },
			PhantomData,
		)
	}

	/// The returned vector can never be invalid, but moving from negative signed
	/// to unsigned will overflow, and moving from unsigned to signed will underflow.
	#[must_use]
	pub fn new_raw(inner: __m128i) -> Self {
		Self(inner, PhantomData)
	}

	#[must_use]
	pub fn zeroed() -> Self {
		Self(unsafe { _mm_setzero_si128() }, PhantomData)
	}

	/// The first element is set to `value`, and all other components are zeroed.
	#[must_use]
	pub fn new_e0(value: T) -> Self {
		Self(unsafe { _mm_cvtsi32_si128(value.to_i32()) }, PhantomData)
	}

	#[must_use]
	pub fn splat(value: T) -> Self {
		Self(unsafe { _mm_set1_epi32(value.to_i32()) }, PhantomData)
	}
}

#[cfg(not(target_feature = "sse4.1"))]
impl<T: Int32> I32X4<T> {
	#[must_use]
	pub fn e0(self) -> T {
		use super::mm_shuffle;

		Int32::from_i32(unsafe {
			_mm_cvtsi128_si32(_mm_shuffle_epi32::<{ mm_shuffle(3, 3, 3, 3) }>(self.0))
		})
	}

	#[must_use]
	pub fn e1(self) -> T {
		use super::mm_shuffle;

		Int32::from_i32(unsafe {
			_mm_cvtsi128_si32(_mm_shuffle_epi32::<{ mm_shuffle(2, 2, 2, 2) }>(self.0))
		})
	}

	#[must_use]
	pub fn e2(self) -> T {
		use super::mm_shuffle;

		Int32::from_i32(unsafe {
			_mm_cvtsi128_si32(_mm_shuffle_epi32::<{ mm_shuffle(1, 1, 1, 1) }>(self.0))
		})
	}

	#[must_use]
	pub fn e3(self) -> T {
		Int32::from_i32(unsafe { _mm_cvtsi128_si32(self.0) })
	}
}

#[cfg(target_feature = "sse4.1")]
impl<T: Int32> I32X4<T> {
	#[must_use]
	pub fn e0(self) -> T {
		Int32::from_i32(unsafe { _mm_extract_epi32(self.0, 3) })
	}

	#[must_use]
	pub fn e1(self) -> T {
		Int32::from_i32(unsafe { _mm_extract_epi32(self.0, 2) })
	}

	#[must_use]
	pub fn e2(self) -> T {
		Int32::from_i32(unsafe { _mm_extract_epi32(self.0, 1) })
	}

	#[must_use]
	pub fn e3(self) -> T {
		Int32::from_i32(unsafe { _mm_extract_epi32(self.0, 0) })
	}
}

impl<T: Int32> From<F32X4> for I32X4<T> {
	fn from(value: F32X4) -> Self {
		Self(unsafe { _mm_castps_si128(value.0) }, PhantomData)
	}
}

impl<T: Int32> PartialEq for I32X4<T> {
	fn eq(&self, other: &Self) -> bool {
		unsafe {
			// From glam
			let cmp = _mm_cmpeq_epi32(self.0, other.0);
			(_mm_movemask_epi8(cmp) as u32 & 0x7) == 0x7
		}
	}
}

impl<T: Int32> Eq for I32X4<T> {}

impl<T: Int32> std::ops::Add for I32X4<T> {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self(unsafe { _mm_add_epi32(self.0, rhs.0) }, PhantomData)
	}
}

impl<T: Int32> std::ops::Sub for I32X4<T> {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self(unsafe { _mm_sub_epi32(self.0, rhs.0) }, PhantomData)
	}
}

impl<T: Int32> std::ops::Mul for I32X4<T> {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		// Both from Gaijin Entertainment

		#[cfg(target_feature = "sse4.1")]
		unsafe {
			Self(_mm_mullo_epi32(self.0, rhs.0), PhantomData)
		}

		#[cfg(not(target_feature = "sse4.1"))]
		unsafe {
			let tmp1 = _mm_mul_epu32(self.0, rhs.0);
			let tmp2 = _mm_mul_epu32(_mm_srli_si128(self.0, 4), _mm_srli_si128(rhs.0, 4));

			let tmp1 = _mm_shuffle_epi32::<{ mm_shuffle(0, 0, 2, 0) }>(tmp1);
			let tmp2 = _mm_shuffle_epi32::<{ mm_shuffle(0, 0, 2, 0) }>(tmp2);
			Self(_mm_unpacklo_epi32(tmp1, tmp2), PhantomData)
		}
	}
}

impl<T: Int32 + std::ops::Div<Output = T>> std::ops::Div for I32X4<T> {
	type Output = Self;

	fn div(self, rhs: Self) -> Self::Output {
		Self::new(
			self.e0() / rhs.e0(),
			self.e1() / rhs.e1(),
			self.e2() / rhs.e2(),
			self.e3() / rhs.e3(),
		)
	}
}

// Details /////////////////////////////////////////////////////////////////////

// (RAT) I am sad to say this is the cleanest way there is to do this.

/// Both `i32` and `u32` can be packed into an `__m128i`.
/// A helper trait makes [`I32X4`] generic over both.
pub trait Int32: Default + Copy + Clone {
	#[must_use]
	fn to_i32(self) -> i32;
	#[must_use]
	fn from_i32(value: i32) -> Self;
}

macro_rules! int32_impl {
	($int_t:ty) => {
		impl sealed::Sealed for $int_t {}

		impl Int32 for $int_t {
			fn to_i32(self) -> i32 {
				self as i32
			}

			fn from_i32(value: i32) -> Self {
				value as $int_t
			}
		}
	};
}

int32_impl!(i32);
int32_impl!(u32);
#[cfg(target_pointer_width = "32")]
int32_impl!(isize);
#[cfg(target_pointer_width = "32")]
int32_impl!(usize);

mod sealed {
	pub trait Sealed {}
}
