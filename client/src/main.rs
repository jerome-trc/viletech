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

#[allow(dead_code)]
mod core;

use impure::{
	console::Console,
	data::DataCore,
	depends::*,
	gfx::GfxCore,
	lua::ImpureLua,
	vfs::{ImpureVfs, VirtualFs},
};
use log::{error, info};
use mlua::prelude::*;
use parking_lot::RwLock;
use std::{boxed::Box, env, error::Error, path::Path, sync::Arc};
use winit::{
	dpi::PhysicalSize,
	event::{Event as WinitEvent, VirtualKeyCode, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
};

use crate::core::ClientCore;

fn print_os_info() {
	type Command = std::process::Command;

	match env::consts::OS {
		"linux" => {
			let uname = Command::new("uname").args(&["-s", "-r", "-v"]).output();

			let output = match uname {
				Ok(o) => o,
				Err(err) => {
					error!("Failed to execute `uname -s -r -v`: {}", err);
					return;
				}
			};

			let osinfo = match String::from_utf8(output.stdout) {
				Ok(s) => s.replace('\n', ""),
				Err(err) => {
					error!(
						"Failed to convert `uname -s -r -v` output to UTF-8: {}",
						err
					);
					return;
				}
			};

			info!("{}", osinfo);
		}
		"windows" => {
			let systeminfo = Command::new("systeminfo | findstr")
				.args(&["/C:\"OS\""])
				.output();

			let output = match systeminfo {
				Ok(o) => o,
				Err(err) => {
					error!(
						"Failed to execute `systeminfo | findstr /C:\"OS\"`: {}",
						err
					);
					return;
				}
			};

			let osinfo = match String::from_utf8(output.stdout) {
				Ok(s) => s,
				Err(err) => {
					error!(
						"Failed to convert `systeminfo | findstr /C:\"OS\"` \
						 output to UTF-8: {}",
						err
					);
					return;
				}
			};

			info!("{}", osinfo);
		}
		_ => {}
	}
}

fn main() -> Result<(), Box<dyn Error>> {
	let start_time = std::time::Instant::now();

	for arg in env::args() {
		if arg == "--version" || arg == "-v" {
			println!("Impure Engine version {}.", env!("CARGO_PKG_VERSION"));
			return Ok(());
		}
	}

	let (cons_sender, cons_receiver) = crossbeam::channel::unbounded();

	match impure::log_init(cons_sender) {
		Ok(()) => {}
		Err(err) => {
			eprintln!("Failed to initialise logging backend: {}", err);
			return Err(err);
		}
	}

	let console = Console::new(cons_receiver);

	info!("Impure Engine version {}.", env!("CARGO_PKG_VERSION"));

	print_os_info();

	let data = DataCore::default();
	let vfs = Arc::new(RwLock::new(VirtualFs::default()));

	match vfs.write().mount_enginedata() {
		Ok(()) => {}
		Err(err) => {
			error!("Failed to find engine gamedata. Is impure.zip missing?");
			return Err(Box::new(err));
		}
	};

	let lua = match Lua::new_ex(true, vfs.clone()) {
		Ok(l) => l,
		Err(err) => {
			error!("Failed to initialise client Lua state: {}", err);
			return Err(Box::new(err));
		}
	};

	let event_loop = EventLoop::new();

	let mut gfx = match GfxCore::new(
		match winit::window::WindowBuilder::new()
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
		},
	) {
		Ok(g) => g,
		Err(err) => {
			error!("Graphics subsystem initialisation failed: {}", err);
			return Err(err);
		}
	};

	gfx.pipeline_from_shader(
		vfs.read()
			.read_string(Path::new("/impure/shaders/hello-tri.wgsl"))?,
	);

	let mut core = ClientCore::new(start_time, vfs, lua, data, gfx, console)?;

	event_loop.run(move |event, _, control_flow| match event {
		WinitEvent::RedrawRequested(window_id) => {
			core.redraw_requested(window_id, control_flow);
		}
		WinitEvent::MainEventsCleared => {
			core.process_console_requests();
			core.clear_stopped_sounds();
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
						core.print_uptime();
						*control_flow = ControlFlow::Exit;
						return;
					}

					core.on_key_event(input);
				}
				WindowEvent::CloseRequested => {
					core.print_uptime();
					*control_flow = ControlFlow::Exit;
				}
				WindowEvent::Resized(psize) => {
					core.gfx.resize(*psize);
				}
				WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
					core.gfx.resize(**new_inner_size);
				}
				_ => {}
			}
		}
		_ => {}
	});
}
