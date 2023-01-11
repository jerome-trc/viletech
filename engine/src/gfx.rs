//! All graphics symbols.

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
