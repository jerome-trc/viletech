//! The basis for the LithScript ABI.

// TODO:
// - Use `__vectorcall` calling convention when it stabilizes
#![allow(dead_code)] // - Remove this?

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use crate::{math::mm_shuffle, newtype};

newtype! {
	/// Implementation detail of LithScript's ABI. This type is only exposed to
	/// enable trait bounds on conversions.
	///
	/// A LithScript "word" is a vector of 4 `f32`s, preferably using a SIMD type.
	///
	/// This structure is used to hold anything that can pass between the langauge
	/// boundary; if it can't fit into 128 bits, it gets boxed/referenced. For the
	/// rationale as to this decision, see the [daScript Reference Manual, section 3.1].
	///
	/// Every type conversion implemented herein is considered "safe" in that the
	/// to and from types are both trivial, such as numbers and [`glam`] structures.
	/// For anything which has non-trivial construction/drop semantics, use
	/// [`Word::from_any`] and [`Word::into_any`], but there are no assurances
	/// that something coming out of a `Word` is remotely the same as what got put in.
	/// Take caution.
	///
	/// For now, systems that can't use SSE2's [`__m128`] are unsupported, although
	/// there is nothing stopping an alternate implementation using a plain struct.
	///
	/// [daScript Reference Manual, section 3.1]: https://dascript.org/doc/dascript.pdf
	#[derive(Debug, Clone, Copy)]
	pub struct Word(__m128)
}

impl Word {
	#[inline(always)]
	#[must_use]
	pub(super) fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
		Self(unsafe { _mm_setr_ps(x, y, z, w) })
	}

	#[inline(always)]
	#[must_use]
	pub(super) fn zeroed() -> Self {
		Self(unsafe { _mm_setzero_ps() })
	}

	/// Returns a `Word` with an X set to `value`, and all other components zeroed.
	#[inline(always)]
	#[must_use]
	pub(super) fn x_zero(value: f32) -> Self {
		Self(unsafe { _mm_set_ss(value) })
	}

	#[inline(always)]
	#[must_use]
	pub(super) unsafe fn into_any<T: Sized>(self) -> T {
		debug_assert!(
			std::mem::size_of::<T>() <= 16,
			"Attempted to transmute a Lith word into an oversized type."
		);

		std::mem::transmute_copy(&self)
	}

	#[inline(always)]
	#[must_use]
	pub(super) unsafe fn from_any<T: Sized>(value: T) -> Self {
		debug_assert!(
			std::mem::size_of_val(&value) <= 16,
			"Attempted to overfill a Lith word."
		);

		std::mem::transmute_copy(&value)
	}

	/// Copies the 1st lane of `self` to every lane in a new vector and returns it.
	#[inline(always)]
	#[must_use]
	pub(super) fn splat_x(self) -> Self {
		Self(unsafe { _mm_shuffle_ps::<{ mm_shuffle(0, 0, 0, 0) }>(self.0, self.0) })
	}

	/// Copies the 2nd lane of `self` to every lane in a new vector and returns it.
	#[inline(always)]
	#[must_use]
	pub(super) fn splat_y(self) -> Self {
		Self(unsafe { _mm_shuffle_ps::<{ mm_shuffle(1, 1, 1, 1) }>(self.0, self.0) })
	}

	/// Copies the 3rd lane of `self` to every lane in a new vector and returns it.
	#[inline(always)]
	#[must_use]
	pub(super) fn splat_z(self) -> Self {
		Self(unsafe { _mm_shuffle_ps::<{ mm_shuffle(2, 2, 2, 2) }>(self.0, self.0) })
	}

	/// Copies the 4th lane of `self` to every lane in a new vector and returns it.
	#[inline(always)]
	#[must_use]
	pub(super) fn splat_w(self) -> Self {
		Self(unsafe { _mm_shuffle_ps::<{ mm_shuffle(3, 3, 3, 3) }>(self.0, self.0) })
	}

	#[inline(always)]
	#[must_use]
	pub(super) fn x(self) -> f32 {
		unsafe { _mm_cvtss_f32(self.0) }
	}

	#[inline(always)]
	#[must_use]
	pub(super) fn y(self) -> f32 {
		unsafe { _mm_cvtss_f32(self.splat_y().0) }
	}

	#[inline(always)]
	#[must_use]
	pub(super) fn z(self) -> f32 {
		unsafe { _mm_cvtss_f32(self.splat_z().0) }
	}

	#[inline(always)]
	#[must_use]
	pub(super) fn w(self) -> f32 {
		unsafe { _mm_cvtss_f32(self.splat_w().0) }
	}
}

