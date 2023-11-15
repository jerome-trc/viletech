use std::num::NonZeroU64;

use bevy::{
	asset::load_internal_asset,
	core_pipeline::core_3d::CORE_3D_DEPTH_FORMAT,
	prelude::*,
	render::{
		extract_component::{ExtractComponent, ExtractComponentPlugin},
		render_asset::RenderAssets,
		render_resource::{
			BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutDescriptor,
			BindGroupLayoutEntry, BindingType, BufferBindingType, CachedRenderPipelineId,
			ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState,
			FragmentState, MultisampleState, PipelineCache, PrimitiveState, RawVertexBufferLayout,
			RenderPipelineDescriptor, SamplerBindingType, ShaderStages, ShaderType,
			SpecializedRenderPipeline, SpecializedRenderPipelines, StencilFaceState, StencilState,
			TextureFormat, TextureSampleType, TextureViewDimension, VertexAttribute,
			VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
		},
		renderer::RenderDevice,
		view::{ExtractedView, ViewUniform, ViewUniforms},
		Render, RenderApp, RenderSet,
	},
};

pub struct Sky2dPlugin;

impl Plugin for Sky2dPlugin {
	fn build(&self, app: &mut App) {
		load_internal_asset!(
			app,
			SKY2D_SHADER_HANDLE,
			"../../../assets/viletech/shaders/sky.wgsl",
			Shader::from_wgsl
		);

		app.add_plugins(ExtractComponentPlugin::<Sky2d>::default());

		let render_app = match app.get_sub_app_mut(RenderApp) {
			Ok(render_app) => render_app,
			Err(_) => return,
		};

		render_app
			.init_resource::<SpecializedRenderPipelines<Sky2dPipeline>>()
			.add_systems(
				Render,
				(
					prepare_sky2d_pipelines.in_set(RenderSet::Prepare),
					prepare_sky2d_bind_groups.in_set(RenderSet::PrepareBindGroups),
				),
			);
	}

	fn finish(&self, app: &mut App) {
		let render_app = match app.get_sub_app_mut(RenderApp) {
			Ok(render_app) => render_app,
			Err(_) => return,
		};

		let render_device = render_app.world.resource::<RenderDevice>().clone();

		render_app.insert_resource(Sky2dPipeline::new(&render_device));
	}
}

#[derive(Component, ExtractComponent, Clone)]
pub struct Sky2d(pub Handle<Image>);

#[derive(Component)]
pub struct Sky2dPipelineId(pub CachedRenderPipelineId);

#[derive(Component)]
pub struct Sky2dBindGroup(pub BindGroup);

#[derive(Resource)]
struct Sky2dPipeline {
	bind_group_layout: BindGroupLayout,
}

