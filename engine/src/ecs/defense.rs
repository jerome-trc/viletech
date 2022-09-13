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

#[derive(Debug)]
pub struct DamageFactor {
	damage_type: AssetIndex,
	factor: f32
}

#[derive(Debug, shipyard::Component)]
pub struct Defense {
	flags: DefenseFlags,
	damage_factors: Vec<DamageFactor>,
}

bitflags::bitflags! {
	pub struct DefenseFlags: u8 {
		const NONE = 0;
		/// No incoming damage, no flincing, no target changes.
		const INVULNERABLE = 1 << 0;
		/// Entity has one indestructible hit point.
		const BUDDHA = 1 << 1;
		const NO_RADIUS_DAMAGE = 1 << 2;
		/// Piercing projectiles are destroyed upon hitting this entity.
		const NO_PIERCE = 1 << 3;
		const NO_MORPH = 1 << 4;
		const NO_TELEFRAG = 1 << 5;
		/// Force telefrag damage to be passed through damage factors.
		const TELEFRAG_FACTORS = 1 << 6;
		const NO_LIFESTEAL = 1 << 7;
	}
}
