//! A structure for holding all state common to rendering between scenes.

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

use std::{
	iter,
	time::{Duration, Instant},
};

use egui_wgpu::renderer::{RenderPass as EguiRenderPass, ScreenDescriptor};
use log::info;
use wgpu::{
	util::StagingBelt, CommandEncoder, CommandEncoderDescriptor, RenderPass, RenderPipeline,
	SurfaceConfiguration, SurfaceTexture, TextureView, TextureViewDescriptor,
};
use winit::{event_loop::EventLoopWindowTarget, window::Window};

use super::error::Error;

/// Holds all state common to rendering between scenes.
pub struct GraphicsCore {
	pub window: Window,
	pub window_size: winit::dpi::PhysicalSize<u32>,
	pub instance: wgpu::Instance,
	pub surface: wgpu::Surface,
	pub adapter: wgpu::Adapter,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
	pub surface_config: SurfaceConfiguration,
	pub pipelines: Vec<RenderPipeline>,
	pub staging: StagingBelt,
	pub last_frame_time: Instant,
	pub egui: EguiCore,
}

pub struct EguiCore {
	pub state: egui_winit::State,
	pub context: egui::Context,
	pub render_pass: EguiRenderPass,
}

impl GraphicsCore {
	pub fn new(
		window: Window,
		event_loop: &EventLoopWindowTarget<()>,
	) -> Result<GraphicsCore, Box<dyn std::error::Error>> {
		let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
		let surface = unsafe { instance.create_surface(&window) };

		let adpreq = instance.request_adapter(&wgpu::RequestAdapterOptions {
			power_preference: wgpu::PowerPreference::HighPerformance,
			compatible_surface: Some(&surface),
			force_fallback_adapter: false,
		});

		let adapter = match pollster::block_on(adpreq) {
			Some(a) => a,
			None => {
				return Err(Box::new(Error::AdapterRequest {}));
			}
		};

		let (device, queue) = match pollster::block_on(adapter.request_device(
			&wgpu::DeviceDescriptor {
				features: wgpu::Features::PUSH_CONSTANTS,
				limits: wgpu::Limits::default(),
				label: Some("IMPURE: Device"),
			},
			None,
		)) {
			Ok(dq) => dq,
			Err(err) => {
				return Err(Box::new(err));
			}
		};

		#[cfg(not(debug_assertions))]
		device.on_uncaptured_error(|err| {
			log::error!("WGPU error: {}", err);
		});

		{
			let adpinfo = adapter.get_info();

			info!("WGPU backend: {:?}", adpinfo.backend);
			info!("GPU: {} ({:?})", adpinfo.name, adpinfo.device_type);
		}

		let window_size = window.inner_size();
		let tex_formats = surface.get_supported_formats(&adapter);

		const PREFERRED_FORMATS: [wgpu::TextureFormat; 5] = [
			wgpu::TextureFormat::Bgra8UnormSrgb,
			wgpu::TextureFormat::Rgba8UnormSrgb,
			wgpu::TextureFormat::Bgra8Unorm,
			wgpu::TextureFormat::Rgba8Unorm,
			wgpu::TextureFormat::Rgba16Float,
		];

		let srf_format = match tex_formats.iter().find(|tf| PREFERRED_FORMATS.contains(tf)) {
			Some(tf) => tf,
			None => {
				return Err(Box::new(Error::NoSurfaceFormat));
			}
		};

		let srf_cfg = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: *srf_format,
			width: window_size.width,
			height: window_size.height,
			present_mode: wgpu::PresentMode::Fifo,
		};
		surface.configure(&device, &srf_cfg);

		let egui = EguiCore {
			state: egui_winit::State::new(event_loop),
			context: egui::Context::default(),
			render_pass: EguiRenderPass::new(&device, *srf_format, 1),
		};

