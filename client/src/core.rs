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
	console::{Console, ConsoleCommand, ConsoleRequest},
	data::{game::{DataCore, GameDataKind}, GameDataObject},
	depends::*,
	frontend::{FrontendAction, FrontendMenu},
	gfx::GfxCore,
	rng::RngCore,
	sim::Playsim,
	utils::path::*,
	vfs::{self, VirtualFs, ImpureVfs}, audio::AudioCore
};

use kira::{
	manager::{AudioManager, AudioManagerSettings},
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
};
use log::{error, info};
use mlua::Lua;
use nanorand::WyRand;
use parking_lot::RwLock;
use shipyard::World;
use std::{
	env,
	error::Error,
	fs,
	io,
	path::PathBuf,
	sync::Arc,
	thread::Thread
};
use winit::{event::KeyboardInput, event_loop::ControlFlow, window::WindowId};

enum Scene {
	Frontend {
		menu: FrontendMenu,
	},
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

enum SceneChange {
	PlaysimSingle { to_mount: Vec<PathBuf> },
}

pub struct ClientCore {
	pub start_time: std::time::Instant,
	pub vfs: Arc<RwLock<VirtualFs>>,
	pub lua: Lua,
	pub data: DataCore,
	pub rng: RngCore<WyRand>,
	pub gfx: GfxCore,
	pub audio: AudioCore,
	pub console: Console,
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
		gfx: GfxCore,
		console: Console,
	) -> Result<Self, Box<dyn Error>> {
		let audio_mgr_settings = AudioManagerSettings::default();
		let sound_cap = audio_mgr_settings.capacities.sound_capacity;

		let audio = AudioCore {
			manager: AudioManager::new(audio_mgr_settings)?,
			music1: None,
			music2: None,
			handles: Vec::<_>::with_capacity(sound_cap),
		};

		let mut ret = ClientCore {
			start_time,
			vfs,
			lua,
			data,
			gfx,
			rng: RngCore::default(),
			audio,
			console,
			scene: Scene::Frontend {
				menu: FrontendMenu::default(),
			},
			next_scene: None,
		};

		ret.register_console_commands();
		ret.build_user_dirs()?;

		Ok(ret)
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
						self.next_scene = Some(SceneChange::PlaysimSingle { to_mount });
					}
				}
			}
			Scene::Title { .. } => {
				// ???
			}
			_ => {}
		};

		self.console.draw(&self.gfx.egui.context);
		self.gfx.render_finish(output.0, output.1);
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

					let bytes = match vfsg.read(&arg) {
						Ok(b) => b,
						Err(err) => {
							if let vfs::Error::NonExistentEntry = err {
								info!("No sound file under virtual path: {}", arg);
							} else {
								info!("{}", err);
							}

							continue;
						}
					};

					let bytes = bytes.to_owned();
					let cursor = io::Cursor::new(bytes);

					let sdata = match StaticSoundData::from_cursor(
						cursor,
						StaticSoundSettings::default(),
					) {
						Ok(ssd) => ssd,
						Err(err) => {
							info!("Failed to create sound from file: {}", err);
							continue;
						}
					};

					let snd = match self.audio.manager.play(sdata) {
						Ok(s) => s,
						Err(err) => {
							info!("Failed to play sound: {}", err);
							continue;
						}
					};

					self.audio.handles.push(snd);
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
				SceneChange::PlaysimSingle { to_mount } => {
					let metas = self.vfs.write().mount_gamedata(&to_mount);
					let vfsg = self.vfs.read();

					for meta in metas {
						let kind = vfsg.gamedata_kind(&meta.uuid);
						self.data.objects.push(GameDataObject::new(meta, kind));
					}

					drop(vfsg);
					self.start_game();
				}
			},
		};
	}
}

// Internal implementation details: general.
impl ClientCore {
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
			false,
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
			true,
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
			false,
		));

		self.console.register_command(ConsoleCommand::new(
			"luamem",
			|_, _| ConsoleRequest::LuaMem,
			|_, _| {
				info!("Prints the current heap memory used by the client-side Lua state.");
			},
			true,
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
			true,
		));

		self.console.register_command(ConsoleCommand::new(
			"uptime",
			|_, _| ConsoleRequest::Uptime,
			|_, _| {
				info!("Prints the length of the time the engine has been running.");
			},
			true,
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
			true,
		));

		self.console.register_command(ConsoleCommand::new(
			"wgpudiag",
			|_, _| {
				ConsoleRequest::WgpuDiag
			},
			|_, _| {
				info!("Prints information about the graphics device and WGPU backend.");
			},
			false
		));
	}

	fn build_user_dirs(&self) -> io::Result<()> {
		let user_path = match get_user_dir() {
			Some(up) => up,
			None => {
				return Err(io::Error::new(
					io::ErrorKind::Other,
					"Failed to retrieve user info path. \
					Home directory path is malformed, \
					or this platform is currently unsupported.",
				));
			}
		};

		if !user_path.exists() {
			match fs::create_dir_all(&user_path) {
				Ok(()) => {}
				Err(err) => {
					return Err(io::Error::new(
						err.kind(),
						format!("Failed to create a part of the user info path: {}", err),
					));
				}
			};
		}

		let profiles_path = user_path.join("profiles");

		// End execution with an error if this directory has anything else in it,
		// so as not to clobber any other software's config files

		if !profiles_path.exists() {
			if !user_path.dir_empty() {
				return Err(io::Error::new(
					io::ErrorKind::Other,
					format!(
						"User info folder has unexpected contents; \
						is another program named \"Impure\" using it?
						({})",
						user_path.display()
					),
				));
			} else {
				match fs::create_dir(&profiles_path) {
					Ok(()) => {}
					Err(err) => {
						return Err(io::Error::new(
							io::ErrorKind::Other,
							format!(
								"Failed to create directory: {} \
								Error: {}",
								profiles_path.display(),
								err
							),
						))
					}
				};
			}
		}

		if profiles_path.dir_empty() {
			impure::utils::env::create_default_user_dir()?;
		}

		Ok(())
	}

	fn start_game(&mut self) {
		let vfsg = self.vfs.read();

		for obj in &mut self.data.objects {
			Self::load_assets(&vfsg, obj);
		}
	}
}

// Internal implementation details: on-game-start asset loading.
impl ClientCore {
	fn load_assets(vfs: &VirtualFs, obj: &mut impure::data::game::Object) {
		// ???
	}
}
