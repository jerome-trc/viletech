//! Graphics-related symbols.

use bevy::{
	prelude::*,
	reflect::TypeUuid,
	render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(AsBindGroup, Reflect, Asset, Debug, Clone, TypeUuid)]
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

#[derive(AsBindGroup, Reflect, Asset, Debug, Clone, TypeUuid)]
#[uuid = "8754faf6-ee9a-11ed-a05b-0242ac120003"]
pub struct Sky2dMaterial {
	#[texture(200)]
	#[sampler(201)]
	pub texture: Handle<Image>,
	#[uniform(202)]
	pub tiled_band_size: f32,
}

impl Material for Sky2dMaterial {
	fn vertex_shader() -> ShaderRef {
		concat!(
			env!("CARGO_MANIFEST_DIR"),
			"/../assets/viletech/shaders/sky.wgsl"
		)
		.into()
	}

	fn fragment_shader() -> ShaderRef {
		concat!(
			env!("CARGO_MANIFEST_DIR"),
			"/../assets/viletech/shaders/sky.wgsl"
		)
		.into()
	}
}
