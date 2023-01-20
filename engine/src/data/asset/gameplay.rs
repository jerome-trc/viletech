//! Damage types, actor species, et cetera.

use bitflags::bitflags;

use super::Asset;

#[derive(Debug)]
pub struct DamageType {
	pub base_factor: f32,
	pub flags: DamageTypeFlags,
}

bitflags! {
	pub struct DamageTypeFlags: u8 {
		const REPLACE_FACTOR = 1 << 0;
		const BYPASS_ARMOR = 1 << 1;
	}
}

impl Asset for DamageType {}
