//! # VileTech ACS
//!
//! This crate represents the part of VileTech responsible for reading and consuming
//! Raven Software's [Action Code Script](https://doomwiki.org/wiki/ACS).
//!
//! At minimum, this contains a parser for its bytecode object files to be used by
//! a virtual machine; it may eventually grow to include a
//! [Cranelift](https://cranelift.dev/) backend for JIT compilation, and a whole
//! ACS toolchain at most.
//!
//! [Cranelift]: https://cranelift.dev/
//! [Action Code Script]: https://doomwiki.org/wiki/ACS

/// 4 ASCII characters rolled into one `u32`.
/// Byte ordering is **target-endianness dependent**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AsciiId(u32);

impl AsciiId {
	#[must_use]
	pub(crate) const fn _from_chars(a: u8, b: u8, c: u8, d: u8) -> Self {
		let a = a as u32;
		let b = b as u32;
		let c = c as u32;
		let d = d as u32;

		#[cfg(target_endian = "little")]
		{
			Self(a | (b << 8) | (c << 16) | (d << 24))
		}
		#[cfg(target_endian = "big")]
		{
			Self(d | (c << 8) | (b << 16) | (a << 24))
		}
	}

	#[must_use]
	pub(crate) const fn _from_bstr(bstr: &'static [u8; 4]) -> Self {
		let a = bstr[0] as u32;
		let b = bstr[1] as u32;
		let c = bstr[2] as u32;
		let d = bstr[3] as u32;

		#[cfg(target_endian = "little")]
		{
			Self(a | (b << 8) | (c << 16) | (d << 24))
		}
		#[cfg(target_endian = "big")]
		{
			Self(d | (c << 8) | (b << 16) | (a << 24))
		}
	}
}

impl From<u32> for AsciiId {
	fn from(value: u32) -> Self {
		Self(value)
	}
}
