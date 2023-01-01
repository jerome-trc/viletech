//! Infrastructure powering the LithScript language.

#[allow(dead_code)]
pub mod ast;
mod func;
mod interop;
mod module;
pub mod parse;
mod word;

pub use interop::{Params, Returns};
pub use module::{Builder as ModuleBuilder, Module, OpenModule};

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
