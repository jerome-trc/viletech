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

#[derive(Debug, shipyard::Component)]
pub struct Monster {
	flags: MonsterFlags,
	species: AssetIndex,
	/// In game-tics.
	reaction_time: u32,
	/// ::0 is the key for the level on which the trigger should fire when this
	/// monster is killed. ::1 is true if the trigger is secondary, false if primary.
	/// e.g. ("MAP07", false) for the Mancubus, ("MAP07", true) for the Arachnotron.
	boss_triggers: Vec<(String, bool)>,
}

bitflags::bitflags! {
	pub struct MonsterFlags : u8 {
		const AMBUSH = 1 << 0;
		const AVOID_HAZARDS = 1 << 1;
		const FAST_RETALIATE = 1 << 2;
		const NEVER_RESPAWN = 1 << 3;
		const NO_SPLASH_ALERT = 1 << 4;
	}
}
