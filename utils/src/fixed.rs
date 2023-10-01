//! Fixed-point number types.

/// A fixed-point number type with 16 integral bits and 16 fractional bits.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct I16F16(i32);

impl From<i32> for I16F16 {
	fn from(value: i32) -> Self {
		Self(value << 16)
	}
}

impl From<I16F16> for i32 {
	fn from(value: I16F16) -> Self {
		(value.0 + (1 << (16 - 1))) >> 16
	}
}

impl From<f64> for I16F16 {
	fn from(value: f64) -> Self {
		Self((value * (1 << 16) as f64) as i32)
	}
}

impl From<I16F16> for f64 {
	fn from(value: I16F16) -> Self {
		(value.0 as f64) * (1.0 / (1 << 16) as f64)
	}
}

impl PartialEq<i32> for I16F16 {
	fn eq(&self, other: &i32) -> bool {
		i32::from(*self) == *other
	}
}

impl PartialOrd<i32> for I16F16 {
	fn partial_cmp(&self, other: &i32) -> Option<std::cmp::Ordering> {
		i32::from(*self).partial_cmp(other)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn roundtrip_i32() {
		let int1 = 1234_i32;
		let fixed = I16F16::from(int1);
		let int2 = i32::from(fixed);
		assert_eq!(int2, 1234_i32);
	}

	#[test]
	fn roundtrip_f64() {
		let double1 = 0.1234_f64;
		let fixed = I16F16::from(double1);
		let double2 = f64::from(fixed);
		assert!((double2 - double1).abs() < 0.0001);
	}
}
