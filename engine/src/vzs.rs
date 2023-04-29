//! Infrastructure powering VileTech's implementation of the ZScript language.
//!
//! The VTZS toolchain; VileTech's fork of the [ZScript] programming language used
//! by GZDoom and Raze, intended to advance it by introducing breaking changes via
//! "editions" like Rust does.
//!
//! [ZScript]: https://zdoom.org/wiki/ZScript

mod abi;
pub mod ast;
mod func;
mod handle;
pub mod heap;
mod inode;
mod module;
mod parse;
mod project;
mod runtime;
mod symbol;
mod syn;
#[cfg(test)]
mod test;
mod tsys;

pub use self::{
	func::{Flags as FunctionFlags, Function, TFunc},
	handle::*,
	inode::*,
	module::{Builder as ModuleBuilder, Module},
	parse::*,
	project::*,
	runtime::*,
	symbol::*,
	syn::Syn,
	tsys::*,
};

pub type SyntaxNode = doomfront::rowan::SyntaxNode<Syn>;
pub type SyntaxToken = doomfront::rowan::SyntaxToken<Syn>;
pub type Token = doomfront::rowan::SyntaxToken<Syn>;

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

#[derive(Debug)]
pub enum Error {
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
			Self::SignatureMismatch => {
				write!(
					f,
					"Incorrect signature used when downcasting a function handle."
				)
			}
			Self::UnknownIdent => write!(f, "An identifier was not found in the symbol table."),
		}
	}
}
