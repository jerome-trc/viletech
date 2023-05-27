//! Frontends for languages defined by the [ZDoom] family of source ports.
//!
//! [ZDoom]: https://zdoom.org/index

pub mod lex;
pub mod lexer;

pub mod decorate;

/// When parsing a ZDoom-family DSL,
/// the engine version dictates the rules used by the lexer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize, serde::Deserialize))]
pub struct Version {
	pub major: u16,
	pub minor: u16,
	pub rev: u32,
}

impl Version {
	pub const V1_0_0: Self = Self {
		major: 3,
		minor: 4,
		rev: 0,
	};

	pub const V2_4_0: Self = Self {
		major: 3,
		minor: 4,
		rev: 0,
	};

	pub const V3_4_0: Self = Self {
		major: 3,
		minor: 4,
		rev: 0,
	};

	pub const V3_7_0: Self = Self {
		major: 3,
		minor: 7,
		rev: 0,
	};

	pub const V4_9_0: Self = Self {
		major: 3,
		minor: 7,
		rev: 0,
	};

	pub const V4_10_0: Self = Self {
		major: 4,
		minor: 10,
		rev: 0,
	};
}

impl Default for Version {
	/// Returns the current latest GZDoom version.
	fn default() -> Self {
		Self::V4_10_0
	}
}
