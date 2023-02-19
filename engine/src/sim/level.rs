/// Level state for the playsim and renderer.
use crate::{
	data::{asset, Handle},
	sim,
};

/// Stateful playsim map geometry information.
#[derive(Debug, Default)]
pub struct Level {
	pub base: Option<Handle<asset::Map>>,
	pub flags: LevelFlags,
	/// Time spent in this level thus far.
	pub tics_elapsed: u64,
}

bitflags::bitflags! {
	#[derive(Default)]
	pub struct LevelFlags: u8 {
		const FROZEN_LOCAL = 1 << 0;
		const FROZEN_GLOBAL = 1 << 1;
	}
}

impl sim::State {
	#[must_use]
	pub fn level_is_frozen_local(&self) -> bool {
		self.level.flags.contains(LevelFlags::FROZEN_LOCAL)
	}

	#[must_use]
	pub fn level_is_frozen_global(&self) -> bool {
		self.level.flags.contains(LevelFlags::FROZEN_GLOBAL)
	}

	#[must_use]
	pub fn level_is_frozen(&self) -> bool {
		self.level_is_frozen_local() || self.level_is_frozen_global()
	}
}
