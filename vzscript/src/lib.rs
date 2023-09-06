//! # VZScript
//!
//! The VZSC toolchain; VileTech's fork of the [ZScript](https://zdoom.org/wiki/ZScript)
//! programming language used by GZDoom and Raze, designed for transpilation from
//! ZScript and other ZDoom languages.

// TODO: Disallow
#![allow(unused)]
#![allow(dead_code)]

pub mod ast;
pub mod back;
pub mod compile;
pub mod front;
pub mod heap;
pub mod inctree;
pub mod interpreter;
pub mod issue;
pub mod native;
pub mod parse;
pub mod project;
pub mod rti;
pub mod runtime;
pub mod sema;
mod syn;
pub mod tsys;
pub mod vir;
pub mod zname;

use std::hash::BuildHasherDefault;

use rustc_hash::FxHasher;

pub use self::{inctree::IncludeTree, project::Project, runtime::Runtime, syn::Syn};

pub type FxDashMap<K, V> = dashmap::DashMap<K, V, BuildHasherDefault<FxHasher>>;
pub type FxDashSet<K> = dashmap::DashSet<K, BuildHasherDefault<FxHasher>>;

pub type ParseTree = doomfront::ParseTree<Syn>;
pub type SyntaxNode = doomfront::rowan::SyntaxNode<Syn>;
pub type SyntaxToken = doomfront::rowan::SyntaxToken<Syn>;
pub type SyntaxElem = doomfront::rowan::SyntaxElement<Syn>;

/// Each [library] is declared as belonging to a version of the VZScript
/// specification, which uses [SemVer](https://semver.org/).
///
/// [library]: project::Library
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

	/// Check if this version is equal to an existing VZScript spec version.
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

/// Failure modes of this crate's operations, excluding
/// [parse errors](parse::Error) and [compilation issues](issue).
#[derive(Debug)]
pub enum Error {
	/// Tried to parse a SemVer string without any numbers or periods in it.
	EmptyVersion,
	SemVerParse(std::num::ParseIntError),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::EmptyVersion => write!(f, "tried to parse an empty version string"),
			Self::SemVerParse(err) => write!(f, "SemVer parse error: {err}"),
		}
	}
}

#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
compiler_error!("exotic pointer widths are not yet supported");
