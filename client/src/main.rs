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

mod commands;
#[allow(dead_code)]
mod core;

use std::{boxed::Box, env, error::Error, path::Path, sync::Arc};

use impure::{
	console::Console,
	data::DataCore,
	depends::*,
	gfx::{core::GraphicsCore, render},
	lua::ImpureLua,
	vfs::{ImpureVfs, VirtualFs},
};
use log::{error, info};
use mlua::prelude::*;
use parking_lot::RwLock;
use winit::{
	dpi::PhysicalSize,
	event::{Event as WinitEvent, VirtualKeyCode, WindowEvent},
	event_loop::EventLoop,
};

use crate::core::ClientCore;

fn main() -> Result<(), Box<dyn Error>> {
	let start_time = std::time::Instant::now();

	for arg in env::args() {
		if arg == "--version" || arg == "-v" {
			println!("{}", impure::short_version_string());
			println!("Impure client version {}.", env!("CARGO_PKG_VERSION"));
			return Ok(());
		}
	}

	let (cons_sender, cons_receiver) = crossbeam::channel::unbounded();

	match impure::log_init(Some(cons_sender)) {
		Ok(()) => {}
		Err(err) => {
			eprintln!("Failed to initialise logging backend: {}", err);
			return Err(err);
		}
	}

	let console = Console::new(cons_receiver);

	info!("{}", impure::short_version_string());
	info!("Impure client version {}.", env!("CARGO_PKG_VERSION"));
	info!("{}", impure::utils::env::os_info()?);

	let devmode = env::args().any(|arg| arg == "-d" || arg == "--dev");

	let lua = match Lua::new_ex(!devmode) {
		Ok(l) => l,
		Err(err) => {
			error!("Failed to initialise client Lua state: {}", err);
			return Err(Box::new(err));
		}
	};

	let data = DataCore::default();
	let vfs = Arc::new(RwLock::new(VirtualFs::default()));

	match vfs.write().mount_enginedata() {
		Ok(()) => {}
		Err(err) => {
			error!(
				"Failed to find and mount engine gamedata.
				Is 'impure.zip' missing?
				Error: {}",
				err
			);
			return Err(Box::new(err));
		}
	};

	match lua.init_api_common(vfs.clone()) {
		Ok(()) => {}
		Err(err) => {
			error!("Failed to initialise Lua global state: {}", err);
			return Err(Box::new(err));
		}
	};

	let event_loop = EventLoop::new();

	let window = match winit::window::WindowBuilder::new()
		.with_title("Impure")
		.with_min_inner_size(PhysicalSize::new(320, 200))
		.with_max_inner_size(PhysicalSize::new(7680, 4320))
		.with_inner_size(PhysicalSize::new(800, 600))
		.with_decorations(true)
		.with_resizable(true)
		.with_transparent(false)
		.with_window_icon(
			vfs.read()
				.window_icon_from_file(Path::new("/impure/impure.png")),
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

	let shader = impure::gfx::create_shader_module(
		&gfx.device,
		"hello-tri",
		vfs.read().read_str("/impure/shaders/hello-tri.wgsl")?,
	);

	let pipeline = render::pipeline_builder("Hello Triangle", &gfx.device)
		.shader_states(impure::gfx::create_shader_states(
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

	let mut core = ClientCore::new(start_time, vfs, lua, data, gfx, console)?;

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
			core.gfx.egui.state.on_event(&core.gfx.egui.context, event);

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
				WindowEvent::Focused(gained) => {
					if *gained {
						core.audio.resume_all();
					} else {
						core.audio.pause_all();
					}
				}
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
			info!("{}", impure::uptime_string(core.start_time));
		}
		_ => {}
	});
}
