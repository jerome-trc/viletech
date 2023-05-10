//! Structures making up individual difficulty configurations.

use bitflags::bitflags;

use crate::data::dobj;

/// i.e., a difficulty setting.
#[derive(Debug)]
pub struct SkillInfo {
	pub flags: SkillFlags,
	/// e.g. "Hurt Me Plenty". Displayed to the user. May be a string ID.
	pub name: String,
	/// Displayed to the user in the selection menu.
	pub graphic: dobj::InHandle<dobj::Image>,
	pub spawn_filter: SpawnFilter,
	pub respawn_time: u32,
	pub respawn_limit: u32,
	pub ammo_factor: f32,
	pub ammo_loot_factor: f32,
	pub damage_factor: f32,
	/// Minimum 0.0, maximum 1.0.
	pub mons_aggression: f32,
	pub heal_factor: f32,
	pub knockback_factor: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpawnFilter {
	Baby = 1,
	Easy,
	Normal,
	Hard,
	Nightmare,
	Udmf6,
	Udmf7,
	Udmf8,
	Udmf9,
	Udmf10,
	Udmf11,
	Udmf12,
	Udmf13,
	Udmf14,
	Udmf15,
}

bitflags! {
	pub struct SkillFlags: u16 {
		const DEFAULT = 1 << 0;
		/// Does not appear in the selection menu.
		const HIDDEN = 1 << 1;
		const FORCE_CONFIRM = 1 << 2;
		/// Halves the duration of certain specially-marked monster states.
		/// Additionally, projectiles flying monsters move faster.
		const FAST_MONSTERS = 1 << 3;
		/// Doubles the duration of certain specially-marked monster states.
		const SLOW_MONSTERS = 1 << 4;
		const NO_INFIGHTING = 1 << 5;
		const TOTAL_INFIGHTING = 1 << 6;
		/// Keys appear on the automap at all times, for Heretic's min. difficulty.
		const AUTOMAP_KEYS = 1 << 7;
		const INSTANT_REACTION = 1 << 8;
		const NO_MONS_PAIN = 1 << 9;
		const MULTIPLAYER_SPAWN = 1 << 10;
		const ALLOW_RESPAWN = 1 << 11;
		const AUTO_USE_HEALTH = 1 << 12;
	}
}
