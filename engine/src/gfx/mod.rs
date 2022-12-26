//! All graphics symbols.

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
along with this program. If not, see <http://www.gnu.org/licenses/>.

*/

pub type Rgb32 = palette::rgb::Rgb<palette::encoding::Srgb, u8>;

pub mod camera;
pub mod core;
pub mod doom;
pub mod error;
pub mod render;

/// For building a shader wherein the vertex and shader entry points are both
/// in the same text source. Only works for WGSL.
pub fn create_shader_module<'d>(
	device: &'d wgpu::Device,
	label: &'d str,
	source: &'d str,
) -> wgpu::ShaderModule {
	use wgpu::{ShaderModuleDescriptor, ShaderSource};

	let source = ShaderSource::Wgsl(std::borrow::Cow::Borrowed(source));

	let desc = ShaderModuleDescriptor {
		label: Some(label),
		source,
	};

	device.create_shader_module(desc)
}

pub fn create_shader_states<'m>(
	module: &'m wgpu::ShaderModule,
	entry_point_vert: &'m str,
	vertex_buffers: &'m [wgpu::VertexBufferLayout],
	entry_point_frag: &'m str,
	color_targets: &'m [Option<wgpu::ColorTargetState>],
) -> (wgpu::VertexState<'m>, wgpu::FragmentState<'m>) {
	(
		wgpu::VertexState {
			module,
			entry_point: entry_point_vert,
			buffers: vertex_buffers,
		},
		wgpu::FragmentState {
			module,
			entry_point: entry_point_frag,
			targets: color_targets,
		},
	)
}
