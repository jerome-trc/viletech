//! # Lithica

#![doc(
	html_favicon_url = "https://media.githubusercontent.com/media/jerome-trc/viletech/master/assets/viletech/viletech.png",
	html_logo_url = "https://media.githubusercontent.com/media/jerome-trc/viletech/master/assets/viletech/viletech.png"
)]

pub(crate) mod compile;
pub(crate) mod data;
pub(crate) mod front;
pub(crate) mod intern;

pub mod arena;
pub mod ast;
pub mod filetree;
pub mod interop;
pub mod issue;
pub mod parse;
pub mod rti;
pub mod runtime;
pub mod syn;

pub use self::{compile::*, front::decl::*, syn::*};

pub type ParseTree = doomfront::ParseTree<Syn>;
pub type SyntaxElem = doomfront::rowan::SyntaxElement<Syn>;
pub type SyntaxNode = doomfront::rowan::SyntaxNode<Syn>;
pub type SyntaxToken = doomfront::rowan::SyntaxToken<Syn>;

/// Each [library] is declared as belonging to a version of the Lithica
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

	/// Check if this version is equal to an existing Lithica spec version.
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

/// "Backend type".
pub(crate) type _BType = cranelift::codegen::ir::Type;
pub(crate) type _ValVec = smallvec::SmallVec<[cranelift::codegen::data_value::DataValue; 1]>;

pub(crate) type FxDashMap<K, V> =
	dashmap::DashMap<K, V, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>;
pub(crate) type FxDashView<K, V> =
	dashmap::ReadOnlyView<K, V, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>;
