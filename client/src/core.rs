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
	console::{Command as ConsoleCommand, Console, Request as ConsoleRequest},
	data::game::DataCore,
	depends::*,
	frontend::{FrontendAction, FrontendMenu},
	gfx::{camera::Camera, core::GraphicsCore},
	rng::RngCore,
	sim::{InMessage as SimMessage, PlaySim, ThreadContext as SimThreadContext},
	utils::path::*,
	vfs::{ImpureVfs, VirtualFs},
};

use kira::{
	manager::{AudioManager, AudioManagerSettings},
	sound::static_sound::StaticSoundSettings,
};
use log::{error, info};
use mlua::Lua;
use nanorand::WyRand;
use parking_lot::RwLock;
use shipyard::World;
use std::{
	env,
	error::Error,
	path::PathBuf,
	sync::{atomic::AtomicBool, Arc},
	thread::JoinHandle,
};
use winit::{event::KeyboardInput, event_loop::ControlFlow, window::WindowId};

enum Scene {
	/// The user hasn't entered the game yet. From here they can select a user
	/// profile and engine-global/cross-game preferences, assemble a load order,
	/// and begin the game launch process.
	Frontend {
		menu: FrontendMenu,
	},
	/// Where the user is taken after leaving the frontend, unless they have
	/// specified to be taken directly to a playsim.
	Title,
	Playsim {
		running: Arc<AtomicBool>,
		messenger: crossbeam::channel::Sender<SimMessage>,
		thread: JoinHandle<()>,
	},
	CastCall,
}

enum SceneChange {
	Title { to_mount: Vec<PathBuf> },
}

pub struct ClientCore<'lua> {
	pub start_time: std::time::Instant,
	pub vfs: Arc<RwLock<VirtualFs>>,
	pub lua: Lua,
	pub data: DataCore<'lua>,
	pub rng: RngCore<WyRand>,
	pub gfx: GraphicsCore,
	pub audio: AudioCore,
	pub console: Console,
	pub gui: World,
	pub camera: Camera,
	pub playsim: Arc<RwLock<PlaySim>>,
	scene: Scene,
	next_scene: Option<SceneChange>,
}

// Public interface.
impl<'lua> ClientCore<'lua> {
	pub fn new(
		start_time: std::time::Instant,
		vfs: Arc<RwLock<VirtualFs>>,
		lua: Lua,
		data: DataCore<'lua>,
		gfx: GraphicsCore,
		console: Console,
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
			lua,
			data,
			gfx,
			rng: RngCore::default(),
			audio,
			console,
			gui: World::default(),
			camera,
			playsim: Arc::new(RwLock::new(PlaySim::default())),
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
		let cons_reqs: Vec<_> = self.console.requests.drain(..).collect();

		for req in cons_reqs {
			match req {
				ConsoleRequest::None => {}
				ConsoleRequest::File(p) => {
					let vfsg = self.vfs.read();
					info!("{}", vfsg.ccmd_file(p));
				}
				ConsoleRequest::LuaMem => {
					info!(
						"Client Lua state heap usage (bytes): {}",
						self.lua.used_memory()
					);
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
				ConsoleRequest::Uptime => {
					info!("{}", impure::uptime_string(self.start_time))
				}
				ConsoleRequest::WgpuDiag => {
					info!("{}", self.gfx.diag());
				}
			}
		}
	}

	pub fn on_window_resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		self.gfx.resize(new_size);
		self.camera
			.resize(new_size.width as f32, new_size.height as f32);
	}

	pub fn on_key_event(&mut self, input: &KeyboardInput) {
		self.console.on_key_event(input);
	}

	pub fn scene_change(&mut self) {
		let next_scene = self.next_scene.take();

		match next_scene {
			None => {
				// TODO: Mark as likely with intrinsics...?
			}
			Some(scene) => match scene {
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

					self.data.populate(metas, &self.vfs.read());
					self.start_game();
				}
			},
		};
	}
}

// Internal implementation details: general.
impl<'lua> ClientCore<'lua> {
	fn register_console_commands(&mut self) {
		self.console.register_command(ConsoleCommand::new(
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
		));

		self.console.register_command(ConsoleCommand::new(
			"file",
			|_, args| {
				let path = if args.is_empty() { "/" } else { args[0] };
				ConsoleRequest::File(PathBuf::from(path))
			},
			|_, _| {
				info!(
					"Prints the contents of a virtual file system directory, \
					or information about a file."
				);
			},
		));

		self.console.register_command(ConsoleCommand::new(
			"home",
			|_, _| {
				match get_user_dir() {
					Some(p) => info!("{}", p.display()),
					None => {
						info!(
							"Home directory path is malformed, \
							or this platform is unsupported."
						);
					}
				}

				ConsoleRequest::None
			},
			|_, _| {
				info!("Prints the directory used to store user info.");
			},
		));

		self.console.register_command(ConsoleCommand::new(
			"luamem",
			|_, _| ConsoleRequest::LuaMem,
			|_, _| {
				info!("Prints the current heap memory used by the client-side Lua state.");
			},
		));

		self.console.register_command(ConsoleCommand::new(
			"sound",
			|this, args| {
				if args.is_empty() {
					this.call_help(None);
					return ConsoleRequest::None;
				}

				ConsoleRequest::Sound(args[0].to_string())
			},
			|this, _| {
				info!(
					"Starts a sound at default settings from the virtual file system.
					Usage: {} <virtual file path/asset number/asset ID>",
					this.get_id()
				);
			},
		));

		self.console.register_command(ConsoleCommand::new(
			"uptime",
			|_, _| ConsoleRequest::Uptime,
			|_, _| {
				info!("Prints the length of the time the engine has been running.");
			},
		));

		self.console.register_command(ConsoleCommand::new(
			"version",
			|_, _| {
				info!("{}", impure::full_version_string());
				ConsoleRequest::None
			},
			|_, _| {
				info!("Prints the engine version.");
			},
		));

		self.console.register_command(ConsoleCommand::new(
			"wgpudiag",
			|_, _| ConsoleRequest::WgpuDiag,
			|_, _| {
				info!("Prints information about the graphics device and WGPU backend.");
			},
		));
	}

	fn start_game(&mut self) {}

	fn start_sim(&mut self) {
		let (sender, receiver) = crossbeam::channel::unbounded();
		let playsim = self.playsim.clone();
		let running = Arc::new(AtomicBool::new(true));

		self.scene = Scene::Playsim {
			running: running.clone(),
			messenger: sender,
			thread: std::thread::Builder::new()
				.name("Impure: Playsim".to_string())
				.spawn(move || {
					impure::sim::run(SimThreadContext {
						playsim,
						receiver,
						running,
					});
				})
				.expect("Failed to spawn OS thread for playsim."),
		};
	}
}
