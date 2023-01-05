//! The basis for the LithScript ABI.

use std::ops::Range;

use crate::simd::{F32X4, F64X2, I32X4, I64X2};

/// The principal unit of LithScript's ABI.
/// Wraps either an [`__m128`], an [`__m128d`], or an [`__m128i`].
/// Only exposed to enable type conversions.
///
/// This structure is used to hold anything that can pass between the langauge
/// boundary; if it can't fit into 128 bits, it gets boxed/referenced. For the
/// rationale as to this decision, see the [daScript Reference Manual], section 3.1.
///
/// Every type conversion implemented herein is considered "safe" in that the
/// to and from types are both trivial, such as numbers and [`glam`] structures.
///
/// For now, systems that can't use SSE2 are unsupported, although there is nothing
/// stopping an alternative implementation with a plain byte array underneath.
///
/// [daScript Reference Manual]: https://dascript.org/doc/dascript.pdf
#[derive(Clone, Copy)]
pub union Word {
	pub(crate) f32x4: F32X4,
	pub(crate) i32x4: I32X4<i32>,
	pub(crate) u32x4: I32X4<u32>,
	pub(crate) f64x2: F64X2,
	pub(crate) i64x2: I64X2<i64>,
	pub(crate) u64x2: I64X2<u64>,
}

impl Word {
	#[inline(always)]
	#[must_use]
	pub(crate) fn zeroed() -> Self {
		Self {
			f32x4: F32X4::zeroed(),
		}
	}
}

impl std::fmt::Debug for Word {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		unsafe {
			f.debug_struct("Word")
				.field("i32x4[0]", &self.i32x4.e0())
				.field("i32x4[1]", &self.i32x4.e1())
				.field("i32x4[2]", &self.i32x4.e2())
				.field("i32x4[3]", &self.i32x4.e3())
				.finish()
		}
	}
}

// Conversions: miscellaneous //////////////////////////////////////////////////

macro_rules! simd_wrapper_conv {
	($t:ty, $field:ident) => {
		impl From<Word> for $t {
			#[inline(always)]
			fn from(value: Word) -> Self {
				unsafe { value.$field }
			}
		}

		impl From<$t> for Word {
			#[inline(always)]
			fn from(value: $t) -> Self {
				Self { $field: value }
			}
		}
	};
}

simd_wrapper_conv!(F32X4, f32x4);
simd_wrapper_conv!(I32X4<i32>, i32x4);
simd_wrapper_conv!(I32X4<u32>, u32x4);
simd_wrapper_conv!(F64X2, f64x2);
simd_wrapper_conv!(I64X2<i64>, i64x2);
simd_wrapper_conv!(I64X2<u64>, u64x2);

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
	#[inline(always)]
	fn from(value: Word) -> Self {
		unsafe { value.u32x4.e0() != 0 }
	}
}

impl From<bool> for Word {
	#[inline(always)]
	fn from(value: bool) -> Self {
		Self {
			u32x4: I32X4::new_e0(value as u32),
		}
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
					unsafe {
						value.i32x4.e0() as Self
					}
				}
			}

			impl From<$int_t> for Word {
				#[inline(always)]
				fn from(value: $int_t) -> Self {
					I32X4::new_e0(value).into()
				}
			}
		)+
	};
}

// int_converters!(i8, u8, i16, u16, i32, u32);
int_converters!(i32, u32);

impl From<Word> for i64 {
	#[inline(always)]
	fn from(value: Word) -> Self {
		unsafe { value.i64x2.e0() }
	}
}

impl From<i64> for Word {
	#[inline(always)]
	fn from(value: i64) -> Self {
		Self {
			i64x2: I64X2::new_e0(value),
		}
	}
}

impl From<Word> for u64 {
	#[inline(always)]
	fn from(value: Word) -> Self {
		unsafe { value.u64x2.e0() }
	}
}

