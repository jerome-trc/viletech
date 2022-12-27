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
along with this program. If not, see <http://www.gnu.org/licenses/>.

*/

use std::{error::Error, path::PathBuf, sync::Arc};

use log::error;
use mlua::prelude::*;
use nanorand::WyRand;
use parking_lot::{Mutex, RwLock};
use shipyard::World;
use vile::{
	audio::AudioCore,
	console::{self, Console},
	data::DataCore,
	frontend::{FrontendAction, FrontendMenu},
	gfx::{camera::Camera, core::GraphicsCore},
	input::InputCore,
	lua::LuaExt,
	rng::RngCore,
	sim::{self, PlaySim},
	vfs::{VirtualFs, VirtualFsExt},
};
use winit::{
	event::{ElementState, KeyboardInput, VirtualKeyCode},
	event_loop::ControlFlow,
	window::WindowId,
};

use crate::commands::{
	self, Command as ConsoleCommand, CommandFlags as ConsoleCommandFlags, Request as ConsoleRequest,
};

type DeveloperGui = vile::DeveloperGui<DevGuiStatus>;

enum Scene {
	Transition,
	/// The user hasn't entered the game yet. From here they can select a user
	/// profile and engine-global/cross-game preferences, assemble a load order,
	/// and begin the game launch process.
	Frontend {
		menu: FrontendMenu,
	},
	/// Where the user is taken after leaving the frontend, unless they have
	/// specified to be taken directly to a playsim.
	Title {
		inner: sim::Handle,
	},
	PlaySim {
		inner: sim::Handle,
	},
	CastCall,
}

enum SceneChange {
	Exit,
	Frontend,
	Title { to_mount: Vec<PathBuf> },
	PlaySim {},
}

pub struct ClientCore {
	pub start_time: std::time::Instant,
	pub vfs: Arc<RwLock<VirtualFs>>,
	pub lua: Arc<Mutex<Lua>>,
	pub data: Arc<RwLock<DataCore>>,
	pub rng: Arc<Mutex<RngCore<WyRand>>>,
	pub gfx: GraphicsCore,
	pub audio: AudioCore,
	pub input: InputCore,
	pub console: Console<ConsoleCommand>,
	pub gui: World,
	pub camera: Camera,
	devgui: DeveloperGui,
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
		let camera = Camera::new(
			gfx.surface_config.width as f32,
			gfx.surface_config.height as f32,
		);

		let vfs_audio = vfs.clone();

		let mut ret = ClientCore {
			start_time,
			vfs,
			lua: Arc::new(Mutex::new(lua)),
			data: Arc::new(RwLock::new(data)),
			gfx,
			rng: Arc::new(Mutex::new(RngCore::default())),
			audio: AudioCore::new(vfs_audio, None)?,
			input: InputCore::default(),
			console,
			gui: World::default(),
			camera,
			devgui: DeveloperGui {
				#[cfg(debug_assertions)]
				open: true,
				#[cfg(not(debug_assertions))]
				open: false,
				left: DevGuiStatus::Audio,
				right: DevGuiStatus::Console,
			},
			scene: Scene::Frontend {
				menu: FrontendMenu::default(),
			},
			next_scene: None,
		};

		{
			let lua = ret.lua.lock();
			lua.init_api_client(Some(ret.rng.clone()))?;
			lua.load_api_client();
		}

		ret.register_console_commands();
		vile::user::build_user_dirs()?;

