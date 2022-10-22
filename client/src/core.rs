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

use impure::{
	audio::{self, AudioCore},
	console::Console,
	data::game::DataCore,
	depends::{parking_lot::Mutex, winit::event::ElementState, *},
	frontend::{FrontendAction, FrontendMenu},
	gfx::{camera::Camera, core::GraphicsCore},
	input::InputCore,
	rng::RngCore,
	sim::{self, PlaySim},
	terminal,
	vfs::{ImpureVfs, VirtualFs},
};

use kira::{
	manager::{AudioManager, AudioManagerSettings},
	sound::static_sound::StaticSoundSettings,
};
use log::{error, info};
use mlua::prelude::*;
use nanorand::WyRand;
use parking_lot::RwLock;
use shipyard::World;
use std::{error::Error, path::PathBuf, sync::Arc, thread::JoinHandle};
use winit::{event::KeyboardInput, event_loop::ControlFlow, window::WindowId};

use crate::commands::{
	self, Command as ConsoleCommand, CommandFlags as ConsoleCommandFlags, Request as ConsoleRequest,
};

enum Scene {
	/// The user hasn't entered the game yet. From here they can select a user
	/// profile and engine-global/cross-game preferences, assemble a load order,
	/// and begin the game launch process.
	Frontend {
		menu: FrontendMenu,
	},
	/// Where the user is taken after leaving the frontend, unless they have
	/// specified to be taken directly to a playsim.
	Title {
		playsim: Arc<RwLock<PlaySim>>,
	},
	Playsim {
		sender: sim::InSender,
		receiver: sim::OutReceiver,
		playsim: Arc<RwLock<PlaySim>>,
		thread: JoinHandle<()>,
	},
	CastCall,
}

enum SceneChange {
	Exit,
	Title { to_mount: Vec<PathBuf> },
}

pub struct ClientCore {
	pub start_time: std::time::Instant,
	pub vfs: Arc<RwLock<VirtualFs>>,
	pub lua: Arc<Mutex<Lua>>,
	pub data: Arc<RwLock<DataCore>>,
	pub rng: RngCore<WyRand>,
	pub gfx: GraphicsCore,
	pub audio: AudioCore,
	pub input: InputCore,
	pub console: Console<ConsoleCommand>,
	pub gui: World,
	pub camera: Camera,
	scene: Scene,
	next_scene: Option<SceneChange>,
}

// Public interface.
impl ClientCore {
	pub fn new(
		start_time: std::time::Instant,
		vfs: Arc<RwLock<VirtualFs>>,
		lua: Lua,
		data: DataCore,
		gfx: GraphicsCore,
		console: Console<ConsoleCommand>,
	) -> Result<Self, Box<dyn Error>> {
		let audio_mgr_settings = AudioManagerSettings::default();
		let sound_cap = audio_mgr_settings.capacities.sound_capacity;

		let audio = AudioCore {
			manager: AudioManager::new(audio_mgr_settings)?,
			music1: None,
			music2: None,
			sounds: Vec::<_>::with_capacity(sound_cap),
		};

		let camera = Camera::new(
			gfx.surface_config.width as f32,
			gfx.surface_config.height as f32,
		);

		let mut ret = ClientCore {
			start_time,
			vfs,
			lua: Arc::new(Mutex::new(lua)),
			data: Arc::new(RwLock::new(data)),
			gfx,
			rng: RngCore::default(),
			audio,
			input: InputCore::default(),
			console,
			gui: World::default(),
			camera,
			scene: Scene::Frontend {
				menu: FrontendMenu::default(),
			},
			next_scene: None,
		};

		ret.register_console_commands();
		impure::user::build_user_dirs()?;

		Ok(ret)
	}

