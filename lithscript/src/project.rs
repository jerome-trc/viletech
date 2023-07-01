use std::{collections::HashMap, sync::Arc};

use util::rstring::RString;

use crate::{compile::JitModule, rti::Record, Version};

#[derive(Debug, Default)]
pub struct Project {
	pub(crate) libs: Vec<Library>,
	/// Names are fully-qualified.
	pub(crate) rti: HashMap<RString, Record>,
}

impl Project {
	pub fn clear(&mut self) {
		self.libs.clear();
		self.rti.clear();
	}
}

#[derive(Debug)]
pub struct Library {
	pub(crate) name: String,
	pub(crate) version: Version,
	#[allow(unused)]
	pub(crate) module: Arc<JitModule>,
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
