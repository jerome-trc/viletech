//! # VileTech Client

mod commands;
#[allow(dead_code)]
mod core;

use std::{boxed::Box, env, error::Error};

use log::{error, info, warn};
use vile::{
	console::Console,
	data::{Catalog, CatalogExt},
	gfx::{core::GraphicsCore, render},
};
use winit::{
	dpi::PhysicalSize,
	event::{Event as WinitEvent, VirtualKeyCode, WindowEvent},
	event_loop::EventLoop,
};

use crate::core::ClientCore;

#[must_use]
pub fn version_string() -> String {
	format!("VileTech client version: {}", env!("CARGO_PKG_VERSION"))
}

fn main() -> Result<(), Box<dyn Error>> {
	let start_time = std::time::Instant::now();

	for arg in env::args() {
		if arg == "--version" || arg == "-v" {
			println!("{}", vile::short_version_string());
			println!("VileTech client version {}.", env!("CARGO_PKG_VERSION"));
			return Ok(());
		}
	}

	let (log_sender, log_receiver) = crossbeam::channel::unbounded();

	match vile::log_init(Some(log_sender)) {
		Ok(()) => {}
		Err(err) => {
			eprintln!("Failed to initialise logging backend: {}", err);
			return Err(err);
		}
	}

	let console = Console::new(log_receiver);

	vile::log_init_diag(&version_string())?;

	let _devmode = env::args().any(|arg| arg == "-d" || arg == "--dev");

	let mut catalog = Catalog::default();

	if let Err(err) = catalog.mount_basedata() {
		error!("Failed to find and mount engine base data: {err}");
		return Err(err);
	}

	let event_loop = EventLoop::new();

	let window = match winit::window::WindowBuilder::new()
		.with_title("VileTech")
		.with_min_inner_size(PhysicalSize::new(320, 200))
		.with_max_inner_size(PhysicalSize::new(7680, 4320))
		.with_inner_size(PhysicalSize::new(800, 600))
		.with_decorations(true)
		.with_resizable(true)
		.with_transparent(false)
		.with_window_icon(
			catalog
				.window_icon_from_file("/viletech/viletech.png")
				.map_err(|err| {
					warn!("Failed to load engine's window icon: {err}");
					err
				})
				.ok(),
		)
		.build(&event_loop)
	{
		Ok(w) => w,
		Err(err) => {
			return Err(Box::new(err));
		}
	};

	let mut gfx = match GraphicsCore::new(window, &event_loop) {
		Ok(g) => g,
		Err(err) => {
			error!("Graphics subsystem initialisation failed: {}", err);
			return Err(err);
		}
	};

	let shader = vile::gfx::create_shader_module(
		&gfx.device,
		"hello-tri",
		catalog
			.get_file("/viletech/shaders/hello-tri.wgsl")
			.expect("Engine base data validity check is compromised.")
			.try_read_str()?,
	);

	let pipeline = render::pipeline_builder("Hello Triangle", &gfx.device)
		.shader_states(vile::gfx::create_shader_states(
			&shader,
			"vs_main",
			&[],
			"fs_main",
			&[Some(wgpu::ColorTargetState {
				format: gfx.surface_config.format,
				blend: Some(wgpu::BlendState::REPLACE),
				write_mask: wgpu::ColorWrites::ALL,
			})],
		))
		// Remaining defaults are acceptable
		.build();

	gfx.pipelines.push(pipeline);

	let mut core = ClientCore::new(start_time, catalog, gfx, console)?;

	event_loop.run(move |event, _, control_flow| match event {
		WinitEvent::RedrawRequested(window_id) => {
			core.redraw_requested(window_id, control_flow);
		}
		WinitEvent::MainEventsCleared => {
			core.process_console_requests();
			core.scene_change(control_flow);
			core.audio.update();
			core.gfx.window.request_redraw();
		}
		WinitEvent::WindowEvent {
			ref event,
			window_id,
		} if window_id == core.gfx.window.id() => {
			let resp = core.gfx.egui.state.on_event(&core.gfx.egui.context, event);

			match event {
				WindowEvent::CloseRequested => {
					core.exit();
					core.scene_change(control_flow);
				}
				WindowEvent::Resized(psize) => {
					core.on_window_resize(*psize);
				}
				WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
					core.on_window_resize(**new_inner_size);
				}
				WindowEvent::Focused(gained) => {
					if *gained {
						core.audio.resume_all();
					} else {
						core.audio.pause_all();
					}
				}
				_ => {}
			}

			if resp.consumed {
				return;
			}

			match event {
				WindowEvent::KeyboardInput { input, .. } => {
					if input.state == winit::event::ElementState::Pressed
						&& input.virtual_keycode == Some(VirtualKeyCode::Escape)
					{
						core.exit();
						core.scene_change(control_flow);
						return;
					}

					core.on_key_event(input);
				}
				WindowEvent::MouseInput { state, button, .. } => {
					core.input.on_mouse_input(button, state);
				}
				WindowEvent::ModifiersChanged(state) => {
					core.input.on_modifiers_changed(state);
				}
				WindowEvent::CursorMoved { position, .. } => {
					core.input.on_cursor_moved(position);
				}
				_ => {}
			}
		}
		WinitEvent::Suspended => {
			info!("Application suspended...");
		}
		WinitEvent::Resumed => {
			info!("Application resumed...");
		}
		WinitEvent::LoopDestroyed => {
			info!("{}", vile::uptime_string(core.start_time));
		}
		_ => {}
	});
}
