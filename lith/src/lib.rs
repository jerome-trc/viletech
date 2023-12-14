//! # Lithica
//!
//! ## About
//!
//! Lithica is a hybrid statically- and dynamically-typed scripting language that
//! takes after [Lua], [Terra], and [Zig]. It is designed for use in projects which:
//! - benefit from being able to quickly write dynamic scripts
//! - sometimes need to drop down to lower-level, statically-typed, JIT-compiled
//! functions to meet performance requirements (e.g. video games)
//! - have use for staged programming to performantly and flexibly modify the
//! behavior of the non-dynamic code at compile time
//!
//! [Lua]: https://www.lua.org/
//! [Terra]: https://terralang.org/
//! [Zig]: https://ziglang.org/

#![doc(
	html_favicon_url = "https://media.githubusercontent.com/media/jerome-trc/viletech/master/assets/viletech/viletech.png",
	html_logo_url = "https://media.githubusercontent.com/media/jerome-trc/viletech/master/assets/viletech/viletech.png"
)]

pub extern crate ariadne;

pub mod ast;
pub mod parse;
pub mod syntax;

pub use syntax::*;

pub type ParseTree = doomfront::ParseTree<Syntax>;
pub type SyntaxElem = doomfront::rowan::SyntaxElement<Syntax>;
pub type SyntaxNode = doomfront::rowan::SyntaxNode<Syntax>;
pub type SyntaxToken = doomfront::rowan::SyntaxToken<Syntax>;

/// Each Lithica chunk is compiled against a specific version of the Lithica
/// standard, which uses [SemVer](https://semver.org/).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Version(u16, u16, u16);

impl Version {
	pub const V0_0_0: Self = Self(0, 0, 0);
}

impl std::str::FromStr for Version {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"0.0.0" => Ok(Self::V0_0_0),
			"" => Err(Error::EmptyVersion),
			_ => Err(Error::SemVerParse),
		}
	}
}

/// Failure modes of this crate's operations, excluding [frontend issues](issue).
#[derive(Debug)]
pub enum Error {
	/// Tried to parse a SemVer string without any numbers or periods in it.
	/// See [`Version::from_str`].
	EmptyVersion,
	/// See [`Version::from_str`].
	SemVerParse,
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Error::EmptyVersion => None,
			Error::SemVerParse => None,
		}
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::EmptyVersion => write!(f, "tried to parse an empty version string"),
			Self::SemVerParse => write!(f, "SemVer parser could not match a known Lithica version"),
		}
	}
}
