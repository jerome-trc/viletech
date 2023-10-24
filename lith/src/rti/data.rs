//! Runtime data object information.

use cranelift_module::DataId;

#[derive(Debug)]
pub struct DataObj {
	pub(crate) _ptr: *const u8,
	pub(crate) id: DataId,
	pub(crate) size: usize,
	pub(crate) immutable: bool,
}

impl DataObj {
	#[must_use]
	pub fn id(&self) -> DataId {
		self.id
	}

	#[must_use]
	pub fn size(&self) -> usize {
		self.size
	}

	#[must_use]
	pub fn is_immutable(&self) -> bool {
		self.immutable
	}
}