impl Sky2dPipeline {
	#[must_use]
	fn new(device: &RenderDevice) -> Self {
		let desc = BindGroupLayoutDescriptor {
			label: Some("viletech_sky2d_bgl"),
			entries: &[
				BindGroupLayoutEntry {
					binding: 0,
					visibility: ShaderStages::VERTEX,
					ty: BindingType::Buffer {
						ty: BufferBindingType::Uniform,
						has_dynamic_offset: true,
						min_binding_size: Some(ViewUniform::min_size()),
					},
					count: None,
				},
				BindGroupLayoutEntry {
					binding: 1,
					visibility: ShaderStages::FRAGMENT,
					ty: BindingType::Texture {
						sample_type: TextureSampleType::Float { filterable: true },
						view_dimension: TextureViewDimension::D2,
						multisampled: false,
					},
					count: None,
				},
				BindGroupLayoutEntry {
					binding: 2,
					visibility: ShaderStages::FRAGMENT,
					ty: BindingType::Sampler(SamplerBindingType::Filtering),
					count: None,
				},
				// BindGroupLayoutEntry {
				// 	binding: 202,
				// 	visibility: ShaderStages::FRAGMENT,
				// 	ty: BindingType::Buffer {
				// 		ty: BufferBindingType::Uniform,
				// 		has_dynamic_offset: true,
				// 		min_binding_size: Some(
				// 			NonZeroU64::new(std::mem::size_of::<f32>() as u64).unwrap(),
				// 		),
				// 	},
				// 	count: None,
				// },
			],
		};

		Self {
			bind_group_layout: device.create_bind_group_layout(&desc),
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Sky2dPipelineKey {
	samples: u32,
	depth_format: TextureFormat,
}

impl SpecializedRenderPipeline for Sky2dPipeline {
	type Key = Sky2dPipelineKey;

	fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
		let vbl = VertexBufferLayout {
			array_stride: (std::mem::size_of::<Vec3>() as u64)
				+ (std::mem::size_of::<Vec2>() as u64),
			step_mode: VertexStepMode::Vertex,
			attributes: vec![
				VertexAttribute {
					format: VertexFormat::Float32x3,
					offset: 0,
					shader_location: 0,
				},
				VertexAttribute {
					format: VertexFormat::Float32x2,
					offset: std::mem::size_of::<Vec3>() as u64,
					shader_location: 1,
				},
			],
		};

		RenderPipelineDescriptor {
			label: Some("sky2d_pipeline".into()),
			layout: vec![self.bind_group_layout.clone()],
			push_constant_ranges: Vec::new(),
			vertex: VertexState {
				shader: SKY2D_SHADER_HANDLE,
				shader_defs: Vec::new(),
				entry_point: "sky2d_vertex".into(),
				buffers: vec![vbl],
			},
			primitive: PrimitiveState::default(),
			depth_stencil: Some(DepthStencilState {
				format: key.depth_format,
				depth_write_enabled: false,
				depth_compare: CompareFunction::GreaterEqual,
				stencil: StencilState {
					front: StencilFaceState::IGNORE,
					back: StencilFaceState::IGNORE,
					read_mask: 0,
					write_mask: 0,
				},
				bias: DepthBiasState {
					constant: 0,
					slope_scale: 0.0,
					clamp: 0.0,
				},
			}),
			multisample: MultisampleState {
				count: key.samples,
				mask: !0,
				alpha_to_coverage_enabled: false,
			},
			fragment: Some(FragmentState {
				shader: SKY2D_SHADER_HANDLE,
				shader_defs: Vec::new(),
				entry_point: "sky2d_fragment".into(),
				targets: vec![Some(ColorTargetState {
					format: TextureFormat::Rgba32Float,
					// `BlendState::REPLACE`` is not needed here,
					// and None will be potentially much faster in some cases.
					blend: None,
					write_mask: ColorWrites::ALL,
				})],
			}),
		}
	}
}

fn prepare_sky2d_pipelines(
	mut commands: Commands,
	pipeline_cache: Res<PipelineCache>,
	mut pipelines: ResMut<SpecializedRenderPipelines<Sky2dPipeline>>,
	pipeline: Res<Sky2dPipeline>,
	msaa: Res<Msaa>,
	views: Query<(Entity, &ExtractedView), With<Sky2d>>,
) {
	for (entity, view) in &views {
		let pipeline_id = pipelines.specialize(
			&pipeline_cache,
			&pipeline,
			Sky2dPipelineKey {
				samples: msaa.samples(),
				depth_format: CORE_3D_DEPTH_FORMAT,
			},
		);

		commands.entity(entity).insert(Sky2dPipelineId(pipeline_id));
	}
}

fn prepare_sky2d_bind_groups(
	mut commands: Commands,
	pipeline: Res<Sky2dPipeline>,
	view_uniforms: Res<ViewUniforms>,
	images: Res<RenderAssets<Image>>,
	render_device: Res<RenderDevice>,
	views: Query<(Entity, &Sky2d)>,
) {
	for (entity, sky2d) in &views {
		if let (Some(gpu_img), Some(view_uniforms)) =
			(images.get(&sky2d.0), view_uniforms.uniforms.binding())
		{
			let bind_group = render_device.create_bind_group(
				"sky2d_bind_group",
				&pipeline.bind_group_layout,
				&BindGroupEntries::sequential((
					view_uniforms,
					&gpu_img.texture_view,
					&gpu_img.sampler,
				)),
			);

			commands.entity(entity).insert(Sky2dBindGroup(bind_group));
		}
	}
}

const SKY2D_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(4237892349);
