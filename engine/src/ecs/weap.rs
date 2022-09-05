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

use glam::Vec2;

use crate::data::game::AssetIndex;

/// Allows an `Item` entity to be wielded as a weapon.
#[derive(Debug, shipyard::Component)]
pub struct Weapon {
	flags: WeaponFlags,
	bob_style: WeaponBobStyle,
	bob_range: Vec2,
	slot_priority: i16,
	fallback_priority: i16,
	crosshair: AssetIndex,
}

#[repr(u8)]
#[derive(Debug)]
enum WeaponBobStyle {
	Normal,
	Alpha,
	Smooth,
	InverseNormal,
	InverseAlpha,
	InverseSmooth,
}

bitflags::bitflags! {
	pub struct WeaponFlags : u8 {
		const NONE = 0;
		const NO_AUTOAIM = 1 << 0;
		/// If picking up the first instance of this weapon, don't
		/// automatically switch to it as per the vanilla Doom behavior.
		const NO_PICKUP_AUTOSWITCH = 1 << 1;
		/// If wielding this weapon, and ammo is collected for another weapon which
		/// is not `WEAK` and not marked `NO_FROMWEAK_AUTOSWITCH`, automatically
		/// switch to that weapon at the first possible opportunity.
		const WEAK = 1 << 2;
		const NO_FROMWEAK_AUTOSWITCH = 1 << 3;
		const NOBOB = 1 << 4;
		const ALLOW_DEATH_INPUT = 1 << 5;
	}
}
