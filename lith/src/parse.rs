//! Functions for parsing different elements of the Lithica syntax.

mod common;

use crate::Syntax;

pub type Error = doomfront::ParseError<Syntax>;