		Ok(ret)
	}

	/// Draw a new frame. Since this requires branching on the current scene, take
	/// the opportunity to do scene-specific processing like acting on egress
	/// messages coming out of a running playsim.
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
			Scene::PlaySim { inner } => {
				while let Ok(egress) = inner.receiver.try_recv() {
					match egress {
						sim::OutMessage::Toast(toast) => {
							self.console.write(toast, console::MessageKind::Toast)
						}
					}
				}
			}
			Scene::Title { .. } => {
				// ???
			}
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
			_ => {}
		};

		// TODO: mark as `unlikely` when it stabilizes
		if self.devgui.open {
			let ctx = &self.gfx.egui.context;
			let mut devgui_open = true;
			let screen_rect = ctx.input().screen_rect;

			DeveloperGui::window(ctx)
				.open(&mut devgui_open)
				.show(ctx, |ui| {
					// Prevent window from overflowing off the screen's sides
					ui.set_max_width(screen_rect.width());

					self.devgui.selectors(
						ui,
						&[
							(DevGuiStatus::Console, "Console"),
							(DevGuiStatus::LithRepl, "REPL"),
							(DevGuiStatus::Graphics, "Graphics"),
							(DevGuiStatus::Vfs, "VFS"),
							(DevGuiStatus::Audio, "Audio"),
						],
					);

					self.devgui.panel_left(ctx).show_inside(ui, |ui| {
						match self.devgui.left {
							DevGuiStatus::Console => {
								self.console.ui(ctx, ui);
							}
							DevGuiStatus::LithRepl => {
								// Soon!
							}
							DevGuiStatus::Vfs => {
								// Soon!
							}
							DevGuiStatus::Graphics => {
								self.gfx.ui(ctx, ui);
							}
							DevGuiStatus::Audio => {
								self.audio.ui(ctx, ui);
							}
						};
					});

					self.devgui.panel_right(ctx).show_inside(ui, |ui| {
						match self.devgui.right {
							DevGuiStatus::Console => {
								self.console.ui(ctx, ui);
							}
							DevGuiStatus::LithRepl => {
								// Soon!
							}
							DevGuiStatus::Vfs => {
								// Soon!
							}
							DevGuiStatus::Graphics => {
								self.gfx.ui(ctx, ui);
							}
							DevGuiStatus::Audio => {
								self.audio.ui(ctx, ui);
							}
						};
					});
				});

			self.devgui.open = devgui_open;
		}

		self.gfx.render_finish(frame);
	}

	pub fn process_console_requests(&mut self) {
		while !self.console.requests.is_empty() {
			match self.console.requests.pop_front().unwrap() {
				ConsoleRequest::Callback(func) => {
					(func)(self);
				}
				ConsoleRequest::Exit => {
					self.exit();
					return;
				}
				ConsoleRequest::None => {}
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
		} else if let Some(VirtualKeyCode::Grave) = event.virtual_keycode {
			if event.state == ElementState::Pressed {
				self.devgui.open = !self.devgui.open;
			}
		}

		let vkc = event.virtual_keycode.unwrap();
		let binds = self
			.input
			.user_binds
			.iter()
			.filter(|kb| kb.keycode == vkc && kb.modifiers == self.input.modifiers);
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
			Some(scene) => {
				let mut prev = Scene::Transition;
				std::mem::swap(&mut self.scene, &mut prev);

				// Disable contextual console commands
				match prev {
					Scene::Frontend { .. } => {
						self.console.disable_commands(|ccmd| {
							ccmd.flags.contains(ConsoleCommandFlags::FRONTEND)
						});
					}
					Scene::Title { .. } => {
						self.console.disable_commands(|ccmd| {
							ccmd.flags.contains(ConsoleCommandFlags::TITLE)
						});
					}
					_ => {}
				}

				// End sim where necessary
				match prev {
					Scene::PlaySim { inner } | Scene::Title { inner } => {
						self.end_sim(inner);
					}
					_ => {}
				};

				match scene {
					SceneChange::Exit => {
						*control_flow = ControlFlow::Exit;
					}
					SceneChange::Frontend => {
						self.console.enable_commands(|ccmd| {
							ccmd.flags.contains(ConsoleCommandFlags::FRONTEND)
						});
					}
					SceneChange::Title { to_mount } => {
						let mut metas = vec![self
							.vfs
							.read()
							.parse_gamedata_meta("/viletech/meta.toml")
							.expect("Engine data package manifest is malformed.")];

						if !to_mount.is_empty() {
							let mut m = self.vfs.write().mount_gamedata(&to_mount);
							metas.append(&mut m);
						}

						self.data.write().populate(metas, &self.vfs.read());

						self.console.enable_commands(|ccmd| {
							ccmd.flags.contains(ConsoleCommandFlags::TITLE)
						});

						self.scene = Scene::Title {
							inner: self.start_sim(),
						};
					}
					SceneChange::PlaySim {} => {
						self.scene = Scene::PlaySim {
							inner: self.start_sim(),
						};
					}
				}
			}
		};
	}

	pub fn exit(&mut self) {
		self.next_scene = Some(SceneChange::Exit);
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
			"mididiag",
			ConsoleCommand {
				flags: ConsoleCommandFlags::all(),
				func: commands::ccmd_mididiag,
			},
			true,
		);

		self.console.register_command(
			"music",
			ConsoleCommand {
				flags: ConsoleCommandFlags::all(),
				func: commands::ccmd_music,
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

		self.console.register_command(
			"vfsdiag",
			ConsoleCommand {
				flags: ConsoleCommandFlags::all(),
				func: commands::ccmd_vfsdiag,
			},
			true,
		);
	}

	fn start_sim(&mut self) -> sim::Handle {
		let (txout, rxout) = crossbeam::channel::unbounded();
		let (txin, rxin) = crossbeam::channel::unbounded();

		let sim = Arc::new(RwLock::new(PlaySim::default()));
		let lua = self.lua.clone();
		let data = self.data.clone();

		{
			let l = self.lua.lock();

			l.init_api_playsim(sim.clone())
				.expect("Failed to construct Lua's playsim API.");

			l.set_app_data(sim.clone());
		}

		self.console
			.enable_commands(|ccmd| ccmd.flags.contains(ConsoleCommandFlags::SIM));

		sim::Handle {
			sim: sim.clone(),
			sender: txin,
			receiver: rxout,
			thread: std::thread::Builder::new()
				.name("VileTech: Playsim".to_string())
				.spawn(move || {
					vile::sim::run::<{ sim::Config::CLIENT.bits() }>(sim::Context {
						sim,
						lua,
						data,
						sender: txout,
						receiver: rxin,
					});
				})
				.expect("Failed to spawn OS thread for playsim."),
		}
	}

	fn end_sim(&mut self, sim: sim::Handle) {
		self.console
			.disable_commands(|ccmd| ccmd.flags.contains(ConsoleCommandFlags::SIM));

		sim.sender
			.send(sim::InMessage::Stop)
			.expect("Sim sender channel unexpectedly disconnected.");

		match sim.thread.join() {
			Ok(()) => {}
			Err(err) => panic!("Sim thread panicked: {:#?}", err),
		};

		let lua = self.lua.lock();
		lua.remove_app_data::<Arc<RwLock<PlaySim>>>().unwrap();

		lua.clear_api_playsim()
			.expect("Failed to destroy Lua's playsim API.");
		lua.expire_registry_values();

		// This function's documentation recommends running it twice
		lua.gc_collect()
			.expect("Failed to run Lua GC after closing playsim.");
		lua.gc_collect()
			.expect("Failed to run Lua GC after closing playsim.");

		// The arc-locked playsim object is meant to be dropped upon scene change,
		// so ensure no references have survived thread teardown and Lua GC

		debug_assert_eq!(
			Arc::strong_count(&sim.sim),
			1,
			"Sim state has {} illegal extra references.",
			Arc::strong_count(&sim.sim) - 1
		);
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevGuiStatus {
	Console,
	LithRepl,
	Vfs,
	Graphics,
	Audio,
}
