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

use egui_wgpu::renderer::{RenderPass as EguiRenderPass, ScreenDescriptor};
use log::info;
use std::{error::Error, fmt, iter};
use wgpu::{RenderPipeline, SurfaceConfiguration, SurfaceTexture, TextureView};
use winit::window::Window;

/// Holds all state common to rendering between scenes.
pub struct GfxCore {
	pub window: Window,
	pub window_size: winit::dpi::PhysicalSize<u32>,
	pub instance: wgpu::Instance,
	pub surface: wgpu::Surface,
	pub adapter: wgpu::Adapter,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
	pub surface_config: SurfaceConfiguration,
	pub pipelines: Vec<RenderPipeline>,
	pub egui: EguiCore,
}

pub struct EguiCore {
	pub state: egui_winit::State,
	pub context: egui::Context,
	pub render_pass: EguiRenderPass,
}

#[derive(Debug)]
struct RequestAdapterError {}
impl Error for RequestAdapterError {}
impl fmt::Display for RequestAdapterError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Failed to retrieve an adapter.")
	}
}

impl GfxCore {
	pub fn new(window: Window) -> Result<GfxCore, Box<dyn Error>> {
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
				return Err(Box::new(RequestAdapterError {}));
			}
		};

		let (device, queue) = match pollster::block_on(adapter.request_device(
			&wgpu::DeviceDescriptor {
				features: wgpu::Features::default(),
				limits: wgpu::Limits::default(),
				label: None,
			},
			None,
		)) {
			Ok(dq) => dq,
			Err(err) => {
				return Err(Box::new(err));
			}
		};

		{
			let dinfo = adapter.get_info();

			info!("WGPU backend: {:?}", dinfo.backend);
			info!("GPU: {} ({:?})", dinfo.name, dinfo.device_type);
		}

		let window_size = window.inner_size();
		let srf_format = surface.get_preferred_format(&adapter).unwrap();
		let srf_cfg = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: srf_format,
			width: window_size.width as u32,
			height: window_size.height as u32,
			present_mode: wgpu::PresentMode::Fifo,
		};
		surface.configure(&device, &srf_cfg);

		let egui = EguiCore {
			state: egui_winit::State::new(4096, &window),
			context: egui::Context::default(),
			render_pass: EguiRenderPass::new(&device, srf_format, 1),
		};

		Ok(GfxCore {
			window,
			window_size,
			instance,
			surface,
			adapter,
			device,
			queue,
			surface_config: srf_cfg,
			pipelines: Vec::<wgpu::RenderPipeline>::default(),
			egui,
		})
	}

	pub fn pipeline_from_shader(&mut self, string: String) {
		let ssrc = wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(string));

		let sdesc = wgpu::ShaderModuleDescriptor {
			label: Some("hello-tri"),
			source: ssrc,
		};

		let shader = self.device.create_shader_module(&sdesc);

		let layout = self
			.device
			.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("hello-tri"),
				bind_group_layouts: &[],
				push_constant_ranges: &[],
			});

		let pipeline = self
			.device
			.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
				label: Some("hello-tri"),
				layout: Some(&layout),
				vertex: wgpu::VertexState {
					module: &shader,
					entry_point: "vs_main",
					buffers: &[],
				},
				fragment: Some(wgpu::FragmentState {
					module: &shader,
					entry_point: "fs_main",
					targets: &[wgpu::ColorTargetState {
						format: self.surface_config.format,
						blend: Some(wgpu::BlendState::REPLACE),
						write_mask: wgpu::ColorWrites::ALL,
					}],
				}),
				primitive: wgpu::PrimitiveState {
					topology: wgpu::PrimitiveTopology::TriangleList,
					strip_index_format: None,
					front_face: wgpu::FrontFace::Ccw,
					cull_mode: Some(wgpu::Face::Back),
					polygon_mode: wgpu::PolygonMode::Fill,
					unclipped_depth: false,
					conservative: false,
				},
				depth_stencil: None,
				multisample: wgpu::MultisampleState {
					count: 1,
					mask: !0,
					alpha_to_coverage_enabled: false,
				},
				multiview: None,
			});

		self.pipelines.push(pipeline);
	}

	pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		if new_size.width > 0 && new_size.height > 0 {
			self.window_size = new_size;
			self.surface_config.width = new_size.width;
			self.surface_config.height = new_size.height;
			self.surface.configure(&self.device, &self.surface_config);
		}
	}

	pub fn render_start(&self) -> Result<(SurfaceTexture, TextureView), wgpu::SurfaceError> {
		let ret1 = self.surface.get_current_texture()?;

		let ret2 = ret1
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());

		Ok((ret1, ret2))
	}

	pub fn egui_start(&mut self) {
		let input = self.egui.state.take_egui_input(&self.window);
		self.egui.context.begin_frame(input);
	}

	pub fn render_finish(&mut self, outframe: SurfaceTexture, outview: TextureView) {
		let output = self.egui.context.end_frame();
		let paint_jobs = self.egui.context.tessellate(output.shapes);

		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor {
				label: Some("Render Encoder"),
			});

		let scr_desc = ScreenDescriptor {
			size_in_pixels: [self.surface_config.width, self.surface_config.height],
			pixels_per_point: self.window.scale_factor() as f32,
		};

		{
			let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Pass"),
				color_attachments: &[wgpu::RenderPassColorAttachment {
					view: &outview,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color {
							r: 0.0,
							g: 0.0,
							b: 0.0,
							a: 1.0,
						}),
						store: true,
					},
				}],
				depth_stencil_attachment: None,
			});

			rpass.set_pipeline(&self.pipelines[0]);
			rpass.draw(0..3, 0..1);
		}

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
			.update_buffers(&self.device, &self.queue, &paint_jobs, &scr_desc);

		self.egui
			.render_pass
			.execute(&mut encoder, &outview, &paint_jobs, &scr_desc, None);

		self.queue.submit(iter::once(encoder.finish()));
		outframe.present();
	}
}