// Conversions: miscellaneous //////////////////////////////////////////////////

impl From<Word> for () {
	#[inline(always)]
	fn from(_: Word) -> Self {}
}

impl From<()> for Word {
	#[inline(always)]
	fn from(_: ()) -> Self {
		Self::zeroed()
	}
}

impl From<Word> for bool {
	fn from(value: Word) -> Self {
		Into::<Vec4I>::into(value).x() != 0
	}
}

impl From<bool> for Word {
	fn from(value: bool) -> Self {
		Vec4I::x_zero(value as i32).into()
	}
}

#[repr(C)]
union CharToWord {
	word: Word,
	character: char,
}

impl From<Word> for char {
	#[inline(always)]
	fn from(value: Word) -> Self {
		let ctw = CharToWord { word: value };
		unsafe { ctw.character }
	}
}

impl From<char> for Word {
	#[inline(always)]
	fn from(value: char) -> Self {
		let ctw = CharToWord { character: value };
		unsafe { ctw.word }
	}
}

impl From<Word> for Vec4I {
	#[inline(always)]
	fn from(value: Word) -> Self {
		Self(unsafe { _mm_castps_si128(value.0) })
	}
}

impl From<Vec4I> for Word {
	#[inline(always)]
	fn from(value: Vec4I) -> Self {
		Self(unsafe { _mm_castsi128_ps(value.0) })
	}
}

impl<T> From<Word> for (T,)
where
	T: Into<Word> + From<Word>,
{
	#[inline(always)]
	fn from(value: Word) -> Self {
		value.into()
	}
}

impl<T> From<(T,)> for Word
where
	T: Into<Word> + From<Word>,
{
	#[inline(always)]
	fn from(value: (T,)) -> Self {
		value.into()
	}
}

// Conversions: integral ///////////////////////////////////////////////////////

macro_rules! int_converters {
	($($int_t:ty),+) => {
		$(
			impl From<Word> for $int_t {
				#[inline(always)]
				fn from(value: Word) -> Self {
					Into::<Vec4I>::into(value).x() as Self
				}
			}

			impl From<$int_t> for Word {
				#[inline(always)]
				fn from(value: $int_t) -> Self {
					Vec4I::x_zero(value as i32).into()
				}
			}
		)+
	};
}

int_converters!(i8, u8, i16, u16, i32, u32);

impl From<Word> for i64 {
	#[inline(always)]
	fn from(value: Word) -> Self {
		Into::<Vec4I>::into(value).x64()
	}
}

impl From<i64> for Word {
	#[inline(always)]
	fn from(value: i64) -> Self {
		Vec4I::splat_half(value).into()
	}
}

impl From<Word> for u64 {
	#[inline(always)]
	fn from(value: Word) -> Self {
		Into::<Vec4I>::into(value).x64u()
	}
}

impl From<u64> for Word {
	#[inline(always)]
	fn from(value: u64) -> Self {
		Vec4I::splat_half_u(value).into()
	}
}

impl From<Word> for isize {
	#[inline(always)]
	fn from(value: Word) -> Self {
		#[cfg(target_pointer_width = "64")]
		let i: i64 = value.into();
		#[cfg(target_pointer_width = "32")]
		let i: i32 = value.into();

		i as isize
	}
}

impl From<isize> for Word {
	#[inline(always)]
	fn from(value: isize) -> Self {
		#[cfg(target_pointer_width = "64")]
		let v = value as i64;
		#[cfg(target_pointer_width = "32")]
		let v = value as i32;

		v.into()
	}
}

impl From<Word> for usize {
	#[inline(always)]
	fn from(value: Word) -> Self {
		#[cfg(target_pointer_width = "64")]
		let i: u64 = value.into();
		#[cfg(target_pointer_width = "32")]
		let i: u32 = value.into();

		i as usize
	}
}

impl From<usize> for Word {
	#[inline(always)]
	fn from(value: usize) -> Self {
		#[cfg(target_pointer_width = "64")]
		let v = value as u64;
		#[cfg(target_pointer_width = "32")]
		let v = value as u32;

		v.into()
	}
}

// Conversions: floating-point /////////////////////////////////////////////////

impl From<Word> for f32 {
	#[inline(always)]
	fn from(value: Word) -> Self {
		value.x()
	}
}

impl From<f32> for Word {
	#[inline(always)]
	fn from(value: f32) -> Self {
		Self::x_zero(value)
	}
}

impl From<Word> for f64 {
	#[inline(always)]
	fn from(value: Word) -> Self {
		unsafe { _mm_cvtsd_f64(_mm_cvtps_pd(value.0)) }
	}
}

