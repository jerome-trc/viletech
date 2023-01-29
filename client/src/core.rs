use std::{
	borrow::Cow,
	error::Error,
	path::{Path, PathBuf},
	sync::Arc,
	thread::JoinHandle,
	time::Instant,
};

use log::error;
use nanorand::WyRand;
use parking_lot::{Mutex, RwLock};
use shipyard::World;
use vile::{
	audio::AudioCore,
	console::{self, Console},
	data::{Catalog, LoadError, LoadRequest, LoadTracker},
	frontend::{FrontendAction, FrontendMenu},
	gfx::{camera::Camera, core::GraphicsCore},
	input::InputCore,
	lith,
	rng::RngCore,
	sim::{self, PlaySim},
	user::UserCore,
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
	/// The user needs to choose whether their information should be stored
	/// portably or at a "home" directory.
	FirstStartup {
		/// `true` is the default presented to the user.
		portable: bool,

		portable_path: PathBuf,
		home_path: Option<PathBuf>,
	},
	Transition,
	/// The user hasn't entered the game yet. From here they can select a user
	/// profile and engine-global/cross-game preferences, assemble a load order,
	/// and begin the game launch process.
	Frontend {
		menu: FrontendMenu,
	},
	GameLoad {
		/// The mount thread takes a write guard to the catalog and another
		/// pointer to `tracker`.
		thread: JoinHandle<Vec<Result<(), LoadError>>>,
		/// How far along the mount/load process is `thread`?
		tracker: Arc<LoadTracker>,
		/// Print to the log how long the mount takes for diagnostic purposes.
		start_time: Instant,
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

#[derive(Debug)]
enum SceneChange {
	/// The user has requested an immediate exit from any other scene.
	/// Stop everything, drop everything, and close the window as fast as possible.
	Exit,
	/// Only used to transition from the first-time startup screen.
	ToFrontend,
	FrontendToTitle {
		/// The user's load order. Gets handed off to the mount thread.
		to_mount: Vec<PathBuf>,
	},
	TitleToFrontend,
}

pub struct ClientCore {
	/// (Rat) In my experience, a runtime log is much more informative if it
	/// states the duration for which the program executed.
	pub start_time: Instant,
	pub user: UserCore,
	pub catalog: Arc<RwLock<Catalog>>,
	pub project: Arc<RwLock<lith::Project>>,
	pub gfx: GraphicsCore,
	pub audio: AudioCore,
	pub input: InputCore,
	/// Kept behind an arc-lock in case the client's script API ends up needing
	/// to call into it from multiple threads. If this proves to never happen,
	/// it will be unwrapped.
	pub rng: Arc<Mutex<RngCore<WyRand>>>,
	pub console: Console<ConsoleCommand>,
	pub gui: World, // TODO: Replace with a menu stack
	pub camera: Camera,
	devgui: DeveloperGui,
	scene: Scene,
	next_scene: Option<SceneChange>,
}

// Public interface.
impl ClientCore {
	pub fn new(
		start_time: std::time::Instant,
		catalog: Catalog,
		gfx: GraphicsCore,
		console: Console<ConsoleCommand>,
	) -> Result<Self, Box<dyn Error>> {
		let user;
		let scene;
		let user_dir_home = vile::user::user_dir_home();
		let user_dir_portable = vile::user::user_dir_portable();
		let user_dir = vile::user::select_user_dir(&user_dir_portable, &user_dir_home);

		match user_dir {
			Some(udir) => {
				scene = Scene::Frontend {
					menu: FrontendMenu::default(),
				};

				user = UserCore::new(udir)?;
			}
			None => {
				scene = Scene::FirstStartup {
					portable: true,
					portable_path: user_dir_portable,
					home_path: user_dir_home,
				};

				user = UserCore::uninit();
			}
		}

		let camera = Camera::new(
			gfx.surface_config.width as f32,
			gfx.surface_config.height as f32,
		);

		let catalog = Arc::new(RwLock::new(catalog));
		let catalog_audio = catalog.clone();
		let catalog_lith = catalog.clone();

		let mut ret = ClientCore {
			start_time,
			user,
			catalog,
			project: Arc::new(RwLock::new(lith::Project::new(catalog_lith))),
			gfx,
			rng: Arc::new(Mutex::new(RngCore::default())),
			audio: AudioCore::new(catalog_audio, None)?,
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
			scene,
			next_scene: None,
		};

		ret.register_console_commands();

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
			Scene::GameLoad {
				thread: _,
				tracker: _,
				start_time: _,
			} => {
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
						self.next_scene = Some(SceneChange::FrontendToTitle { to_mount });
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
			Scene::FirstStartup {
				portable,
				portable_path,
				home_path,
			} => {
				if let Some(next) = Self::first_startup_screen(
					&self.gfx.egui.context,
					portable,
					portable_path.as_path(),
					home_path,
				) {
					self.next_scene = Some(next);
				}
			}
			_ => unimplemented!(),
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
		let _binds = self
			.input
			.user_binds
			.iter()
			.filter(|kb| kb.keycode == vkc && kb.modifiers == self.input.modifiers);

		// TODO: Invoke LithScript callbacks
	}

	pub fn scene_change(&mut self, control_flow: &mut ControlFlow) {
		// TODO: Tell branch predictor this is likely when the intrinsic stabilizes
		if self.next_scene.is_none() {
			return;
		}

		match &mut self.next_scene {
			Some(SceneChange::Exit) => {
				*control_flow = ControlFlow::Exit;
			}
			Some(SceneChange::FrontendToTitle { to_mount }) => {
				let to_mount = std::mem::take(to_mount);

				self.scene = match self.mount_load_order(to_mount) {
					Ok(s) => s,
					Err(err) => {
						error!("Game load failed. Reason: {err}");
						return;
					}
				};
			}
			Some(SceneChange::TitleToFrontend) => {
				self.catalog.write().truncate(1); // Keep base data
			}
			Some(SceneChange::ToFrontend) => {
				let first_startup = std::mem::replace(
					&mut self.scene,
					Scene::Frontend {
						menu: FrontendMenu::default(),
					},
				);

				if let Scene::FirstStartup {
					portable,
					portable_path,
					home_path,
				} = first_startup
				{
					let path = if portable {
						portable_path
					} else {
						home_path.unwrap()
					};

					if path.exists() {
						panic!(
							"Could not create user info folder; \
							something already exists at path: {p}",
							p = path.display(),
						);
					}

					std::fs::create_dir(&path)
						.expect("User information setup failed: directory creation error.");

					// If the basic file IO needed to initialize user information
					// is not even possible, there's no reason to go further

					self.user = match UserCore::new(path) {
						Ok(u) => u,
						Err(err) => panic!("User information setup failed: {err}"),
					};
				} else {
					unreachable!();
				}
			}
			None => unreachable!(),
		}

		let _ = self.next_scene.take();
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
			"quit",
			ConsoleCommand {
				flags: ConsoleCommandFlags::all(),
				func: commands::ccmd_exit,
			},
			true,
		); // Built-in alias for "exit"

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

	fn mount_load_order(&mut self, to_mount: Vec<PathBuf>) -> Result<Scene, String> {
		let start_time = Instant::now();
		let catalog = self.catalog.clone();
		let tracker = Arc::new(LoadTracker::default());
		let mut mounts = Vec::with_capacity(to_mount.len());

		for real_path in to_mount {
			if real_path.is_symlink() {
				return Err(format!(
					"Could not mount file: {}\r\n\t\
					Details: mounting symbolic links is forbidden.",
					real_path.display()
				));
			}

			let fstem = if let Some(stem) = real_path.file_stem() {
				stem
			} else {
				return Err(format!(
					"Could not mount file: {}\r\n\t\
					Details: file has no name.",
					real_path.display()
				));
			};

			let mount_point = if let Some(s) = fstem.to_str() {
				s
			} else {
				return Err(format!(
					"Could not mount file: {}\r\n\t\
					Details: file has invalid characters in its name.",
					real_path.display()
				));
			};

			let mount_point = mount_point.to_string();

			mounts.push((real_path, mount_point));
		}

		let tracker_sent = tracker.clone();
		let project_sent = self.project.clone();

		let thread = std::thread::spawn(move || {
			let request = LoadRequest {
				paths: &mounts,
				project: project_sent,
				tracker: Some(tracker_sent),
			};

			catalog.write().load(request)
		});

		Ok(Scene::GameLoad {
			thread,
			tracker,
			start_time,
		})
	}

	fn start_sim(&mut self) -> sim::Handle {
		let (txout, rxout) = crossbeam::channel::unbounded();
		let (txin, rxin) = crossbeam::channel::unbounded();

		let sim = Arc::new(RwLock::new(PlaySim::default()));
		let catalog = self.catalog.clone();

		self.console
			.enable_commands(|ccmd| ccmd.flags.contains(ConsoleCommandFlags::SIM));

		sim::Handle {
			sim: sim.clone(),
			sender: txin,
			receiver: rxout,
			thread: std::thread::Builder::new()
				.name("vile-playsim".to_string())
				.spawn(move || {
					vile::sim::run::<{ sim::Config::CLIENT.bits() }>(sim::Context {
						sim,
						catalog,
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
			Err(err) => panic!("Sim thread panicked: {err:#?}"),
		};

		// The arc-locked playsim object is meant to be dropped upon scene change,
		// so ensure no references have survived thread teardown

		debug_assert_eq!(
			Arc::strong_count(&sim.sim),
			1,
			"Sim state has {} illegal extra references.",
			Arc::strong_count(&sim.sim) - 1
		);
	}

	fn first_startup_screen(
		ctx: &egui::Context,
		portable: &mut bool,
		portable_path: &Path,
		home_path: &Option<PathBuf>,
	) -> Option<SceneChange> {
		let mut ret = None;

		egui::Window::new("Initial Setup").show(ctx, |ui| {
			ui.label(
				"Select where you want user information \
				- saved games, preferences, screenshots - \
				to be stored.",
			);

			ui.separator();

			ui.horizontal(|ui| {
				ui.radio_value(portable, true, "Portable: ");
				let p_path = portable_path.to_string_lossy();
				ui.code(p_path.as_ref());
			});

			ui.horizontal(|ui| {
				ui.add_enabled_ui(home_path.is_some(), |ui| {
					let label;
					let h_path;

					if let Some(home) = home_path {
						label = "Home: ";
						h_path = home.to_string_lossy();
					} else {
						label = "No home folder found.";
						h_path = Cow::Borrowed("");
					}

					ui.radio_value(portable, false, label);
					ui.code(h_path.as_ref())
				});
			});

			ui.separator();

			ui.horizontal(|ui| {
				if ui.button("Continue").clicked() {
					ret = Some(SceneChange::ToFrontend);
				}

				if ui.button("Exit").clicked() {
					ret = Some(SceneChange::Exit);
				}
			});
		});

		ret
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
