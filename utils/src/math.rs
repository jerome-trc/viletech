use std::ops::{Add, AddAssign, Mul};

use glam::{DQuat, DVec3, EulerRot, Quat, Vec3A};

pub type Fixed32 = fixed::types::I16F16;
pub type Fixed64 = fixed::types::I32F32;

pub trait Dimension: Sized + Copy + Add<Output = Self> + AddAssign + Mul<Output = Self> {}

impl<T> Dimension for T where T: Sized + Copy + Add<Output = Self> + AddAssign + Mul<Output = Self> {}

/// A generic rectangle represented with a top-left, width, and height.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Rect4<T>
where
	T: Dimension,
{
	left: T,
	top: T,
	width: T,
	height: T,
}

impl<T> Rect4<T>
where
	T: Dimension,
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

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MinMaxBox {
	pub min: glam::Vec3A,
	pub max: glam::Vec3A,
}

/// A strongly-typed angle in degrees of type `T`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Angle<T: Dimension>(T);

pub type Angle32 = Angle<f32>;
pub type Angle64 = Angle<f64>;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rotator32(Vec3A);

impl Rotator32 {
	#[must_use]
	pub fn new(pitch: f32, yaw: f32, roll: f32) -> Self {
		Self(Vec3A::new(pitch, yaw, roll))
	}

	#[must_use]
	pub fn pitch(&self) -> f32 {
		self.0[0]
	}

	#[must_use]
	pub fn yaw(&self) -> f32 {
		self.0[1]
	}

	#[must_use]
	pub fn roll(&self) -> f32 {
		self.0[2]
	}

	pub fn set_pitch(&mut self, pitch: f32) {
		self.0[0] = pitch
	}

	pub fn set_yaw(&mut self, yaw: f32) {
		self.0[1] = yaw
	}

	pub fn set_roll(&mut self, roll: f32) {
		self.0[2] = roll
	}

	#[must_use]
	pub fn to_quat(self) -> Quat {
		Quat::from_euler(EulerRot::XZY, self.pitch(), self.yaw(), self.roll())
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rotator64(DVec3);

// (RAT) Q: Would this type benefit from SIMD? using `__m256d` would depend on
// AVX2 intrinsics that aren't even supported on my current machine.

impl Rotator64 {
	#[must_use]
	pub fn new(pitch: f64, yaw: f64, roll: f64) -> Self {
		Self(DVec3::new(pitch, yaw, roll))
	}

	/// Rotation around the X axis.
	#[must_use]
	pub fn pitch(&self) -> f64 {
		self.0[0]
	}

	/// Rotation around the Z axis.
	#[must_use]
	pub fn yaw(&self) -> f64 {
		self.0[1]
	}

	/// Rotation around the Y axis.
	#[must_use]
	pub fn roll(&self) -> f64 {
		self.0[2]
	}

	pub fn set_pitch(&mut self, pitch: f64) {
		self.0[0] = pitch
	}

	pub fn set_yaw(&mut self, yaw: f64) {
		self.0[1] = yaw
	}

	pub fn set_roll(&mut self, roll: f64) {
		self.0[2] = roll
	}

	#[must_use]
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

pub type UAngle = u32;

#[must_use]
pub fn point_to_angle(x: Fixed32, y: Fixed32) -> UAngle {
	let ang = f64::atan2(x.to_num(), y.to_num());
	let rad2bam = ((1 << 30) as f64) / std::f64::consts::PI;
	let dbam = ang * rad2bam;
	(dbam as u32) << 1
}

pub const UANGLE_180: UAngle = 1 << 31;

/// "Fixed-point multiply" with right-shift scaling.
#[must_use]
pub fn fxmul<const SHIFT: i64>(a: Fixed32, b: Fixed32) -> Fixed32 {
	let a64 = a.to_bits() as i64;
	let b64 = b.to_bits() as i64;
	Fixed32::from_bits(((a64 * b64) >> SHIFT) as i32)
}

/// "Fixed-point divide" with left-shift scaling.
#[must_use]
pub fn fxdiv<const SHIFT: i64>(a: Fixed32, b: Fixed32) -> Fixed32 {
	let a64 = a.to_bits() as f64;
	let b64 = b.to_bits() as f64;
	let ret = a64 / b64 * (1 << 30) as f64;
	Fixed32::from_bits(ret as i32)
}

/// "Fixed-point double-precision multiply".
pub fn fxdmul(a: Fixed32, b: Fixed32, c: Fixed32, d: Fixed32) -> Fixed32 {
	let a64 = a.to_bits() as f64;
	let b64 = b.to_bits() as f64;
	let c64 = c.to_bits() as f64;
	let d64 = d.to_bits() as f64;
	Fixed32::from_bits(((a64 * b64 + c64 * d64) / 4294967296.0) as i32)
}

#[cfg(test)]
#[test]
fn fixed_ops() {
	let a = Fixed32::from_num(3104);
	let b = Fixed32::from_num(-4864);
	assert_eq!(fxmul::<30>(a, b), Fixed32::from_num(-921.5));
	assert_eq!(fxdiv::<30>(a, b), Fixed32::from_num(-10455.57893));

	let c = Fixed32::from_num(64);
	let d = Fixed32::from_num(-2816);
	assert_eq!(fxdmul(a, b, c, d), Fixed32::from_num(-233.1250));
}
