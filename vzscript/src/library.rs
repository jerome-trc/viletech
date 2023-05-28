//! A collection of [modules](Module).

use std::collections::HashMap;

use crate::{module::Module, Version};

/// A collection of [modules](Module).
#[derive(Debug)]
pub struct Library {
	/// e.g. `core` or `viletech`. Follows C identifier rules.
	pub(super) name: String,
	/// A module's VZS version affects its compilation rules.
	pub(super) version: Version,
	/// Keys are fully-qualified paths (e.g. `vzscript/math`).
	pub(super) _modules: HashMap<String, Module>,
}

impl Library {
	#[must_use]
	pub fn name(&self) -> &str {
		&self.name
	}

	#[must_use]
	pub fn version(&self) -> Version {
		self.version
	}
}
