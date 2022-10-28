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

use std::ops::Range;

use wgpu::{
	BindGroupLayout, ColorTargetState, Device, FragmentState, FrontFace, IndexFormat,
	MultisampleState, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology,
	PushConstantRange, RenderPipeline, RenderPipelineDescriptor, ShaderModule, ShaderStages,
	VertexBufferLayout, VertexState,
};

pub struct PipelineBuilder<'d> {
	device: &'d Device,

	label: Option<&'d str>,
	bind_group_layouts: Vec<&'d BindGroupLayout>,
	push_constant_ranges: Vec<PushConstantRange>,
	vertex: Option<VertexState<'d>>,
	fragment: Option<FragmentState<'d>>,
	primitives: PrimitiveState,
	multisampling: MultisampleState,
}

impl<'d> PipelineBuilder<'d> {
	pub fn vertex(
		mut self,
		module: &'d ShaderModule,
		entry_point: &'d str,
		buffers: &'d [VertexBufferLayout],
	) -> Self {
		self.vertex = Some(VertexState {
			module,
			entry_point,
			buffers,
		});

		self
	}

	pub fn vertex_state(mut self, state: VertexState<'d>) -> Self {
		self.vertex = Some(state);
		self
	}

	pub fn fragment(
		mut self,
		module: &'d ShaderModule,
		entry_point: &'d str,
		targets: &'d [Option<ColorTargetState>],
	) -> Self {
		self.fragment = Some(FragmentState {
			module,
			entry_point,
			targets,
		});

		self
	}

	pub fn fragment_state(mut self, state: FragmentState<'d>) -> Self {
		self.fragment = Some(state);
		self
	}

	pub fn shader_states(mut self, states: (VertexState<'d>, FragmentState<'d>)) -> Self {
		self.vertex = Some(states.0);
		self.fragment = Some(states.1);
		self
	}

	pub fn bind_group_layout(&mut self, layout: &'d BindGroupLayout) {
		self.bind_group_layouts.push(layout);
	}

	pub fn push_constant_range(&mut self, stages: ShaderStages, range: Range<u32>) {
		self.push_constant_ranges
			.push(PushConstantRange { stages, range });
	}

	pub fn primitive(
		&mut self,
		topology: PrimitiveTopology,
		front_face: FrontFace,
		cull_mode: Option<wgpu::Face>,
		polygon_mode: PolygonMode,
	) {
		self.primitives = PrimitiveState {
			topology,
			front_face,
			cull_mode,
			polygon_mode,
			..self.primitives
		};
	}

	pub fn primitive_advanced(
		mut self,
		strip_index_format: IndexFormat,
		unclipped_depth: bool,
		conservative: bool,
	) -> Self {
		self.primitives = PrimitiveState {
			strip_index_format: Some(strip_index_format),
			unclipped_depth,
			conservative,
			..self.primitives
		};

		self
	}

	pub fn multisampling(mut self, count: u32, mask: u64, alpha_to_coverage: bool) -> Self {
		self.multisampling.count = count;
		self.multisampling.mask = mask;
		self.multisampling.alpha_to_coverage_enabled = alpha_to_coverage;
		self
	}

	pub fn build(self) -> RenderPipeline {
		self.device
			.create_render_pipeline(&RenderPipelineDescriptor {
				label: Some(&format!(
					"IMPURE: Render Pipeline, {}",
					self.label.unwrap_or("unnamed")
				)),
				layout: Some(
					&self
						.device
						.create_pipeline_layout(&PipelineLayoutDescriptor {
							label: Some(&format!(
								"IMPURE: Render Pipeline Layout, {}",
								self.label.unwrap_or("unnamed")
							)),
							bind_group_layouts: &self.bind_group_layouts,
							push_constant_ranges: &self.push_constant_ranges,
						}),
				),
				vertex: self
					.vertex
					.expect("Attempted to build a render pipeline with no vertex state."),
				fragment: self.fragment,
				primitive: self.primitives,
				depth_stencil: None,
				multisample: self.multisampling,
				multiview: None,
			})
	}
}

/// For label, only pass in something like "Sky" or "Sprite".
pub fn pipeline_builder<'d>(label: &'d str, device: &'d Device) -> PipelineBuilder<'d> {
	PipelineBuilder {
		device,
		label: Some(label),
		bind_group_layouts: Default::default(),
		push_constant_ranges: Default::default(),
		vertex: None,
		fragment: None,
		primitives: PrimitiveState {
			topology: wgpu::PrimitiveTopology::TriangleList,
			strip_index_format: None,
			front_face: wgpu::FrontFace::Ccw,
			cull_mode: Some(wgpu::Face::Back),
			polygon_mode: wgpu::PolygonMode::Fill,
			unclipped_depth: false,
			conservative: false,
		},
		multisampling: MultisampleState::default(),
	}
}
