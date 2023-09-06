//! [`RString`] but with case-insensitive comparison and hashing.

use std::{
	hash::{Hash, Hasher},
	ops::Deref,
};

use util::rstring::RString;

/// [`RString`] but with case-insensitive comparison and hashing.
#[derive(Debug, Clone)]
#[repr(transparent)]
pub(crate) struct ZName(RString);

impl PartialEq for ZName {
	fn eq(&self, other: &Self) -> bool {
		self.0.deref().eq_ignore_ascii_case(other.0.as_ref())
	}
}

impl Eq for ZName {}

impl Hash for ZName {
	fn hash<H: Hasher>(&self, state: &mut H) {
		for c in self.0.deref().chars() {
			c.to_ascii_lowercase().hash(state);
		}
	}
}

impl std::borrow::Borrow<str> for ZName {
	fn borrow(&self) -> &str {
		self.0.deref()
	}
}

impl std::ops::Deref for ZName {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.0.deref()
	}
}

impl From<RString> for ZName {
	fn from(value: RString) -> Self {
		Self(value)
	}
}

impl From<&RString> for ZName {
	fn from(value: &RString) -> Self {
		Self(value.clone())
	}
}

impl std::fmt::Display for ZName {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}