impl From<f64> for Word {
	#[inline(always)]
	fn from(value: f64) -> Self {
		unsafe { Self(_mm_castpd_ps(_mm_set_sd(value))) }
	}
}

// Conversions: pointers ///////////////////////////////////////////////////////

impl<T> From<Word> for *const T {
	#[inline(always)]
	fn from(value: Word) -> Self {
		usize::from(value) as *const T
	}
}

impl<T> From<*const T> for Word {
	#[inline(always)]
	fn from(value: *const T) -> Self {
		(value as usize).into()
	}
}

impl<T> From<Word> for *mut T {
	#[inline(always)]
	fn from(value: Word) -> Self {
		usize::from(value) as *mut T
	}
}

impl<T> From<*mut T> for Word {
	#[inline(always)]
	fn from(value: *mut T) -> Self {
		(value as usize).into()
	}
}

// Vec4I ///////////////////////////////////////////////////////////////////////

newtype! {
	/// Implementation detail of `Word`.
	///
	/// Essentially just a friendlier type alias for `Word`'s integral sibling,
	/// with some methods attached for convenience.
	#[derive(Clone, Copy)]
	struct Vec4I(pub(self) __m128i)
}

impl Vec4I {
	#[inline(always)]
	#[must_use]
	fn _zero() -> Self {
		Self(unsafe { _mm_setzero_si128() })
	}

	#[inline(always)]
	#[must_use]
	fn _splat(value: i32) -> Self {
		Self(unsafe { _mm_set1_epi32(value) })
	}

	/// The first half of the returned vector gets filled with the given value.
	/// The rest is uninitialized.
	#[inline(always)]
	#[must_use]
	fn splat_half(value: i64) -> Self {
		let addr = std::ptr::addr_of!(value) as *const __m128i;
		Self(unsafe { _mm_loadl_epi64(addr) })
	}

	/// The first half of the returned vector gets filled with the given value.
	/// The rest is uninitialized.
	#[inline(always)]
	#[must_use]
	fn splat_half_u(value: u64) -> Self {
		let addr = std::ptr::addr_of!(value) as *const __m128i;
		Self(unsafe { _mm_loadl_epi64(addr) })
	}

	#[inline(always)]
	#[must_use]
	fn x_zero(value: i32) -> Self {
		Self(unsafe { _mm_cvtsi32_si128(value) })
	}

	#[inline(always)]
	#[must_use]
	fn x(self) -> i32 {
		unsafe { _mm_cvtsi128_si32(self.0) }
	}
}

#[cfg(not(target_feature = "sse4.1"))]
impl Vec4I {
	#[inline(always)]
	#[must_use]
	fn y(self) -> i32 {
		unsafe { _mm_cvtsi128_si32(_mm_shuffle_epi32::<{ mm_shuffle(1, 1, 1, 1) }>(self.0)) }
	}

	#[inline(always)]
	#[must_use]
	fn z(self) -> i32 {
		unsafe { _mm_cvtsi128_si32(_mm_shuffle_epi32::<{ mm_shuffle(2, 2, 2, 2) }>(self.0)) }
	}

	#[inline(always)]
	#[must_use]
	fn w(self) -> i32 {
		unsafe { _mm_cvtsi128_si32(_mm_shuffle_epi32::<{ mm_shuffle(3, 3, 3, 3) }>(self.0)) }
	}

	#[inline(always)]
	#[must_use]
	fn x64(self) -> i64 {
		let mut ret = 0;
		unsafe {
			_mm_storel_epi64(std::ptr::addr_of_mut!(ret) as *mut __m128i, self.0);
		}
		ret
	}

	#[inline(always)]
	#[must_use]
	fn x64u(self) -> u64 {
		let mut ret = 0;
		unsafe {
			_mm_storel_epi64(std::ptr::addr_of_mut!(ret) as *mut __m128i, self.0);
		}
		ret
	}
}

#[cfg(target_feature = "sse4.1")]
impl Vec4I {
	#[inline(always)]
	#[must_use]
	fn y(self) -> i32 {
		unsafe { _mm_extract_epi32(self.0, 1) }
	}

	#[inline(always)]
	#[must_use]
	fn z(self) -> i32 {
		unsafe { _mm_extract_epi32(self.0, 2) }
	}

	#[inline(always)]
	#[must_use]
	fn z(self) -> i32 {
		unsafe { _mm_extract_epi32(self.0, 3) }
	}

	#[inline(always)]
	#[must_use]
	fn x64(self) -> i64 {
		unsafe { _mm_extract_epi64(self.0, 0) }
	}
}
