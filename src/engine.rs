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

use crate::{
	console::{Console, ConsoleCommand, ConsoleRequest},
	data::DataCore,
	gfx::GfxCore,
	utils::*,
	vfs::VirtualFs,
};
use log::{error, info};
use mlua::Lua;
use nanorand::WyRand;
use parking_lot::RwLock;
use shipyard::World;
use std::{env, path::PathBuf, sync::Arc, thread::Thread};
use winit::{event::KeyboardInput, event_loop::ControlFlow, window::WindowId};

pub struct Playsim {
	rng: WyRand,
	world: World,
}

pub enum EngineScene {
	Frontend,
	Title {
		gui: World,
	},
	Intermission {
		gui: World,
	},
	Text {
		gui: World,
	},
	PlaysimSingle {
		thread: Thread,
		gui: World,
		playsim: RwLock<Playsim>,
	},
	Demo {
		gui: World,
		playsim: RwLock<Playsim>,
	},
	CastCall,
	Cutscene,
}

pub struct Engine {
	pub start_time: std::time::Instant,
	pub vfs: Arc<RwLock<VirtualFs>>,
	pub lua: Lua,
	pub data: DataCore,
	pub gfx: GfxCore,
	pub console: Console,
	pub scene: EngineScene,
}

impl Engine {
	pub fn new(
		start_time: std::time::Instant,
		vfs: Arc<RwLock<VirtualFs>>,
		lua: Lua,
		data: DataCore,
		gfx: GfxCore,
		console: Console,
	) -> Self {
		let mut ret = Engine {
			start_time,
			vfs,
			lua,
			data,
			gfx,
			console,
			scene: EngineScene::Frontend,
		};

		ret.console.register_command(ConsoleCommand::new(
			"version",
			|_, _| {
				info!(
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
				info!("Prints the engine version.");
			},
			true,
		));

		ret.console.register_command(ConsoleCommand::new(
			"uptime",
			|_, _| ConsoleRequest::Uptime,
			|_, _| {
				info!("Prints the length of the time the engine has been running.");
			},
			true,
		));

		ret.console.register_command(ConsoleCommand::new(
			"home",
			|_, _| {
				match get_user_dir() {
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
			false,
		));

		ret.console.register_command(ConsoleCommand::new(
			"file",
			|_, args| {
				let path = if args.is_empty() { "/" } else { args[0] };
				ConsoleRequest::File(PathBuf::from(path))
			},
			|_, _| {
				info!("Prints the contents of a virtual file system directory.");
			},
			true,
		));

		ret.console.register_command(ConsoleCommand::new(
			"args",
			|_, _| {
				let mut args = env::args();

				let argv0 = match args.next() {
					Some(a) => a,
					None => {
						error!("This runtime did not receive `argv[0]`.");
						return ConsoleRequest::None;
					}
				};

				let mut output = argv0;

				for arg in args {
					output.push('\r');
					output.push('\n');
					output.push('\t');
					output += &arg;
				}

				info!("{}", output);

				ConsoleRequest::None
			},
			|_, _| info!("Prints out all of the program's launch arguments."),
			false,
		));

		ret
	}

	pub fn print_uptime(&self) {
		let elapsed = self.start_time.elapsed();
		let dur = chrono::Duration::from_std(elapsed).unwrap();
		let secs = dur.num_seconds();
		let mins = secs / 60;
		let hours = mins / 60;
		info!("Uptime: {:02}:{:02}:{:02}", hours, mins % 60, secs % 60);
	}

	pub fn redraw_requested(&mut self, window_id: WindowId, control_flow: &mut ControlFlow) {
		if window_id != self.gfx.window.id() {
			return;
		}

		let output = match self.gfx.render_start() {
			Ok(o) => o,
			Err(wgpu::SurfaceError::Lost) => {
				self.gfx.resize(self.gfx.window_size);
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

		self.gfx.egui_start();
		self.console.draw(&self.gfx.egui.context);
		self.gfx.render_finish(output.0, output.1);
	}

	pub fn process_console_requests(&mut self) {
		let cons_reqs: Vec<_> = self.console.requests.drain(..).collect();

		for req in cons_reqs {
			match req {
				ConsoleRequest::File(p) => {
					let vfsg = self.vfs.read();

					if !vfsg.is_dir(&p) {
						info!("\"{}\" is not a directory.", p.display());
						continue;
					}

					let files = vfsg.file_names(&p);

					let mut output = String::with_capacity(files.len() * 32);

					for f in &files {
						output.push('\r');
						output.push('\n');
						output.push('\t');
						output = output + f;

						// TODO: Bespoke VFS may allow this to be optimised
						let fullpath: PathBuf = [p.clone(), PathBuf::from(f)].iter().collect();

						if vfsg.is_dir(fullpath.to_str().unwrap_or_default()) {
							output.push('/');
						}
					}

					info!(
						"Files under \"{}\" ({}): {}",
						p.display(),
						files.len(),
						output
					);
				}
				ConsoleRequest::Uptime => {
					self.print_uptime();
				}
				_ => {}
			}
		}
	}

	pub fn on_key_event(&mut self, input: &KeyboardInput) {
		self.console.on_key_event(input);
	}
}
