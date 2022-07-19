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

mod console;
mod data;
mod engine;
mod gfx;
mod lua;
mod teal;
mod utils;
mod vfs;

use crate::{
	console::{Console, ConsoleCommand, ConsoleRequest, ConsoleWriter},
	data::*,
	engine::EngineScene,
	gfx::GfxCore,
	lua::LuaImpure,
	utils::exe_dir,
	vfs::ImpureVfs,
};
use log::{error, info, warn};
use mlua::prelude::*;
use parking_lot::RwLock;
use physfs_rs::*;
use std::{
	boxed::Box,
	env,
	error::Error,
	fs, io,
	path::{Path, PathBuf},
	sync::Arc,
};
use winit::{
	dpi::PhysicalSize,
	event::{Event as WinitEvent, VirtualKeyCode, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
};

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
						"Failed to convert `systeminfo | findstr /C:\"OS\"`
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

	let (cons_sender, cons_receiver) = crossbeam::channel::unbounded();
	let mut console = Console::new(cons_receiver);

	// Logging initialisation

	{
		let colors = fern::colors::ColoredLevelConfig::new()
			.info(fern::colors::Color::Green)
			.warn(fern::colors::Color::Yellow)
			.error(fern::colors::Color::Red)
			.debug(fern::colors::Color::Cyan)
			.trace(fern::colors::Color::Magenta);

		let file_cfg = fern::Dispatch::new()
			.format(|out, message, record| {
				out.finish(format_args!(
					"{}[{}][{}] {}",
					chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
					record.target(),
					record.level(),
					message
				))
			})
			.chain(
				fs::OpenOptions::new()
					.write(true)
					.create(true)
					.truncate(true)
					.open("impure.log")?,
			);

		// Stdout logging has console colouring and less date-time elaboration
		let stdout_cfg = fern::Dispatch::new()
			.format(move |out, message, record| {
				out.finish(format_args!(
					"{}[{}][{}] {}",
					chrono::Local::now().format("[%H:%M:%S]"),
					record.target(),
					colors.color(record.level()),
					message
				))
			})
			.chain(io::stdout());

		let console_cfg = fern::Dispatch::new()
			.format(move |out, message, record| {
				out.finish(format_args!("[{}] {}", record.level(), message))
			})
			.chain(Box::new(ConsoleWriter::new(cons_sender)) as Box<dyn io::Write + Send>);

		let logres = fern::Dispatch::new()
			.level(log::LevelFilter::Warn)
			.level_for("impure", log::LevelFilter::Debug)
			.level_for("wgpu_hal", log::LevelFilter::Error)
			.level_for("wgpu_core", log::LevelFilter::Error)
			.chain(console_cfg)
			.chain(file_cfg)
			.chain(stdout_cfg)
			.apply();

		if let Err(err) = logres {
			return Err(Box::new(err));
		}
	}

	info!("Impure Engine version {}.", env!("CARGO_PKG_VERSION"));

	print_os_info();

	for arg in env::args() {
		if arg == "--version" || arg == "-v" {
			return Ok(());
		}
	}

	let vfs = Arc::new(RwLock::new(
		PhysFs::get().expect("PhysicsFS was unexpectedly re-initialised."),
	));

	let lua = match Lua::new_ex(true, vfs.clone()) {
		Ok(l) => l,
		Err(err) => {
			error!("Failed to initialise client Lua state: {}", err);
			return Err(Box::new(err));
		}
	};

	// Mount contents of '/exe_dir/gamedata'

	let gdata_path: PathBuf = [exe_dir(), PathBuf::from("gamedata")]
		.iter()
		.collect();

	mount_gamedata(&mut vfs.write(), &lua, gdata_path);

	// If neither '/exe_dir/gamedata/impure' nor '/exe_dir/gamedata/impure.zip'
	// were found in the previous step, check the PWD (but only on the dev build)

	#[cfg(debug_assertions)]
	{
		let p: PathBuf = [env::current_dir()?, PathBuf::from("gamedata")]
			.iter()
			.collect();

		mount_gamedata(&mut vfs.write(), &lua, p);
	}

	// If there still isn't a valid '/impure' directory in the VFS search path,
	// the engine's data is missing or malformed, and we can't continue

	if !vfs.read().is_directory("/impure") {
		return Err(Box::<io::Error>::new(io::ErrorKind::NotFound.into()));
	}

	// Mount userdata directory

	mount_userdata(&mut vfs.write())?;

	let icon = vfs
		.read()
		.window_icon_from_file(Path::new("/impure/impure.png"));
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
			.with_window_icon(icon)
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

	// Register console commands

	console.register_command(ConsoleCommand::new(
		"version",
		|_, _| {
			println!(
				"Impure engine version: {}.{}.{} (commit {}). Compiled on: {}",
				env!("CARGO_PKG_VERSION_MAJOR"),
				env!("CARGO_PKG_VERSION_MINOR"),
				env!("CARGO_PKG_VERSION_PATCH"),
				env!("GIT_HASH"),
				env!("COMPILE_DATETIME")
			);

			ConsoleRequest::None
		},
		|_, _| {
			println!("Prints the engine version.");
		},
	));

	console.register_command(ConsoleCommand::new(
		"home",
		|_, _| {
			match get_userdata_path() {
				Some(p) => info!("{}", p.display()),
				None => {
					info!(
						"Home directory path is malformed, 
						or this platform is unsupported."
					);
				}
			}

			ConsoleRequest::None
		},
		|_, _| {
			info!("Prints the directory used to store userdata.");
		},
	));

	console.register_command(ConsoleCommand::new(
		"file",
		|_, args| {
			let path = if args.is_empty() { "/" } else { args[0] };
			ConsoleRequest::File(PathBuf::from(path))
		},
		|_, _| {
			info!("Prints the contents of a virtual file system directory.");
		},
	));

	let _scene = EngineScene::Frontend;

	let stc = start_time;

	let on_close = move || {
		info!("Runtime duration (s): {}", stc.elapsed().as_secs());
	};

	event_loop.run(move |event, _, control_flow| match event {
		WinitEvent::RedrawRequested(window_id) if window_id == gfx.window.id() => {
			let output = match gfx.render_start() {
				Ok(o) => o,
				Err(wgpu::SurfaceError::Lost) => {
					gfx.resize(gfx.window_size);
					return;
				}
				Err(wgpu::SurfaceError::OutOfMemory) => {
					error!("Insufficient memory to allocate a new WGPU frame.");
					*control_flow = ControlFlow::Exit;
					return;
				}
				Err(err) => {
					error!("${:?}", err);
					return;
				}
			};

			gfx.egui_start();
			console.draw(&gfx.egui.context);
			gfx.render_finish(output.0, output.1);
		}
		WinitEvent::MainEventsCleared => {
			let cons_reqs = console.requests.drain(..);

			for req in cons_reqs {
				match req {
					ConsoleRequest::File(p) => {
						let vfsg = vfs.read();

						if !vfsg.is_directory(p.as_os_str().to_string_lossy()) {
							info!("{} is not a directory.", p.display());
							continue;
						}

						let files = match vfsg.enumerate_files(p.as_os_str().to_string_lossy()) {
							Some(f) => f,
							None => {
								warn!("{} can not be enumerated.", p.display());
								continue;
							}
						};

						let mut output = String::with_capacity(files.len() * 32);

						for f in files {
							output.push('\r');
							output.push('\n');
							output.push('\t');
							output = output + &f;

							let fullpath: PathBuf = [p.clone(), PathBuf::from(f)].iter().collect();

							if vfsg.is_directory(fullpath.to_str().unwrap_or_default()) {
								output.push('/');
							}
						}

						info!("Files under '{}': {}", p.display(), output);
					}
					_ => {}
				}
			}

			gfx.window.request_redraw();
		}
		WinitEvent::WindowEvent {
			ref event,
			window_id,
		} if window_id == gfx.window.id() => {
			gfx.egui.state.on_event(&gfx.egui.context, event);

			match event {
				WindowEvent::KeyboardInput { input, .. } => {
					if input.state == winit::event::ElementState::Pressed
						&& input.virtual_keycode == Some(VirtualKeyCode::Escape)
					{
						on_close();
						*control_flow = ControlFlow::Exit;
						return;
					}

					console.on_key_event(input);
				}
				WindowEvent::CloseRequested => {
					on_close();
					*control_flow = ControlFlow::Exit;
				}
				WindowEvent::Resized(psize) => {
					gfx.resize(*psize);
				}
				WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
					gfx.resize(**new_inner_size);
				}
				_ => {}
			}
		}
		_ => {}
	});
}
