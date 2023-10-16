//! Runtime data object information.

use cranelift_module::DataId;

#[derive(Debug)]
pub struct DataObj {
	pub(crate) ptr: *const u8,
	pub(crate) size: usize,
	pub(crate) id: DataId,
	pub(crate) immutable: bool,
}
