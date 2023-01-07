//! Infrastructure powering the LithScript language.

#[allow(dead_code)]
pub mod ast;
mod func;
mod interop;
mod module;
pub mod parse;
mod tsys;
mod word;

pub use interop::{Params, Returns};
pub use module::{Builder as ModuleBuilder, Module, OpenModule};
pub use tsys::*;

/// No LithScript identifier in human-readable form may exceed this byte length.
/// Mind that Lith only allows ASCII alphanumerics and underscores for identifiers,
/// so this is also a character limit.
/// For reference, the following string is exactly 64 ASCII characters:
/// `_0_i_weighed_down_the_earth_through_the_stars_to_the_pavement_9_`
pub const MAX_IDENT_LEN: usize = 64;

/// The maximum number of parameters which may be declared in a LithScript
/// function's signature; also the maximum number of arguments which may be passed.
/// Native functions are also bound to this limit.
pub const MAX_PARAMS: usize = 12;

/// The maximum number of return values which may be declared in a LithScript
/// function's signature; also the maximum number of values that a function
/// may return. Native functions are also bound to this limit.
pub const MAX_RETS: usize = 4;

#[derive(Debug)]
pub enum Error {
	Parse(parse::Error),
	/// Tried to retrieve a symbol from a module using an identifier that didn't
	/// resolve to anything.
	UnknownIdentifier,
	/// Tried to retrieve a function from a module and found it, but failed to
	/// pass the generic arguments matching its signature.
	SignatureMismatch,
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Self::Parse(err) => Some(err),
			_ => None,
		}
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Parse(err) => err.fmt(f),
			Self::UnknownIdentifier => write!(
				f,
				"Module symbol lookup failure; identifier didn't resolve to anything."
			),
			Self::SignatureMismatch => write!(
				f,
				"Module symbol lookup failure; function signature mismatch."
			),
		}
	}
}
