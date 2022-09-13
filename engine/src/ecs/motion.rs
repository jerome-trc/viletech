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

use glam::Vec3;

#[derive(Debug, shipyard::Component)]
pub struct Motion {
	flags: MotionFlags,
	mass: u32,
	velocity: Vec3,
	accel: Vec3,
	friction: f32,
	gravity: f32,
	max_step_height: f32,
	max_dropoff_height: f32,
	max_slope_steepness: f32,
}

bitflags::bitflags! {
	pub struct MotionFlags : u8 {
		const NONE = 0;
		const NOGRAVITY = 1 << 0;
		const FALLDAMAGE = 1 << 1;
	}
}
