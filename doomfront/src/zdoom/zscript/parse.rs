mod common;
mod expr;

use super::Version;

#[derive(Debug, Clone)]
pub struct ParserBuilder {
	pub(self) _version: Version,
}

impl ParserBuilder {
	#[must_use]
	pub fn new(version: Version) -> Self {
		Self { _version: version }
	}
}