	pub fn redraw_requested(&mut self, window_id: WindowId, control_flow: &mut ControlFlow) {
		if window_id != self.gfx.window.id() {
			return;
		}

		let mut frame = match self.gfx.render_start() {
			Ok(f) => f,
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

		// Temporary discard
		let _ = self.camera.update(frame.delta_time_secs_f32());
		self.gfx.egui_start();

		match &mut self.scene {
			Scene::Frontend { menu } => {
				let action = menu.ui(&self.gfx.egui.context);

				match action {
					FrontendAction::None => {}
					FrontendAction::Quit => {
						*control_flow = ControlFlow::Exit;
					}
					FrontendAction::Start => {
						let to_mount = menu.to_mount();
						let to_mount = to_mount.into_iter().map(|p| p.to_path_buf()).collect();
						self.next_scene = Some(SceneChange::Title { to_mount });
					}
				}

				let clear_color = if self.gfx.egui.context.style().visuals.dark_mode {
					wgpu::Color {
						r: 0.0,
						g: 0.0,
						b: 0.0,
						a: 1.0,
					}
				} else {
					wgpu::Color {
						r: 0.9,
						g: 0.9,
						b: 0.9,
						a: 1.0,
					}
				};

				let mut rpass = frame.render_pass(clear_color);

				rpass.set_pipeline(&self.gfx.pipelines[0]);
				rpass.draw(0..3, 0..1);
			}
			Scene::Title { .. } => {
				// ???
			}
			Scene::Playsim { .. } => {}
			_ => {}
		};

		self.console.ui(&self.gfx.egui.context);
		self.gfx.render_finish(frame);
	}

	pub fn process_console_requests(&mut self) {
		while !self.console.requests.is_empty() {
			match self.console.requests.pop_front().unwrap() {
				ConsoleRequest::None => {}
				ConsoleRequest::EchoAllCommands => {
					let mut string = "All available commands:".to_string();

					for command in self.console.all_commands() {
						string.push('\r');
						string.push('\n');
						string.push_str(command.0);
					}

					info!("{}", string);
				}
				ConsoleRequest::CommandHelp(key) => match self.console.find_command(&key) {
					Some(cmd) => {
						(cmd.func)(terminal::CommandArgs(vec![&key, "--help"]));
					}
					None => {
						info!("No command found by name: {}", key);
					}
				},
				ConsoleRequest::Exit => {
					self.next_scene = Some(SceneChange::Exit);
					return;
				}
				ConsoleRequest::CreateAlias(alias, string) => {
					info!("Alias registered: {}\r\nExpands to: {}", &alias, &string);
					self.console.register_alias(alias, string);
				}
				ConsoleRequest::EchoAlias(alias) => match self.console.find_alias(&alias) {
					Some(a) => {
						info!("{}", a.1);
					}
					None => {
						info!("No existing alias: {}", alias);
					}
				},
				ConsoleRequest::Callback(func) => {
					(func)(self);
				}
				ConsoleRequest::File(p) => {
					let vfsg = self.vfs.read();
					info!("{}", vfsg.ccmd_file(p));
				}
				ConsoleRequest::Sound(arg) => {
					let vfsg = self.vfs.read();

					let handle = match vfsg.lookup(&arg) {
						Some(h) => h,
						None => {
							info!("No file under virtual path: {}", arg);
							continue;
						}
					};

					let sdat = match audio::sound_from_file(handle, StaticSoundSettings::default())
					{
						Ok(ssd) => ssd,
						Err(err) => {
							info!("Failed to create sound from file: {}", err);
							continue;
						}
					};

					match self.audio.play_global(sdat) {
						Ok(()) => {}
						Err(err) => {
							info!("Failed to play sound: {}", err);
							continue;
						}
					};
				}
			}
		}
	}

	pub fn on_window_resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		self.gfx.resize(new_size);
		self.camera
			.resize(new_size.width as f32, new_size.height as f32);
	}

	pub fn on_key_event(&mut self, event: &KeyboardInput) {
		self.console.on_key_event(event);
		self.input.on_key_event(event);

		if event.virtual_keycode.is_none() {
			return;
		}

		let vkc = event.virtual_keycode.unwrap();
		let binds = self.input.user_binds.iter().filter(|kb| kb.keycode == vkc);
		let lua = self.lua.lock();

		if event.state == ElementState::Pressed {
			for bind in binds {
				let func: LuaFunction = lua.registry_value(&bind.on_press).unwrap();

				match func.call(()) {
					Ok(()) => {}
					Err(err) => {
						error!("Error in key action `{}`: {}", bind.id, err);
					}
				};
			}
		} else {
			for bind in binds {
				let func: LuaFunction = lua.registry_value(&bind.on_release).unwrap();

				match func.call(()) {
					Ok(()) => {}
					Err(err) => {
						error!("Error in key action `{}`: {}", bind.id, err);
					}
				};
			}
		}
	}

