//! Infrastructure powering the LithScript language.

/*

Copyright (C) 2022 ***REMOVED***

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <http://www.gnu.org/licenses/>.

*/

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
