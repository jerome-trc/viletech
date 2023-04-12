#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::mm_shuffle;

/// An `__m128` used for holding 2 64-bit floating-point numbers.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct F64X2(pub(super) __m128d);

impl F64X2 {
	/// `e0` a.k.a. `x`; `e1` a.k.a. `y`
	#[must_use]
	pub fn new(e0: f64, e1: f64) -> Self {
		unsafe { Self(_mm_set_pd(e0, e1)) }
	}

	#[must_use]
	pub fn new_raw(inner: __m128d) -> Self {
		Self(inner)
	}

	#[must_use]
	pub fn zeroed() -> Self {
		unsafe { Self(_mm_setzero_pd()) }
	}

	/// The first element is set to `value`; the second is zeroed.
	#[must_use]
	pub fn new_e0(value: f64) -> Self {
		Self(unsafe { _mm_set_sd(value) })
	}

	/// Copies the first element of `self` to the first and second elements of
	/// a new vector and returns it.
	#[must_use]
	pub fn splat_e0(self) -> Self {
		Self(unsafe { _mm_shuffle_pd::<{ mm_shuffle(0, 0, 0, 0) }>(self.0, self.0) })
	}

	/// Copies the second element of `self` to the first and second elements of
	/// a new vector and returns it.
	#[must_use]
	pub fn splat_e1(self) -> Self {
		Self(unsafe { _mm_shuffle_pd::<{ mm_shuffle(0, 0, 2, 0) }>(self.0, self.0) })
	}

	#[must_use]
	pub fn e0(self) -> f64 {
		unsafe { _mm_cvtsd_f64(self.0) }
	}

	#[must_use]
	pub fn e1(self) -> f64 {
		unsafe { _mm_cvtsd_f64(self.splat_e1().0) }
	}
}

impl PartialEq for F64X2 {
	fn eq(&self, other: &Self) -> bool {
		unsafe {
			// From glam
			let cmp = _mm_cmpeq_pd(self.0, other.0);
			(_mm_movemask_pd(cmp) as u32 & 0x7) == 0x7
		}
	}
}

impl std::ops::Add for F64X2 {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self(unsafe { _mm_add_pd(self.0, rhs.0) })
	}
}

impl std::ops::Sub for F64X2 {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self(unsafe { _mm_sub_pd(self.0, rhs.0) })
	}
}

impl std::ops::Mul for F64X2 {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		Self(unsafe { _mm_mul_pd(self.0, rhs.0) })
	}
}

impl std::ops::Div for F64X2 {
	type Output = Self;

	fn div(self, rhs: Self) -> Self::Output {
		Self(unsafe { _mm_div_pd(self.0, rhs.0) })
	}
}
