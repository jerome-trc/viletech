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

use crate::data::game::AssetId;
use bitflags::bitflags;

pub struct DamageType {
	id: AssetId,

	base_factor: f32,
	flags: DamageTypeFlags,
}

bitflags! {
	pub struct DamageTypeFlags : u8 {
		const NONE = 0;
		const REPLACE_FACTOR = 1 << 0;
		const BYPASS_ARMOR = 1 << 1;
	}
}

pub struct DamageOverTime {
	damage_type: AssetId,
	/// Applied per tic.
	damage: i32,
	tics_remaining: u32,
	/// Damage is applied every `interval` tics.
	interval: u32,
}

pub struct Species {
	id: AssetId,
}

pub struct ActorState {
	duration: i16,
	tic_range: u16,
	flags: ActorStateFlags,
}

bitflags! {
	pub struct ActorStateFlags : u8 {
		const NONE = 0;
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
