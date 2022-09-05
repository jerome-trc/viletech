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

#[derive(Debug, shipyard::Component)]
pub struct Bounce {
	flags: BounceFlags,
	/// How many times can this entity bounce before expiring?
	/// If the blueprint doesn't specify otherwise, this is effectively infinite.
	max_count: u32,
	lostvel_wall: f32,
	lostvel_floorceil: f32,
	/// How many times has this entity bounced thus far?
	bounces: u32,
}

bitflags::bitflags! {
	pub struct BounceFlags: u16 {
		const NONE = 0;
		const ON_WALLS = 1 << 0;
		const ON_FLOORS = 1 << 1;
		const ON_CEILINGS = 1 << 2;
		const AUTO_OFF = 1 << 3;
		const AUTO_OFF_FLOOR_ONLY = 1 << 4;
		const LIKE_HERETIC = 1 << 5;
		const ON_ACTORS = 1 << 6;
		const ON_UNRIPPABLE = 1 << 7;
		const NO_WALL_SOUND = 1 << 8;
		const STATE_RESOLVE = 1 << 9;
		const MBF = 1 << 15;
	}
}
