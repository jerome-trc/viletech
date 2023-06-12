mod common;
mod expr;

use crate::zdoom;

/// Gives context to functions yielding parser combinators
/// (e.g. the user's selected ZScript version).
///
/// Thus, this information never has to be passed through deep call trees, and any
/// breaking changes to this context are minimal in scope.
#[derive(Debug, Clone)]
pub struct ParserBuilder {
	pub(self) _version: zdoom::Version,
}

impl ParserBuilder {
	#[must_use]
	pub fn new(version: zdoom::Version) -> Self {
		Self { _version: version }
	}
}
