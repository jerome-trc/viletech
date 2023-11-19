//! # VileData
//!
//! VileData is a library providing data structures for representing (and
//! procedures for reading, writing, introspecting, and manipulating) formats
//! that are relevant to anyone building id Tech 1-descendant technology, such as
//! a Doom source port.

#![doc(
	html_favicon_url = "https://media.githubusercontent.com/media/jerome-trc/viletech/master/assets/viletech/viletech.png",
	html_logo_url = "https://media.githubusercontent.com/media/jerome-trc/viletech/master/assets/viletech/viletech.png"
)]

#[cfg(feature = "acs")]
pub mod acs;
#[cfg(feature = "graphics")]
pub mod gfx;
pub mod level;

/// See <https://zdoom.org/wiki/Editor_number>. Used when populating levels.
pub type EditorNum = u16;
/// See <https://zdoom.org/wiki/Spawn_number>. Used by ACS.
pub type SpawnNum = u16;

/// Failure modes for reading data.
///
/// Also see [`level::Error`].
#[derive(Debug)]
pub enum Error {
	InvalidHeader { details: &'static str },
	MissingHeader { expected: usize },
	MissingRecord { expected: usize, actual: usize },
	SizeMismatch { expected: usize, actual: usize },
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::InvalidHeader { details } => {
				write!(f, "data header is malformed: {details}")
			}
			Self::MissingHeader { expected } => {
				write!(f, "expected at least {expected} bytes for a header")
			}
			Self::MissingRecord { expected, actual } => write!(
				f,
				"a record was cut off at {actual} bytes; needed at least {expected}"
			),
			Self::SizeMismatch { expected, actual } => {
				write!(f, "expected total file size of {expected}, found {actual}")
			}
		}
	}
}
