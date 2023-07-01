//! # LithScript
//!
//! The LithSC toolchain; VileTech's answer to the [ZScript] programming language
//! used by GZDoom and Raze, designed for being transpiled to from ZScript and
//! other ZDoom languages, while not compromising on versatility as a game script,
//! as ZScript did.
//!
//! [ZScript]: https://zdoom.org/wiki/ZScript

#![allow(unused)]
#![allow(dead_code)]

pub mod ast;
pub mod codegen;
pub mod compile;
mod heap;
pub mod issue;
pub mod lir;
pub mod parse;
pub mod project;
pub mod rti;
pub mod runtime;
mod syn;
pub mod tsys;
#[cfg(feature = "viletech")]
pub mod viletech;

pub use self::{heap::TPtr, project::Project, runtime::Runtime, syn::Syn};

#[cfg(feature = "viletech")]
pub use viletech::IncludeTree;

pub type ParseTree = doomfront::ParseTree<Syn>;
pub type SyntaxNode = doomfront::rowan::SyntaxNode<Syn>;
pub type SyntaxToken = doomfront::rowan::SyntaxToken<Syn>;
pub type SyntaxElem = doomfront::rowan::SyntaxElement<Syn>;

/// Each library is declared as belonging to a version of the LithScript specification.
///
/// The specification is versioned as per [Semantic Versioning](https://semver.org/).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Version {
	pub major: u16,
	pub minor: u16,
	pub rev: u16,
}

impl std::str::FromStr for Version {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut parts = s.split('.');

		let major = parts
			.next()
			.ok_or(Error::EmptyVersion)?
			.parse()
			.map_err(Error::SemVerParse)?;

		let minor = parts
			.next()
			.map_or(Ok(0), |m| m.parse::<u16>().map_err(Error::SemVerParse))?;

		let rev = if let Some(r) = parts.next() {
			r.parse::<u16>().map_err(Error::SemVerParse)?
		} else {
			0
		};

		Ok(Self { major, minor, rev })
	}
}

impl Version {
	#[must_use]
	pub const fn new(major: u16, minor: u16, rev: u16) -> Self {
		Self { major, minor, rev }
	}

	/// Check if this version is equal to an existing LithScript spec version.
	#[must_use]
	pub fn is_valid(&self) -> bool {
		matches!(
			self,
			Version {
				major: 0,
				minor: 0,
				rev: 0,
			}
		)
	}
}

/// Things that can go wrong when using this crate, excluding parse and compilation issues.
#[derive(Debug)]
pub enum Error {
	/// Tried to parse a SemVer string without any numbers or periods in it.
	EmptyVersion,
	SemVerParse(std::num::ParseIntError),
	/// Tried to retrieve a function from a library and found it, but failed to
	/// pass the generic arguments matching its signature.
	SignatureMismatch,
	/// Tried to retrieve a symbol from a library using an identifier that didn't
	/// resolve to anything.
	UnknownIdent,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::EmptyVersion => write!(f, "tried to parse an empty version string"),
			Self::SemVerParse(err) => err.fmt(f),
			Self::SignatureMismatch => {
				write!(
					f,
					"incorrect signature used when downcasting a function pointer"
				)
			}
			Self::UnknownIdent => write!(f, "an identifier was not found in the symbol table"),
		}
	}
}
