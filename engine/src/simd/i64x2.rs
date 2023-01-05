#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use std::marker::PhantomData;

/// An `__m128i` used for holding 2 64-bit integers, signed or unsigned.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct I64X2<T: Int64>(pub(super) __m128i, PhantomData<T>);

impl<T: Int64> I64X2<T> {
	/// `e0` a.k.a. `x`; `e1` a.k.a. `y`
	#[inline(always)]
	#[must_use]
	pub fn new(e0: T, e1: T) -> Self {
		Self(
			unsafe { _mm_set_epi64x(e0.to_i64(), e1.to_i64()) },
			PhantomData,
		)
	}

	/// The returned vector can never be invalid, but moving from negative signed
	/// to unsigned will overflow, and moving from unsigned to signed will underflow.
	#[inline(always)]
	#[must_use]
	pub fn new_raw(inner: __m128i) -> Self {
		Self(inner, PhantomData)
	}

	#[inline(always)]
	#[must_use]
	pub fn zeroed() -> Self {
		Self(unsafe { _mm_setzero_si128() }, PhantomData)
	}

	#[inline(always)]
	#[must_use]
	pub fn splat(value: T) -> Self {
		Self(unsafe { _mm_set1_epi64x(value.to_i64()) }, PhantomData)
	}

	/// The first element is set to `value`; the second is zeroed.
	#[inline(always)]
	#[must_use]
	pub fn new_e0(value: T) -> Self {
		Self(unsafe { _mm_set_epi64x(value.to_i64(), 0) }, PhantomData)
	}
}

#[cfg(not(target_feature = "sse4.1"))]
impl<T: Int64> I64X2<T> {
	#[inline(always)]
	#[must_use]
	pub fn e0(self) -> T {
		use super::mm_shuffle;

		unsafe {
			let m128d = _mm_castsi128_pd(self.0);
			let m128d = _mm_shuffle_pd::<{ mm_shuffle(0, 0, 2, 0) }>(m128d, m128d);
			let m128i = _mm_castpd_si128(m128d);
			Int64::from_i64(_mm_cvtsi128_si64x(m128i))
		}
	}

	#[inline(always)]
	#[must_use]
	pub fn e1(self) -> T {
		Int64::from_i64(unsafe { _mm_cvtsi128_si64x(self.0) })
	}
}

#[cfg(target_feature = "sse4.1")]
impl<T: Int64> I64X2<T> {
	#[inline(always)]
	#[must_use]
	pub fn e0(self) -> T {
		Int64::from_i64(unsafe { _mm_extract_epi64::<0>(self.0) })
	}

	#[inline(always)]
	#[must_use]
	pub fn e1(self) -> T {
		Int64::from_i64(unsafe { _mm_extract_epi64::<1>(self.0) })
	}
}

impl<T: Int64> PartialEq for I64X2<T> {
	#[inline(always)]
	fn eq(&self, other: &Self) -> bool {
		unsafe {
			// From glam
			let cmp = _mm_cmpeq_epi64(self.0, other.0);
			(_mm_movemask_epi8(cmp) as u32 & 0x7) == 0x7
		}
	}
}

impl<T: Int64> Eq for I64X2<T> {}

impl<T: Int64> std::ops::Add for I64X2<T> {
	type Output = Self;

	#[inline(always)]
	fn add(self, rhs: Self) -> Self::Output {
		Self(unsafe { _mm_add_epi64(self.0, rhs.0) }, PhantomData)
	}
}

impl<T: Int64> std::ops::Sub for I64X2<T> {
	type Output = Self;

	#[inline(always)]
	fn sub(self, rhs: Self) -> Self::Output {
		Self(unsafe { _mm_sub_epi64(self.0, rhs.0) }, PhantomData)
	}
}

impl<T: Int64> std::ops::Mul for I64X2<T> {
	type Output = Self;

	#[inline(always)]
	fn mul(self, rhs: Self) -> Self::Output {
		// https://stackoverflow.com/questions/17863411/sse-multiplication-of-2-64-bit-integers
		// By user "EasyasPi", used under CC BY-SA 4.0
		// https://stackoverflow.com/users/4014461/easyaspi

		unsafe {
			let ac = _mm_mul_epu32(self.0, rhs.0);
			let b = _mm_srli_epi64::<32>(self.0);
			let bc = _mm_mul_epu32(b, rhs.0);
			let d = _mm_srli_epi64::<32>(rhs.0);
			let ad = _mm_mul_epu32(self.0, d);
			let high = _mm_add_epi64(bc, ad);
			let high = _mm_slli_epi64::<32>(high);
			Self(_mm_add_epi64(high, ac), PhantomData)
		}
	}
}

impl<T: Int64 + std::ops::Div<Output = T>> std::ops::Div for I64X2<T> {
	type Output = Self;

	#[inline(always)]
	fn div(self, rhs: Self) -> Self::Output {
		Self::new(self.e0() / rhs.e0(), self.e1() / rhs.e1())
	}
}

// Details /////////////////////////////////////////////////////////////////////

// [Rat] I am sad to say this is the cleanest way there is to do this

/// Both `i64` and `u64` can be packed into an `__m128i`.
/// A helper trait makes [`I64X2`] generic over both.
pub trait Int64: Default + Copy + Clone {
	#[must_use]
	fn to_i64(self) -> i64;
	#[must_use]
	fn from_i64(value: i64) -> Self;
}

impl sealed::Sealed for i64 {}
impl sealed::Sealed for u64 {}

impl Int64 for i64 {
	#[inline(always)]
	fn to_i64(self) -> i64 {
		self
	}

	#[inline(always)]
	fn from_i64(value: i64) -> Self {
		value
	}
}

impl Int64 for u64 {
	#[inline(always)]
	fn to_i64(self) -> i64 {
		self as i64
	}

	#[inline(always)]
	fn from_i64(value: i64) -> Self {
		value as u64
	}
}

mod sealed {
	pub trait Sealed {}
}