impl From<u64> for Word {
	#[inline(always)]
	fn from(value: u64) -> Self {
		Self {
			u64x2: I64X2::new_e0(value),
		}
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

impl From<Word> for i128 {
	#[inline(always)]
	fn from(value: Word) -> Self {
		unsafe { std::mem::transmute(value) }
	}
}

impl From<i128> for Word {
	#[inline(always)]
	fn from(value: i128) -> Self {
		unsafe { std::mem::transmute(value) }
	}
}

impl From<Word> for u128 {
	#[inline(always)]
	fn from(value: Word) -> Self {
		unsafe { std::mem::transmute(value) }
	}
}

impl From<u128> for Word {
	#[inline(always)]
	fn from(value: u128) -> Self {
		unsafe { std::mem::transmute(value) }
	}
}

// Conversions: floating-point /////////////////////////////////////////////////

impl From<Word> for f32 {
	#[inline(always)]
	fn from(value: Word) -> Self {
		unsafe { value.f32x4.e0() }
	}
}

impl From<f32> for Word {
	#[inline(always)]
	fn from(value: f32) -> Self {
		Self {
			f32x4: F32X4::new_e0(value),
		}
	}
}

impl From<Word> for f64 {
	#[inline(always)]
	fn from(value: Word) -> Self {
		unsafe { value.f64x2.e0() }
	}
}

impl From<f64> for Word {
	#[inline(always)]
	fn from(value: f64) -> Self {
		Self {
			f64x2: F64X2::new_e0(value),
		}
	}
}

// Conversions: ranges /////////////////////////////////////////////////////////

impl From<Word> for Range<i32> {
	#[inline(always)]
	fn from(value: Word) -> Self {
		unsafe { value.i32x4.e0()..value.i32x4.e1() }
	}
}

impl From<Range<i32>> for Word {
	#[inline(always)]
	fn from(value: Range<i32>) -> Self {
		I32X4::new(value.start, value.end, 0, 0).into()
	}
}

impl From<Word> for Range<u32> {
	#[inline(always)]
	fn from(value: Word) -> Self {
		unsafe { value.u32x4.e0()..value.u32x4.e1() }
	}
}

impl From<Range<u32>> for Word {
	#[inline(always)]
	fn from(value: Range<u32>) -> Self {
		I32X4::new(value.start, value.end, 0, 0).into()
	}
}

impl From<Word> for Range<i64> {
	#[inline(always)]
	fn from(value: Word) -> Self {
		unsafe { value.i64x2.e0()..value.i64x2.e1() }
	}
}

impl From<Range<i64>> for Word {
	#[inline(always)]
	fn from(value: Range<i64>) -> Self {
		I64X2::new(value.start, value.end).into()
	}
}

impl From<Word> for Range<u64> {
	#[inline(always)]
	fn from(value: Word) -> Self {
		unsafe { value.u64x2.e0()..value.u64x2.e1() }
	}
}

impl From<Range<u64>> for Word {
	#[inline(always)]
	fn from(value: Range<u64>) -> Self {
		I64X2::new(value.start, value.end).into()
	}
}

impl From<Word> for Range<isize> {
	#[inline(always)]
	fn from(value: Word) -> Self {
		#[cfg(target_pointer_width = "64")]
		unsafe {
			(value.i64x2.e0() as isize)..(value.i64x2.e1() as isize)
		}

		#[cfg(target_pointer_width = "32")]
		unsafe {
			(value.i32x4.e0() as isize)..(value.i32x4.e1() as isize)
		}
	}
}

impl From<Range<isize>> for Word {
	#[inline(always)]
	fn from(value: Range<isize>) -> Self {
		#[cfg(target_pointer_width = "64")]
		{
			I64X2::new(value.start as i64, value.end as i64).into()
		}

		#[cfg(target_pointer_width = "32")]
		{
			I32X4::new(value.start as i32, value.end as i32, 0, 0).into()
		}
	}
}

impl From<Word> for Range<usize> {
	#[inline(always)]
	fn from(value: Word) -> Self {
		#[cfg(target_pointer_width = "64")]
		unsafe {
			(value.u64x2.e0() as usize)..(value.u64x2.e1() as usize)
		}

		#[cfg(target_pointer_width = "32")]
		unsafe {
			(value.u32x4.e0() as usize)..(value.u32x4.e1() as usize)
		}
	}
}

impl From<Range<usize>> for Word {
	#[inline(always)]
	fn from(value: Range<usize>) -> Self {
		#[cfg(target_pointer_width = "64")]
		{
			I64X2::new(value.start as u64, value.end as u64).into()
		}

		#[cfg(target_pointer_width = "32")]
		{
			I32X4::new(value.start as u32, value.end as u32, 0, 0).into()
		}
	}
}

impl From<Word> for Range<f32> {
	#[inline(always)]
	fn from(value: Word) -> Self {
		unsafe { value.f32x4.e0()..value.f32x4.e1() }
	}
}

impl From<Range<f32>> for Word {
	#[inline(always)]
	fn from(value: Range<f32>) -> Self {
		F32X4::new(value.start, value.end, 0.0, 0.0).into()
	}
}

impl From<Word> for Range<f64> {
	#[inline(always)]
	fn from(value: Word) -> Self {
		unsafe { value.f64x2.e0()..value.f64x2.e1() }
	}
}

impl From<Range<f64>> for Word {
	#[inline(always)]
	fn from(value: Range<f64>) -> Self {
		F64X2::new(value.start, value.end).into()
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

// Conversions: glam vectors ///////////////////////////////////////////////////

macro_rules! glam_transmutes {
	($($glam_t:ty) +) => {
		$(
			#[cfg(target_feature = "sse2")]
			impl From<Word> for $glam_t {
				#[inline(always)]
				fn from(value: Word) -> Self {
					// Safety: if using SSE2 and not feature `glam/scalar-math`,
					// these two types have identical representation
					unsafe { std::mem::transmute(value) }
				}
			}

			#[cfg(target_feature = "sse2")]
			impl From<$glam_t> for Word {
				#[inline(always)]
				fn from(value: $glam_t) -> Self {
					// Safety: if using SSE2 and not feature `glam/scalar-math`,
					// these two types have identical representation
					unsafe { std::mem::transmute(value) }
				}
			}
		)+
	};
}

glam_transmutes! { glam::Vec3A glam::Vec4 glam::Quat glam::Mat2 }

impl From<Word> for glam::Vec2 {
	#[inline(always)]
	fn from(value: Word) -> Self {
		unsafe { Self::from_array([value.f32x4.e0(), value.f32x4.e1()]) }
	}
}

impl From<glam::Vec2> for Word {
	#[inline(always)]
	fn from(value: glam::Vec2) -> Self {
		let arr = value.to_array();
		F32X4::new(arr[0], arr[1], 0.0, 0.0).into()
	}
}

impl From<Word> for glam::Vec3 {
	#[inline(always)]
	fn from(value: Word) -> Self {
		unsafe { Self::from_array([value.f32x4.e0(), value.f32x4.e1(), value.f32x4.e2()]) }
	}
}

impl From<glam::Vec3> for Word {
	#[inline(always)]
	fn from(value: glam::Vec3) -> Self {
		let arr = value.to_array();
		F32X4::new(arr[0], arr[1], arr[2], 0.0).into()
	}
}

#[cfg(not(target_feature = "sse2"))]
impl From<Word> for glam::Vec3A {
	#[inline(always)]
	fn from(value: Word) -> Self {
		Self::from_array([value.f32x4.e0(), value.f32x4.e0(), value.f32x4.e0()])
	}
}

#[cfg(not(target_feature = "sse2"))]
impl From<glam::Vec3A> for Word {
	#[inline(always)]
	fn from(value: glam::Vec3A) -> Self {
		let arr = value.to_array();
		F32X4::new(arr[0], arr[1], arr[2], 0.0).into()
	}
}

#[cfg(not(target_feature = "sse2"))]
impl From<Word> for glam::Vec4 {
	#[inline(always)]
	fn from(value: Word) -> Self {
		unsafe {
			Self::from_array([
				value.f32x4.e0(),
				value.f32x4.e1(),
				value.f32x4.e2(),
				value.f32x4.e3(),
			])
		}
	}
}

#[cfg(not(target_feature = "sse2"))]
impl From<glam::Vec4> for Word {
	#[inline(always)]
	fn from(value: glam::Vec4) -> Self {
		let arr = value.to_array();
		F32X4::new(arr[0], arr[1], arr[2], arr[3]).into()
	}
}
