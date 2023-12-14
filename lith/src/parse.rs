//! Functions for parsing different elements of the Lithica syntax.

mod common;
mod expr;

use crate::Syntax;

pub use self::expr::*;

pub type Error = doomfront::ParseError<Syntax>;
