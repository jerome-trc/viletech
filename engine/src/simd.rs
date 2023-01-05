//! Abstractions over SSE/SSE2 SIMD registers and SSE4.1/SSE4.2/AVX intrinsics.
//!
//! To keep things simple, all these types can be treated as though they were arrays;
//! the idiosyncrasies of SIMD field ordering is handled internally. "Element 0"
//! and "first element" mean the same thing in these docs.
//!
//! Implementations herein are a mix of the SIMD implementations from [`glam`]
//! and those of Gaijin Entertainment's Dagor Engine 5, found [here]. See the
//! top-level `/ATTRIB.md` document for licensing information.
//!
//! [here]: https://github.com/GaijinEntertainment/daScript/blob/master/include/vecmath/dag_vecMath_pc_sse.h

mod f32x4;
mod f64x2;
mod i32x4;
mod i64x2;

pub use f32x4::*;
pub use f64x2::*;
pub use i32x4::*;
pub use i64x2::*;

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

// TODO:
// - Use `__vectorcall` calling convention when it stabilizes

/// Utility function for SSE SIMD operations.
/// [`core::arch::x86_64::_MM_SHUFFLE`] is unstable; use this in the meantime.
#[must_use]
pub(self) const fn mm_shuffle(e3: u32, e1: u32, e0: u32, e4: u32) -> i32 {
	((e3 << 6) | (e1 << 4) | (e0 << 2) | e4) as i32
}

// Everything below is from glam ///////////////////////////////////////////////

union UnionCast {
	u32x4: [u32; 4],
	f32x4: [f32; 4],
	m128: __m128,
}

pub(self) const fn m128_from_f32x4(f32x4: [f32; 4]) -> __m128 {
	unsafe { UnionCast { f32x4 }.m128 }
}

pub(self) const fn m128_from_u32x4(u32x4: [u32; 4]) -> __m128 {
	unsafe { UnionCast { u32x4 }.m128 }
}

const PS_INV_SIGN_MASK: __m128 = m128_from_u32x4([!0x8000_0000; 4]);
const PS_NO_FRACTION: __m128 = m128_from_f32x4([8388608.0; 4]);

#[inline]
pub(self) unsafe fn m128_floor(v: __m128) -> __m128 {
	// Based on https://github.com/microsoft/DirectXMath `XMVectorFloor`
	// To handle NAN, INF and numbers greater than 8388608, use masking
	let test = _mm_and_si128(_mm_castps_si128(v), _mm_castps_si128(PS_INV_SIGN_MASK));
	let test = _mm_cmplt_epi32(test, _mm_castps_si128(PS_NO_FRACTION));
	// Truncate
	let vint = _mm_cvttps_epi32(v);
	let result = _mm_cvtepi32_ps(vint);
	let larger = _mm_cmpgt_ps(result, v);
	// 0 -> 0, 0xffffffff -> -1.0f
	let larger = _mm_cvtepi32_ps(_mm_castps_si128(larger));
	let result = _mm_add_ps(result, larger);
	// All numbers less than 8388608 will use the round to int
	let result = _mm_and_ps(result, _mm_castsi128_ps(test));
	// All others, use the ORIGINAL value
	let test = _mm_andnot_si128(test, _mm_castps_si128(v));
	_mm_or_ps(result, _mm_castsi128_ps(test))
}
