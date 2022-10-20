/*

Copyright (C) 2022 ***REMOVED***

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.

*/

use crate::data::game::AssetIndex;

use bitflags::bitflags;
use mlua::prelude::*;

/// i.e., a difficulty setting.
pub struct SkillInfo {
	flags: SkillFlags,
	/// Displayed to the user. May be a string ID.
	name: String,
	/// Displayed to the user in the selection menu.
	graphic: AssetIndex,
	spawn_filter: SpawnFilter,
	respawn_time: u32,
	respawn_limit: u32,
	ammo_factor: f32,
	ammo_loot_factor: f32,
	damage_factor: f32,
	/// Minimum 0.0, maximum 1.0.
	mons_aggression: f32,
	heal_factor: f32,
	knockback_factor: f32,
}

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

pub struct DamageType {
	id: AssetIndex,

	base_factor: f32,
	flags: DamageTypeFlags,
}

bitflags! {
	pub struct DamageTypeFlags: u8 {
		const REPLACE_FACTOR = 1 << 0;
		const BYPASS_ARMOR = 1 << 1;
	}
}

#[derive(Debug)]
pub struct DamageOverTime {
	damage_type: AssetIndex,
	/// Applied per tic.
	damage: i32,
	tics_remaining: u32,
	/// Damage is applied every `interval` tics.
	interval: u32,
}

pub struct Species {
	id: AssetIndex,
}

pub enum ActorStateVisual {
	None,
	Sprite(AssetIndex),
	Voxel(AssetIndex),
	Model(AssetIndex),
}

pub struct ActorState {
	visual: ActorStateVisual,
	duration: i16,
	tic_range: u16,
	flags: ActorStateFlags,
	action: Option<LuaRegistryKey>,
}

impl ActorState {
	const INFINITE_DURATION: i16 = -1;
}

bitflags! {
	pub struct ActorStateFlags : u8 {
		const FAST = 1 << 0;
		const SLOW = 1 << 1;
		const FULLBRIGHT = 1 << 2;
		const CAN_RAISE = 1 << 3;
		const USER_0 = 1 << 4;
		const USER_1 = 1 << 5;
		const USER_2 = 1 << 6;
		const USER_3 = 1 << 7;
	}
}

pub struct ActorStateMachine {
	/// Each element's field `::1` indexes into `states`.
	labels: Vec<(String, usize)>,
	states: Vec<ActorState>,
}
