//! Four packed 32-bit floating point numbers in an [`__m128`].

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::{m128_floor, mm_shuffle, Int32, I32X4};

/// An `__m128` used for holding 4 32-bit floating-point numbers.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct F32X4(pub(super) __m128);

impl F32X4 {
    /// `e0` a.k.a. `x` a.k.a. `r`; `e1` a.k.a. `y` a.k.a. `g`;
    /// `e2` a.k.a. `z` a.k.a. `b`; `e3` a.k.a. `w` a.k.a. `a`
    #[must_use]
    pub fn new(e0: f32, e1: f32, e2: f32, e3: f32) -> Self {
        Self(unsafe { _mm_setr_ps(e0, e1, e2, e3) })
    }

    #[must_use]
    pub fn zeroed() -> Self {
        Self(unsafe { _mm_setzero_ps() })
    }

    /// The first element is set to `value`, and all other components are zeroed.
    #[must_use]
    pub fn new_e0(value: f32) -> Self {
        Self(unsafe { _mm_set_ss(value) })
    }

    /// All elements in the returned vector are set to `value`.
    pub fn splat(value: f32) -> Self {
        Self(unsafe { _mm_set1_ps(value) })
    }

    /// Copies the 1st element of `self` to every element in a new vector and returns it.
    #[must_use]
    pub fn splat_e0(self) -> Self {
        Self(unsafe { _mm_shuffle_ps::<{ mm_shuffle(0, 0, 0, 0) }>(self.0, self.0) })
    }

    /// Copies the 2nd element of `self` to every element in a new vector and returns it.
    #[must_use]
    pub fn splat_e1(self) -> Self {
        Self(unsafe { _mm_shuffle_ps::<{ mm_shuffle(1, 1, 1, 1) }>(self.0, self.0) })
    }

    /// Copies the 3rd element of `self` to every element in a new vector and returns it.
    #[must_use]
    pub fn splat_e2(self) -> Self {
        Self(unsafe { _mm_shuffle_ps::<{ mm_shuffle(2, 2, 2, 2) }>(self.0, self.0) })
    }

    /// Copies the 4th element of `self` to every element in a new vector and returns it.
    #[must_use]
    pub fn splat_e3(self) -> Self {
        Self(unsafe { _mm_shuffle_ps::<{ mm_shuffle(3, 3, 3, 3) }>(self.0, self.0) })
    }

    #[must_use]
    pub fn as_tuple(self) -> (f32, f32, f32, f32) {
        (self.e0(), self.e1(), self.e2(), self.e3())
    }

    #[must_use]
    pub fn as_array(self) -> [f32; 4] {
        [self.e0(), self.e1(), self.e2(), self.e3()]
    }

    #[must_use]
    pub fn e0(self) -> f32 {
        unsafe { _mm_cvtss_f32(self.0) }
    }

    #[must_use]
    pub fn e1(self) -> f32 {
        unsafe { _mm_cvtss_f32(self.splat_e1().0) }
    }

    #[must_use]
    pub fn e2(self) -> f32 {
        unsafe { _mm_cvtss_f32(self.splat_e2().0) }
    }

    #[must_use]
    pub fn e3(self) -> f32 {
        unsafe { _mm_cvtss_f32(self.splat_e3().0) }
    }
}

impl<T: Int32> From<I32X4<T>> for F32X4 {
    fn from(value: I32X4<T>) -> Self {
        Self(unsafe { _mm_castsi128_ps(value.0) })
    }
}

impl PartialEq for F32X4 {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            // From glam
            let cmp = _mm_cmpeq_ps(self.0, other.0);
            (_mm_movemask_ps(cmp) as u32 & 0x7) == 0x7
        }
    }
}

impl std::ops::Add for F32X4 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(unsafe { _mm_add_ps(self.0, rhs.0) })
    }
}

impl std::ops::Sub for F32X4 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(unsafe { _mm_sub_ps(self.0, rhs.0) })
    }
}

impl std::ops::Mul for F32X4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(unsafe { _mm_mul_ps(self.0, rhs.0) })
    }
}

impl std::ops::Div for F32X4 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(unsafe { _mm_div_ps(self.0, rhs.0) })
    }
}

impl std::ops::Rem for F32X4 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        // From glam
        unsafe {
            let n = m128_floor(_mm_div_ps(self.0, rhs.0));
            Self(_mm_sub_ps(self.0, _mm_mul_ps(n, rhs.0)))
        }
    }
}
