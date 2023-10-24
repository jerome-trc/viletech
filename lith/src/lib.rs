//! # Lithica

#![doc(
	html_favicon_url = "https://media.githubusercontent.com/media/jerome-trc/viletech/master/assets/viletech/viletech.png",
	html_logo_url = "https://media.githubusercontent.com/media/jerome-trc/viletech/master/assets/viletech/viletech.png"
)]

pub(crate) mod back;
pub(crate) mod builtins;
pub(crate) mod front;
pub(crate) mod intern;
pub(crate) mod interpret;
pub(crate) mod mem;
pub(crate) mod types;

pub mod ast;
pub mod compile;
pub mod filetree;
pub mod interop;
pub mod issue;
pub mod parse;
pub mod rti;
pub mod runtime;
pub mod syn;

pub use self::{compile::Compiler, syn::*};

pub use cranelift::codegen::settings::OptLevel;

pub type ParseTree = doomfront::ParseTree<Syn>;
pub type SyntaxElem = doomfront::rowan::SyntaxElement<Syn>;
pub type SyntaxNode = doomfront::rowan::SyntaxNode<Syn>;
pub type SyntaxToken = doomfront::rowan::SyntaxToken<Syn>;

/// Each Lithica library is declared as belonging to a version of the Lithica
/// specification, which uses [SemVer](https://semver.org/).
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
	/// Can arise during [`filetree::FileTree::from_fs`].
	FromUtf8(std::string::FromUtf8Error),
	Parse,
	/// Can arise during [`filetree::FileTree::from_fs`].
	ReadDir(std::io::Error),
	/// Can arise during [`filetree::FileTree::from_fs`].
	ReadFile(std::io::Error),
	/// See [`Version::from_str`].
	SemVerParse,
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Error::EmptyVersion => None,
			Error::FromUtf8(err) => Some(err),
			Error::Parse => None,
			Error::ReadDir(err) => Some(err),
			Error::ReadFile(err) => Some(err),
			Error::SemVerParse => None,
		}
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::EmptyVersion => write!(f, "tried to parse an empty version string"),
			Self::FromUtf8(err) => write!(
				f,
				"failed to convert file content to UTF-8 when building a file tree: {err}"
			),
			Self::Parse => write!(f, "library registration failed due to parsing errors"),
			Self::ReadDir(err) => write!(
				f,
				"failed to read a directory when building a file tree: {err}"
			),
			Self::ReadFile(err) => {
				write!(f, "failed to read a file when building a file tree: {err}")
			}
			Self::SemVerParse => write!(f, "SemVer parser could not match a known Lithica version"),
		}
	}
}

pub type ValVec = smallvec::SmallVec<[cranelift::codegen::data_value::DataValue; 1]>;

// These constants are used in `UserExternalName::namespace`.

pub(crate) const CLNS_LIBCALL: u32 = u32::MAX - 2;
pub(crate) const CLNS_NATIVE: u32 = u32::MAX - 1;
pub(crate) const CLNS_BUILTIN: u32 = u32::MAX;
