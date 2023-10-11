//! Functions for parsing different elements of the Lithica syntax.

mod common;
mod expr;

#[cfg(test)]
mod test;

use crate::Syn;

pub use self::expr::*;

pub type Error = doomfront::ParseError<Syn>;