	pub fn scene_change(&mut self, control_flow: &mut ControlFlow) {
		let next_scene = self.next_scene.take();

		match next_scene {
			None => {
				// TODO: Mark as likely with intrinsics...?
			}
			Some(scene) => match scene {
				SceneChange::Exit => {
					info!("{}", impure::uptime_string(self.start_time));
					*control_flow = ControlFlow::Exit;
				}
				SceneChange::Title { to_mount } => {
					let mut metas = vec![self
						.vfs
						.read()
						.parse_gamedata_meta("/impure/meta.toml")
						.expect("Engine data package manifest is malformed.")];

					if !to_mount.is_empty() {
						let mut m = self.vfs.write().mount_gamedata(&to_mount);
						metas.append(&mut m);
					}

					self.data.write().populate(metas, &self.vfs.read());
					self.start_game();
				}
			},
		};
	}
}

// Internal implementation details: general.
impl ClientCore {
	fn register_console_commands(&mut self) {
		self.console.register_command(
			"alias",
			ConsoleCommand {
				flags: ConsoleCommandFlags::all(),
				func: commands::ccmd_alias,
			},
			true,
		);

		self.console.register_command(
			"args",
			ConsoleCommand {
				flags: ConsoleCommandFlags::all(),
				func: commands::ccmd_args,
			},
			true,
		);

		self.console.register_command(
			"clear",
			ConsoleCommand {
				flags: ConsoleCommandFlags::all(),
				func: commands::ccmd_clear,
			},
			true,
		);

		self.console.register_command(
			"exit",
			ConsoleCommand {
				flags: ConsoleCommandFlags::all(),
				func: commands::ccmd_exit,
			},
			true,
		);

		self.console.register_command(
			"file",
			ConsoleCommand {
				flags: ConsoleCommandFlags::all(),
				func: commands::ccmd_file,
			},
			true,
		);

		self.console.register_command(
			"hclear",
			ConsoleCommand {
				flags: ConsoleCommandFlags::all(),
				func: commands::ccmd_hclear,
			},
			true,
		);

		self.console.register_command(
			"help",
			ConsoleCommand {
				flags: ConsoleCommandFlags::all(),
				func: commands::ccmd_help,
			},
			true,
		);

		self.console.register_command(
			"home",
			ConsoleCommand {
				flags: ConsoleCommandFlags::all(),
				func: commands::ccmd_home,
			},
			true,
		);

		self.console.register_command(
			"luamem",
			ConsoleCommand {
				flags: ConsoleCommandFlags::all(),
				func: commands::ccmd_luamem,
			},
			true,
		);

		self.console.register_command(
			"quit",
			ConsoleCommand {
				flags: ConsoleCommandFlags::all(),
				func: commands::ccmd_exit,
			},
			true,
		); // Built-in alias for "exit"

		self.console.register_command(
			"sound",
			ConsoleCommand {
				flags: ConsoleCommandFlags::all(),
				func: commands::ccmd_sound,
			},
			true,
		);

		self.console.register_command(
			"uptime",
			ConsoleCommand {
				flags: ConsoleCommandFlags::all(),
				func: commands::ccmd_uptime,
			},
			true,
		);

		self.console.register_command(
			"wgpudiag",
			ConsoleCommand {
				flags: ConsoleCommandFlags::all(),
				func: commands::ccmd_wgpudiag,
			},
			true,
		);

		self.console.register_command(
			"version",
			ConsoleCommand {
				flags: ConsoleCommandFlags::all(),
				func: commands::ccmd_version,
			},
			true,
		);
	}

	fn start_game(&mut self) {}

	fn start_sim(&mut self) {
		let (txout, rxout) = crossbeam::channel::unbounded();
		let (txin, rxin) = crossbeam::channel::unbounded();
		let playsim = Arc::new(RwLock::new(PlaySim::default()));
		let lua = self.lua.clone();
		let data = self.data.clone();

		self.scene = Scene::Playsim {
			sender: txin,
			receiver: rxout,
			playsim: playsim.clone(),
			thread: std::thread::Builder::new()
				.name("Impure: Playsim".to_string())
				.spawn(move || {
					impure::sim::run::<sim::EgressConfigClient>(sim::Context {
						playsim: playsim.clone(),
						lua,
						data,
						sender: txout,
						receiver: rxin,
					});
				})
				.expect("Failed to spawn OS thread for playsim."),
		};
	}
}
