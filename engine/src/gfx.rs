//! Graphics-related symbols.

use bevy::{
	prelude::*,
	reflect::TypeUuid,
	render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(AsBindGroup, Reflect, Debug, Clone, TypeUuid)]
#[uuid = "8754faf6-ee9a-11ed-a05b-0242ac120003"]
pub struct TerrainMaterial {
	#[texture(0)]
	#[sampler(1)]
	pub atlas: Handle<Image>,
	#[texture(2)]
	#[sampler(3)]
	pub colormap: Handle<Image>,
}

impl Material for TerrainMaterial {
	fn fragment_shader() -> ShaderRef {
		"/viletech/shaders/terrain.wgsl".into()
	}
}
