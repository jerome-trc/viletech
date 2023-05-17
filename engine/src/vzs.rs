//! Infrastructure powering VileTech's implementation of the ZScript language.
//!
//! The VZSC toolchain; VileTech's fork of the [ZScript] programming language used
//! by GZDoom and Raze, intended to advance it by introducing breaking changes via
//! "editions" like Rust does.
//!
//! [ZScript]: https://zdoom.org/wiki/ZScript

mod abi;
// pub mod ast;
mod func;
pub mod heap;
mod inode;
mod issue;
mod module;
pub mod parse;
mod project;
mod runtime;
mod sym;
mod syn;
pub mod tsys;

pub use self::{
	func::{Flags as FunctionFlags, Function, TFunc},
	inode::*,
	issue::*,
	module::{Builder as ModuleBuilder, Module},
	parse::{FileParseTree, IncludeTree, ParseTree},
	project::*,
	runtime::*,
	sym::*,
	syn::Syn,
};

pub type SyntaxNode = doomfront::rowan::SyntaxNode<Syn>;
pub type SyntaxToken = doomfront::rowan::SyntaxToken<Syn>;

/// No VZScript identifier in human-readable form may exceed this byte length.
/// Mind that VZS only allows ASCII alphanumerics and underscores for identifiers,
/// so this is also a character limit.
/// For reference, the following string is exactly 64 ASCII characters:
/// `_0_i_weighed_down_the_earth_through_the_stars_to_the_pavement_9_`
pub const MAX_IDENT_LEN: usize = 64;

/// In terms of values, not quad-words.
pub const MAX_PARAMS: usize = 16;

/// In terms of values, not quad-words.
pub const MAX_RETURNS: usize = 4;

/// Each module is declared as belonging to a version of the VZScript specification.
///
/// The specification is versioned as per [Semantic Versioning](https://semver.org/).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
	/// Check if this version is equal to an existing VZScript spec version.
	#[must_use]
	pub fn is_valid(&self) -> bool {
		use std::collections::HashSet;

		use once_cell::sync::Lazy;

		static VERSIONS: Lazy<HashSet<Version>> = Lazy::new(|| {
			HashSet::from([Version {
				major: 0,
				minor: 0,
				rev: 0,
			}])
		});

		VERSIONS.contains(self)
	}
}

#[derive(Debug)]
pub enum Error {
	/// Tried to parse a SemVer string without any numbers or periods in it.
	EmptyVersion,
	SemVerParse(std::num::ParseIntError),
	/// Tried to retrieve a function from a module and found it, but failed to
	/// pass the generic arguments matching its signature.
	SignatureMismatch,
	/// Tried to retrieve a symbol from a module using an identifier that didn't
	/// resolve to anything.
	UnknownIdent,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::EmptyVersion => write!(f, "Tried to parse an empty version string."),
			Self::SemVerParse(err) => err.fmt(f),
			Self::SignatureMismatch => {
				write!(
					f,
					"Incorrect signature used when downcasting a function pointer."
				)
			}
			Self::UnknownIdent => write!(f, "An identifier was not found in the symbol table."),
		}
	}
}
