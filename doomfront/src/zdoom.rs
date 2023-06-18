//! Frontends for languages defined by the [ZDoom] family of source ports.
//!
//! [ZDoom]: https://zdoom.org/index

pub mod lex;

pub mod decorate;
pub mod language;
pub mod zscript;

use std::num::IntErrorKind;

pub use lex::Token;

/// Used to control [lexer](Token) behaviour; newer versions have more keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize, serde::Deserialize))]
pub struct Version {
	pub major: u16,
	pub minor: u16,
	pub rev: u32,
}

impl Version {
	pub const V1_0_0: Self = Self {
		major: 1,
		minor: 0,
		rev: 0,
	};

	pub const V2_4_0: Self = Self {
		major: 2,
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
		major: 4,
		minor: 9,
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

impl std::str::FromStr for Version {
	type Err = IntErrorKind;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut parts = s.split('.');

		let major = parts
			.next()
			.ok_or(IntErrorKind::Empty)?
			.parse::<u64>()
			.map_err(|err| err.kind().clone())?;

		let minor = parts.next().map_or(Ok(0), |m| {
			m.parse::<u64>().map_err(|err| err.kind().clone())
		})?;

		let rev = parts.next().map_or(Ok(0), |m| {
			m.parse::<u64>().map_err(|err| err.kind().clone())
		})?;

		Ok(Self {
			major: major.clamp(0, u16::MAX as u64) as u16,
			minor: minor.clamp(0, u16::MAX as u64) as u16,
			rev: rev.clamp(0, u16::MAX as u64) as u32,
		})
	}
}
