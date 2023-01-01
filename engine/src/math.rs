use std::ops::{AddAssign, DivAssign, MulAssign, Rem, RemAssign, SubAssign};

use glam::{DQuat, DVec3, EulerRot, Quat, Vec3A};

pub trait Numeric:
	Sized + Copy + num::Num + AddAssign + MulAssign + DivAssign + SubAssign + Rem + RemAssign
{
}

impl<T> Numeric for T where
	T: Sized + Copy + num::Num + AddAssign + MulAssign + DivAssign + SubAssign + Rem + RemAssign
{
}

/// Utility function for SSE SIMD operations.
/// [`core::arch::x86_64::_MM_SHUFFLE`] is unstable; use this in the meantime.
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

#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Rotator32(Vec3A);

impl Rotator32 {
	#[must_use]
	pub fn new(pitch: f32, yaw: f32, roll: f32) -> Self {
		Self(Vec3A::new(pitch, yaw, roll))
	}

	#[must_use]
	#[inline(always)]
	pub fn pitch(&self) -> f32 {
		self.0[0]
	}

	#[must_use]
	#[inline(always)]
	pub fn yaw(&self) -> f32 {
		self.0[1]
	}

	#[must_use]
	#[inline(always)]
	pub fn roll(&self) -> f32 {
		self.0[2]
	}

	#[inline(always)]
	pub fn set_pitch(&mut self, pitch: f32) {
		self.0[0] = pitch
	}

	#[inline(always)]
	pub fn set_yaw(&mut self, yaw: f32) {
		self.0[1] = yaw
	}

	#[inline(always)]
	pub fn set_roll(&mut self, roll: f32) {
		self.0[2] = roll
	}

	#[must_use]
	#[inline(always)]
	pub fn to_quat(self) -> Quat {
		Quat::from_euler(EulerRot::XZY, self.pitch(), self.yaw(), self.roll())
	}
}

#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Rotator64(DVec3);

// [Rat] Q: Would this type benefit from SIMD? using `__m256d` would depend on
// AVX2 intrinsics that aren't even supported on my current machine.

impl Rotator64 {
	#[must_use]
	pub fn new(pitch: f64, yaw: f64, roll: f64) -> Self {
		Self(DVec3::new(pitch, yaw, roll))
	}

	/// Rotation around the X axis.
	#[must_use]
	#[inline(always)]
	pub fn pitch(&self) -> f64 {
		self.0[0]
	}

	/// Rotation around the Z axis.
	#[must_use]
	#[inline(always)]
	pub fn yaw(&self) -> f64 {
		self.0[1]
	}

	/// Rotation around the Y axis.
	#[must_use]
	#[inline(always)]
	pub fn roll(&self) -> f64 {
		self.0[2]
	}

	#[inline(always)]
	pub fn set_pitch(&mut self, pitch: f64) {
		self.0[0] = pitch
	}

	#[inline(always)]
	pub fn set_yaw(&mut self, yaw: f64) {
		self.0[1] = yaw
	}

	#[inline(always)]
	pub fn set_roll(&mut self, roll: f64) {
		self.0[2] = roll
	}

	#[must_use]
	#[inline(always)]
	pub fn to_dquat(self) -> DQuat {
		DQuat::from_euler(EulerRot::XZY, self.pitch(), self.yaw(), self.roll())
	}
}

macro_rules! rotator_ops {
	($rot_t:ty, $elem_t:ty, $quat_t:ty) => {
		impl std::ops::Neg for $rot_t {
			type Output = Self;

			fn neg(self) -> Self {
				Self(self.0.neg())
			}
		}

		impl std::ops::Add<$rot_t> for $rot_t {
			type Output = Self;

			fn add(self, rhs: Self) -> Self::Output {
				Self(self.0 + rhs.0)
			}
		}

		impl std::ops::Add<$elem_t> for $rot_t {
			type Output = Self;

			fn add(self, rhs: $elem_t) -> Self::Output {
				Self(self.0 + rhs)
			}
		}

		impl std::ops::Add<$rot_t> for $elem_t {
			type Output = $rot_t;

			fn add(self, rhs: $rot_t) -> Self::Output {
				rhs + self
			}
		}

		impl std::ops::Sub<$rot_t> for $rot_t {
			type Output = Self;

			fn sub(self, rhs: Self) -> Self::Output {
				Self(self.0 - rhs.0)
			}
		}

		impl std::ops::Sub<$elem_t> for $rot_t {
			type Output = Self;

			fn sub(self, rhs: $elem_t) -> Self::Output {
				Self(self.0 - rhs)
			}
		}

		impl std::ops::Sub<$rot_t> for $elem_t {
			type Output = $rot_t;

			fn sub(self, rhs: $rot_t) -> Self::Output {
				rhs - self
			}
		}

		impl std::ops::Mul<$rot_t> for $rot_t {
			type Output = Self;

			fn mul(self, rhs: Self) -> Self::Output {
				Self(self.0 * rhs.0)
			}
		}

		impl std::ops::Mul<$elem_t> for $rot_t {
			type Output = Self;

			fn mul(self, rhs: $elem_t) -> Self::Output {
				Self(self.0 * rhs)
			}
		}

		impl std::ops::Mul<$rot_t> for $elem_t {
			type Output = $rot_t;

			fn mul(self, rhs: $rot_t) -> Self::Output {
				rhs * self
			}
		}

		impl std::ops::AddAssign<$rot_t> for $rot_t {
			fn add_assign(&mut self, rhs: Self) {
				self.0 += rhs.0
			}
		}

		impl std::ops::AddAssign<$elem_t> for $rot_t {
			fn add_assign(&mut self, rhs: $elem_t) {
				self.0 += rhs
			}
		}

		impl std::ops::SubAssign<$rot_t> for $rot_t {
			fn sub_assign(&mut self, rhs: Self) {
				self.0 -= rhs.0
			}
		}

		impl std::ops::SubAssign<$elem_t> for $rot_t {
			fn sub_assign(&mut self, rhs: $elem_t) {
				self.0 -= rhs
			}
		}

		impl std::ops::MulAssign<$rot_t> for $rot_t {
			fn mul_assign(&mut self, rhs: Self) {
				self.0 *= rhs.0
			}
		}

		impl std::ops::MulAssign<$elem_t> for $rot_t {
			fn mul_assign(&mut self, rhs: $elem_t) {
				self.0 *= rhs
			}
		}

		impl std::ops::DivAssign<$rot_t> for $rot_t {
			fn div_assign(&mut self, rhs: Self) {
				self.0 /= rhs.0
			}
		}

		impl std::ops::DivAssign<$elem_t> for $rot_t {
			fn div_assign(&mut self, rhs: $elem_t) {
				self.0 /= rhs
			}
		}
	};
}

rotator_ops!(Rotator32, f32, Quat);
rotator_ops!(Rotator64, f64, DQuat);
