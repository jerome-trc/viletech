//! Graphics-related symbols.

use std::num::NonZeroU32;

use bevy::{
	pbr::{MaterialPipeline, MaterialPipelineKey},
	prelude::*,
	reflect::TypeUuid,
	render::{
		mesh::{MeshVertexAttribute, MeshVertexBufferLayout},
		render_asset::RenderAssets,
		render_resource::{
			AsBindGroup, AsBindGroupError, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntry,
			BindingType, PreparedBindGroup, RenderPipelineDescriptor, SamplerBindingType,
			ShaderRef, ShaderStages, SpecializedMeshPipelineError, TextureSampleType,
			TextureViewDimension, UnpreparedBindGroup, VertexFormat,
		},
		renderer::RenderDevice,
		texture::FallbackImage,
	},
};
use vfs::FileSlot;

use crate::types::FxIndexSet;

pub const ATTRIBUTE_TEXINDEX: MeshVertexAttribute =
	MeshVertexAttribute::new("TexIndex", 666_451, VertexFormat::Uint32);

/// An implementation of [`bevy::app::Plugin`] which configures a Bevy app
/// for VileTech-relevant rendering.
#[derive(Default)]
pub struct GraphicsPlugin;

impl bevy::app::Plugin for GraphicsPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((
			MaterialPlugin::<Sky2dMaterial>::default(),
			MaterialPlugin::<TerrainMaterial>::default(),
		));
	}
}

/// A unique identifier for an image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageSlot {
	pub file: FileSlot,
	/// `0` is a minimum coordinate in a rectangle; `1` is a maximum coordinate.
	pub rect: Option<([u16; 2], [u16; 2])>,
}

#[derive(Asset, TypePath, Debug, Default, Clone)]
pub struct TerrainMaterial {
	pub set: FxIndexSet<ImageSlot>,
	pub textures: Vec<Handle<Image>>,
}

impl TerrainMaterial {
	const MAX_TEXTURE_COUNT: usize = 16;
}

impl Material for TerrainMaterial {
	fn vertex_shader() -> ShaderRef {
		"/home/jerome/Data/viletech/assets/viletech/shaders/terrain.wgsl".into()
	}

	fn fragment_shader() -> ShaderRef {
		"/home/jerome/Data/viletech/assets/viletech/shaders/terrain.wgsl".into()
	}

	fn specialize(
		_pipeline: &MaterialPipeline<Self>,
		descriptor: &mut RenderPipelineDescriptor,
		layout: &MeshVertexBufferLayout,
		_key: MaterialPipelineKey<Self>,
	) -> Result<(), SpecializedMeshPipelineError> {
		let vertex_layout = layout.get_layout(&[
			Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
			Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
			Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
			ATTRIBUTE_TEXINDEX.at_shader_location(3),
		])?;

		descriptor.vertex.buffers = vec![vertex_layout];
		Ok(())
	}
}

impl AsBindGroup for TerrainMaterial {
	type Data = ();

	fn as_bind_group(
		&self,
		layout: &BindGroupLayout,
		render_device: &RenderDevice,
		a_images: &RenderAssets<Image>,
		fallback_image: &FallbackImage,
	) -> Result<PreparedBindGroup<Self::Data>, AsBindGroupError> {
		let mut images = vec![];

		for handle in self.textures.iter().take(Self::MAX_TEXTURE_COUNT) {
			match a_images.get(handle) {
				Some(img) => images.push(img),
				None => return Err(AsBindGroupError::RetryNextUpdate),
			}
		}

		let fallback = &fallback_image.d2;
		let textures = vec![&fallback.texture_view; Self::MAX_TEXTURE_COUNT];
		let mut textures: Vec<_> = textures.into_iter().map(|t| &**t).collect();

		for (id, image) in images.into_iter().enumerate() {
			textures[id] = &*image.texture_view;
		}

		let bind_group = render_device.create_bind_group(
			"terrain_material_bind_group",
			layout,
			&BindGroupEntries::sequential((&textures[..], &fallback.sampler)),
		);

		Ok(PreparedBindGroup {
			bindings: vec![],
			bind_group,
			data: (),
		})
	}

	fn bind_group_layout_entries(_: &RenderDevice) -> Vec<BindGroupLayoutEntry>
	where
		Self: Sized,
	{
		vec![
			// @group(1) @binding(0) var textures: binding_array<texture_2d<f32>>;
			BindGroupLayoutEntry {
				binding: 0,
				visibility: ShaderStages::FRAGMENT,
				ty: BindingType::Texture {
					sample_type: TextureSampleType::Float { filterable: true },
					view_dimension: TextureViewDimension::D2,
					multisampled: false,
				},
				count: NonZeroU32::new(Self::MAX_TEXTURE_COUNT as u32),
			},
			// @group(1) @binding(1) var nearest_sampler: sampler;
			BindGroupLayoutEntry {
				binding: 1,
				visibility: ShaderStages::FRAGMENT,
				ty: BindingType::Sampler(SamplerBindingType::Filtering),
				count: None,
				// Note: as textures, multiple samplers can also be bound onto one binding slot.
				// One may need to pay attention to the limit of sampler binding amount on some platforms.
				// count: NonZeroU32::new(MAX_TEXTURE_COUNT as u32),
			},
		]
	}

	fn label() -> Option<&'static str> {
		Some("terrain material")
	}

	fn unprepared_bind_group(
		&self,
		_: &BindGroupLayout,
		_: &RenderDevice,
		_: &RenderAssets<Image>,
		_: &FallbackImage,
	) -> Result<UnpreparedBindGroup<Self::Data>, AsBindGroupError> {
		unreachable!()
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
