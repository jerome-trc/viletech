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
	audio::AudioCore,
	console::{Console, Command as ConsoleCommand, Request as ConsoleRequest},
	data::{game::DataCore, Namespace},
	depends::*,
	frontend::{FrontendAction, FrontendMenu},
	gfx::{camera::Camera, core::GraphicsCore},
	rng::RngCore,
	sim::{InMessage as SimMessage, PlaySim, ThreadContext as SimThreadContext},
	utils::{path::*, string::line_from_char_index},
	vfs::{self, ImpureVfs, VirtualFs, ImpureVfsHandle, ZsProxyFs}, zscript,
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
	fs, io,
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
			handles: Vec::<_>::with_capacity(sound_cap),
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
		ret.build_user_dirs()?;

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

		self.camera.update(frame.delta_time_secs_f32());
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

				let mut rpass = frame.render_pass();

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

					let bytes = match vfsg.read(&arg) {
						Ok(b) => b,
						Err(err) => {
							if let vfs::Error::NonExistentEntry(_) = err {
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
					if !to_mount.is_empty() {
						let metas = self.vfs.write().mount_gamedata(&to_mount);
						let vfsg = self.vfs.read();
	
						for meta in metas {
							let kind = vfsg.gamedata_kind(&meta.uuid);
							self.data.namespaces.push(Namespace::new(meta, kind));
						}
					}

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
			}
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

		for i in 0..self.data.namespaces.len() {
			Self::load_assets(&vfsg, i, &mut self.data);
		}
	}

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

// Internal implementation details: on-game-start asset loading.
impl<'lua> ClientCore<'lua> {
	fn load_assets(vfs: &VirtualFs, namespace: usize, data: &mut DataCore) {
		let uuid = &data.namespaces[namespace].meta.uuid;

		let entry = vfs
			.lookup(uuid)
			.expect("`ClientCore::load_assets` failed to find a namespace by UUID.");
	
		if entry.has_zscript() {
			let pvfs = ZsProxyFs::new(vfs, uuid);
			let parse_out = zscript::parse(pvfs);

			if !parse_out.errors.is_empty() {
				error!(
					"{} errors during ZScript transpile: {}",
					parse_out.errors.len(),
					uuid
				);
			}

			for err in parse_out.errors {
				let file = &parse_out.files[err.main_spans[0].get_file()];
				let start = err.main_spans[0].get_start();
				let end = err.main_spans[0].get_end();
				let (line, line_index) = line_from_char_index(file.text(), start).unwrap();
				let line = line.trim();
				let line_start = file.text().find(line).unwrap();

				let mut indicators = String::with_capacity(line.len());
				indicators.push('\t');

				for _ in line_start..start {
					indicators.push(' ');
				}

				for _ in 0..(end - start) {
					indicators.push('^');
				}

				error!(
					"{}:{}:{}\r\n\r\n\t{}\r\n{}\r\n\tDetails: {}.\r\n",
					format!("/{}/{}", uuid, file.filename()),
					line_index + 1,
					start - line_start,
					line,
					indicators,
					err.msg
				);
			}
		}
	}
}
