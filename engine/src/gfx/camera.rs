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

use dolly::prelude::*;

pub struct Camera {
	pub rig: CameraRig,
	pub fov_y: f32,
	pub aspect: f32,
	pub z_near: f32,
	pub z_far: f32,
}

impl Camera {
	pub fn new(surface_width: f32, surface_height: f32) -> Self {
		Camera {
			rig: CameraRig::builder()
				.with(Position::new(glam::Vec3::ZERO))
				.with(YawPitch::new())
				.with(Smooth::new_position_rotation(1.0, 1.0))
				.build(),
			fov_y: 45.0,
			aspect: surface_width / surface_height,
			z_near: 0.1,
			z_far: 100.0,
		}
	}

	/// Returns a view-projection matrix.
	pub fn update(&mut self, delta_t: f32) -> glam::Mat4 {
		let xform = self.rig.update(delta_t);
		let view = glam::Mat4::look_at_rh(
			xform.position,
			xform.position + xform.forward(),
			xform.position + xform.up(),
		);
		let proj = glam::Mat4::perspective_rh(self.fov_y, self.aspect, self.z_near, self.z_far);
		proj * view
	}

	pub fn resize(&mut self, new_width: f32, new_height: f32) {
		self.aspect = new_width / new_height
	}
}