		Ok(GraphicsCore {
			window,
			window_size,
			instance,
			surface,
			adapter,
			device,
			queue,
			surface_config: srf_cfg,
			pipelines: Vec::<wgpu::RenderPipeline>::default(),
			staging: StagingBelt::new(0x100),
			last_frame_time: Instant::now(),
			egui,
		})
	}

	pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		if new_size.width > 0 && new_size.height > 0 {
			self.window_size = new_size;
			self.surface_config.width = new_size.width;
			self.surface_config.height = new_size.height;
			self.surface.configure(&self.device, &self.surface_config);
		}
	}

	pub fn render_start(&mut self) -> Result<Frame, wgpu::SurfaceError> {
		let now = Instant::now();
		let delta_time = now - self.last_frame_time;
		self.last_frame_time = now;

		let srftex = self.surface.get_current_texture()?;

		let view = srftex
			.texture
			.create_view(&TextureViewDescriptor::default());

		let encoder = self
			.device
			.create_command_encoder(&CommandEncoderDescriptor {
				label: Some("IMPURE: Render Encoder"),
			});

		Ok(Frame {
			delta_time,
			texture: srftex,
			view,
			encoder,
		})
	}

	pub fn egui_start(&mut self) {
		let input = self.egui.state.take_egui_input(&self.window);
		self.egui.context.begin_frame(input);
	}

	pub fn render_finish(&mut self, frame: Frame) {
		let output = self.egui.context.end_frame();
		let paint_jobs = self.egui.context.tessellate(output.shapes);

		let Frame {
			delta_time: _,
			texture: outframe,
			view: outview,
			mut encoder,
		} = frame;

		let screen_desc = ScreenDescriptor {
			size_in_pixels: [self.surface_config.width, self.surface_config.height],
			pixels_per_point: self.window.scale_factor() as f32,
		};

		for (id, image_delta) in &output.textures_delta.set {
			self.egui
				.render_pass
				.update_texture(&self.device, &self.queue, *id, image_delta);
		}

		for id in &output.textures_delta.free {
			self.egui.render_pass.free_texture(id);
		}

		self.egui
			.render_pass
			.update_buffers(&self.device, &self.queue, &paint_jobs, &screen_desc);

		self.egui
			.render_pass
			.execute(&mut encoder, &outview, &paint_jobs, &screen_desc, None);

		self.queue.submit(iter::once(encoder.finish()));
		outframe.present();
	}

	/// Output for the `wgpudiag` console command.
	#[must_use]
	pub fn diag(&self) -> String {
		let adpinfo = self.adapter.get_info();
		let feats = self.device.features();
		let limits = self.device.limits();

		format!(
			"WGPU diagnostics: \
			\r\nBackend: {:?} \
			\r\nGPU: {} ({:?}) \
			\r\nRelevant features: \
			\r\n\tPush constants: {} \
			\r\n\tTexture binding arrays: {} \
			\r\n\tBuffer binding arrays: {} \
			\r\n\tStorage resource binding arrays: {} \
			\r\nRelevant limits: \
			\r\n\tMax. 2D texture width/height: {} \
			\r\n\tMax. texture array layers: {} \
			\r\n\tMax. bind groups: {} \
			\r\n\tMax. samplers per shader stage: {} \
			\r\n\tMax. sampled textures per shader stage: {} \
			\r\n\tMax. UBOs per shader stage: {} \
			\r\n\tMax. UBO binding size: {} \
			\r\n\tMax. storage buffers per shader stage: {} \
			\r\n\tMax. storage buf. binding size: {} \
			\r\n\tMax. VBOs: {} \
			\r\n\tMax. vertex attributes: {} \
			\r\n\tMax. VBO array stride: {} \
			\r\n\tMax. push constant size: {} \
			\r\n\tMax. inter-stage shader components: {}",
			adpinfo.backend,
			adpinfo.name,
			adpinfo.device_type,
			feats.contains(wgpu::Features::PUSH_CONSTANTS),
			feats.contains(wgpu::Features::TEXTURE_BINDING_ARRAY),
			feats.contains(wgpu::Features::BUFFER_BINDING_ARRAY),
			feats.contains(wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY),
			// TODO: max_buffer_size in wgpu 0.13
			limits.max_texture_dimension_2d,
			limits.max_texture_array_layers,
			limits.max_bind_groups,
			limits.max_samplers_per_shader_stage,
			limits.max_sampled_textures_per_shader_stage,
			limits.max_uniform_buffers_per_shader_stage,
			limits.max_uniform_buffer_binding_size,
			limits.max_storage_buffers_per_shader_stage,
			limits.max_storage_buffer_binding_size,
			limits.max_vertex_buffers,
			limits.max_vertex_attributes,
			limits.max_vertex_buffer_array_stride,
			limits.max_push_constant_size,
			limits.max_inter_stage_shader_components
		)
	}
}

pub struct Frame {
	delta_time: Duration,
	texture: SurfaceTexture,
	view: TextureView,
	encoder: CommandEncoder,
}

impl Frame {
	#[must_use]
	pub fn render_pass(&mut self, clear_color: wgpu::Color) -> RenderPass {
		self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some("IMPURE: Render Pass"),
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view: &self.view,
				resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Clear(clear_color),
					store: true,
				},
			})],
			depth_stencil_attachment: None,
		})
	}

	#[must_use]
	pub fn delta_time_secs_f32(&self) -> f32 {
		self.delta_time.as_secs_f32()
	}
}
