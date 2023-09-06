use std::{any::TypeId, collections::HashMap};

use crate::{
	rti::{self, Record, RtInfo},
	tsys::TypeDef,
	zname::ZName,
	Version,
};

#[derive(Debug, Default)]
pub struct Project {
	pub(crate) libs: Vec<Library>,
	/// Names are fully-qualified.
	pub(crate) rti: HashMap<ZName, Record>,
}

impl Project {
	/// Note that `name` must be fully-qualified.
	#[must_use]
	pub fn get<R: RtInfo>(&self, name: impl AsRef<str>) -> Option<rti::Ref<R>> {
		let typeid = TypeId::of::<R>();
		let Some(record) = self.rti.get(name.as_ref()) else { return None; };

		unsafe {
			match record.tag {
				rti::StoreTag::Function => (typeid == TypeId::of::<rti::Function>())
					.then(|| rti::Ref::new(self, std::mem::transmute::<_, _>(&record.inner.func))),
				rti::StoreTag::Data => (typeid == TypeId::of::<rti::Data>())
					.then(|| rti::Ref::new(self, std::mem::transmute::<_, _>(&record.inner.data))),
				rti::StoreTag::Type => (typeid == TypeId::of::<TypeDef>()).then(|| {
					rti::Ref::new(self, std::mem::transmute::<_, _>(&record.inner.typedef))
				}),
			}
		}
	}

	pub fn clear(&mut self) {
		self.libs.clear();
		self.rti.clear();
	}
}

#[derive(Debug)]
pub struct Library {
	pub(crate) name: String,
	pub(crate) version: Version,
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
