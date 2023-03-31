//! # VileTech Client

mod core;
mod frontend;
mod game;
mod load;

use std::{
	borrow::Cow,
	sync::Arc,
	time::{Duration, Instant},
};

use bevy::{
	app::AppExit,
	diagnostic::LogDiagnosticsPlugin,
	prelude::*,
	window::WindowMode,
	winit::{UpdateMode, WinitSettings},
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use clap::Parser;
use indoc::printdoc;
use parking_lot::RwLock;
use viletech::{
	audio::AudioCore,
	data::{Catalog, CatalogExt},
	lith,
	rng::RngCore,
	user::UserCore,
	DeveloperGui,
};

use self::core::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let start_time = Instant::now();
	let args = Clap::parse();

	if args.version {
		println!("{}", viletech::short_version_string());
		println!("{}", &version_string());
		return Ok(());
	}

	if args.about {
		printdoc! {"
VileTech Client - Copyright (C) 2022-2023 - ***REMOVED***

This program comes with ABSOLUTELY NO WARRANTY.

This is free software, and you are welcome to redistribute it under certain
conditions. See the license document that comes with your installation."
		};

		return Ok(());
	}

	viletech::thread_pool_init(args.threads);

	let mut app = App::new();

	app.add_plugin(LogDiagnosticsPlugin::default());

	// Common //////////////////////////////////////////////////////////////////

	app.add_state::<AppState>()
		.insert_resource(WinitSettings {
			return_from_run: false,
			focused_mode: UpdateMode::Reactive {
				max_wait: Duration::from_secs_f64(1.0 / 60.0),
			},
			unfocused_mode: UpdateMode::ReactiveLowPower {
				max_wait: Duration::from_secs_f64(1.0 / 30.0),
			},
		})
		.add_plugins(
			DefaultPlugins
				.set(WindowPlugin {
					primary_window: Some(Window {
						title: "VileTech Client".to_string(),
						mode: WindowMode::Windowed,
						..Default::default()
					}),
					..default()
				})
				.set(TaskPoolPlugin {
					task_pool_options: TaskPoolOptions::with_num_threads(
						args.threads.unwrap_or_else(|| {
							std::thread::available_parallelism()
								.map(|u| u.get())
								.unwrap_or(0)
						}),
					),
				}),
		)
		.add_plugin(EguiPlugin)
		.add_system(common_update);

	let catalog = Arc::new(RwLock::new(Catalog::default()));
	let catalog_audio = catalog.clone();

	if let Err(err) = catalog.write().mount_basedata() {
		error!("Failed to find and mount engine base data: {err}");
		return Err(Box::new(err));
	}

	let audio = AudioCore::new(catalog_audio, None)?;
	let runtime = lith::Runtime::default();
	let rng = RngCore::default();
	let user;

	let user_dir_home = viletech::user::user_dir_home();
	let user_dir_portable = viletech::user::user_dir_portable();
	let user_dir = viletech::user::select_user_dir(&user_dir_portable, &user_dir_home);

	if let Some(udir) = user_dir {
		user = UserCore::new(udir)?;
	} else {
		app.insert_resource(FirstStartup {
			portable: true,
			portable_path: user_dir_portable,
			home_path: user_dir_home,
		});

		user = UserCore::uninit();
	}

	app.insert_resource(ClientCore {
		start_time,
		audio,
		catalog,
		devgui: DeveloperGui {
			#[cfg(debug_assertions)]
			open: true,
			#[cfg(not(debug_assertions))]
			open: false,
			left: DevGuiStatus::Vfs,
			right: DevGuiStatus::Console,
		},
		runtime,
		rng,
		user,
	});

	app.add_system(init_on_enter.in_schedule(OnEnter(AppState::Init)));

	// First-time startup //////////////////////////////////////////////////////

	app.add_system(first_startup.in_set(OnUpdate(AppState::FirstStartup)));

	// Frontend ////////////////////////////////////////////////////////////////

	app.add_system(frontend::update.in_set(OnUpdate(AppState::Frontend)))
		.add_system(frontend::on_enter.in_schedule(OnEnter(AppState::Frontend)))
		.add_system(frontend::on_exit.in_schedule(OnExit(AppState::Frontend)));

	// Load ////////////////////////////////////////////////////////////////////

	app.add_system(load::update.in_set(OnUpdate(AppState::Load)))
		.add_system(load::on_exit.in_schedule(OnExit(AppState::Load)));

	// Game ////////////////////////////////////////////////////////////////////

	app.add_system(game::update.in_set(OnUpdate(AppState::Game)))
		.insert_resource(FixedTime::new_from_secs(1.0 / 35.0))
		.add_system(
			viletech::sim::tick
				.run_if(in_state(AppState::Game))
				.in_schedule(CoreSchedule::FixedUpdate),
		)
		.add_system(game::on_exit.in_schedule(OnExit(AppState::Game)));

	// Run /////////////////////////////////////////////////////////////////////

	app.run();

	unreachable!("Unexpected return from Winit event loop.")
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum AppState {
	/// Checks if it necessary to go to `FirstStartup`. Otherwise go to `Frontend`.
	#[default]
	Init,
	/// The user needs to choose whether their information should be stored
	/// portably or at a "home" directory.
	FirstStartup,
	/// The user has not entered the game yet. From here they can select a user
	/// profile and engine-global/cross-game preferences, assemble a load order,
	/// and begin the game launch process.
	Frontend,
	/// A loading screen which draws progress bars.
	Load,
	/// The title screen, gameplay, intermissions, casting calls...
	/// - `sim` is only `Some` at the title if the game calls for a title map.
	/// If the script controlling game flow chooses, it can last into gameplay,
	/// but in most cases it will be swapped out with an entirely new instance.
	/// - `sim` stays between intermissions, although much of its state is altered.
	/// - `sim` is put back to `None` when the game finishes and the cast call starts.
	Game,
}

#[must_use]
fn version_string() -> String {
	format!("VileTech Client {}", env!("CARGO_PKG_VERSION"))
}

fn init_on_enter(startup: Option<Res<FirstStartup>>, mut next_state: ResMut<NextState<AppState>>) {
	if startup.is_none() {
		next_state.set(AppState::Frontend);
	} else {
		next_state.set(AppState::FirstStartup);
	}
}

fn first_startup(
	mut startup: ResMut<FirstStartup>,
	mut core: ResMut<ClientCore>,
	mut ctxs: EguiContexts,
	mut next_state: ResMut<NextState<AppState>>,
	mut exit: EventWriter<AppExit>,
) {
	// TODO: Localize these strings.

	egui::Window::new("Initial Setup").show(ctxs.ctx_mut(), |ui| {
		ui.label(
			"Select where you want user information \
			- saved games, preferences, screenshots - \
			to be stored.",
		);

		ui.separator();

		ui.horizontal(|ui| {
			ui.radio_value(&mut startup.portable, true, "Portable: ");
			let p_path = startup.portable_path.to_string_lossy();
			ui.code(p_path.as_ref());
		});

		ui.horizontal(|ui| {
			ui.add_enabled_ui(startup.home_path.is_some(), |ui| {
				let label;
				let h_path;

				if let Some(home) = &startup.home_path {
					label = "Home: ";
					h_path = home.to_string_lossy();
				} else {
					label = "No home folder found.";
					h_path = Cow::Borrowed("");
				}

				let mut portable = startup.portable;

				ui.radio_value(&mut portable, false, label);
				ui.code(h_path.as_ref());

				startup.portable = portable;
			});
		});

		ui.separator();

		ui.horizontal(|ui| {
			if ui.button("Continue").clicked() {
				let path = if startup.portable {
					startup.portable_path.clone()
				} else {
					startup.home_path.clone().unwrap()
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
				// is not even possible, there's no reason to go further.

				core.user = match UserCore::new(path) {
					Ok(u) => u,
					Err(err) => panic!("User information setup failed: {err}"),
				};

				next_state.set(AppState::Frontend);
			}

			if ui.button("Exit").clicked() {
				exit.send(AppExit);
			}
		});
	});
}

fn common_update(mut core: ResMut<ClientCore>) {
	core.audio.update();
}

#[derive(Debug, clap::Parser)]
struct Clap {
	/// Prints the client and engine versions.
	#[arg(short = 'V', long = "version")]
	version: bool,
	/// Prints license information.
	#[arg(short = 'A', long = "about")]
	about: bool,
	/// Sets the number of threads used by the global thread pool
	///
	/// If set to 0 or not set, this will be automatically selected based on the
	/// number of logical CPUs your computer has.
	#[arg(short, long)]
	threads: Option<usize>,
}

#[cfg(not(all()))]
fn main() -> Result<(), Box<dyn Error>> {
	vile::thread_pool_init(args.threads);

	let (log_sender, log_receiver) = crossbeam::channel::unbounded();

	match vile::log_init(Some(log_sender)) {
		Ok(()) => {}
		Err(err) => {
			eprintln!("Failed to initialise logging backend: {err}");
			return Err(err);
		}
	}

	let console = Console::<ccmd::Command>::new(log_receiver);

	vile::log_init_diag(&version_string())?;

	let mut catalog = Catalog::default();

	if let Err(err) = catalog.mount_basedata() {
		error!("Failed to find and mount engine base data: {err}");
		return Err(Box::new(err));
	}

	let event_loop = EventLoop::new();

	let window = match winit::window::WindowBuilder::new()
		.with_title("VileTech")
		.with_min_inner_size(PhysicalSize::new(320, 200))
		.with_max_inner_size(PhysicalSize::new(7680, 4320))
		.with_inner_size(PhysicalSize::new(800, 600))
		.with_decorations(true)
		.with_resizable(true)
		.with_transparent(false)
		.with_window_icon(
			catalog
				.window_icon_from_file("/viletech/viletech.png")
				.map_err(|err| {
					warn!("Failed to load engine's window icon: {err}");
					err
				})
				.ok(),
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

	let shader = vile::gfx::create_shader_module(
		&gfx.device,
		"hello-tri",
		catalog
			.get_file("/viletech/shaders/hello-tri.wgsl")
			.expect("Engine base data validity check is compromised.")
			.try_read_str()?,
	);

	let pipeline = render::pipeline_builder("Hello Triangle", &gfx.device)
		.shader_states(vile::gfx::create_shader_states(
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
		// Remaining defaults are acceptable.
		.build();

	gfx.pipelines.push(pipeline);

	let mut core = match ClientCore::new(start_time, catalog, gfx, console) {
		Ok(c) => c,
		Err(err) => {
			eprintln!("Client init failed: {err}");
			return Err(err);
		}
	};

	event_loop.run(move |event, _, control_flow| match event {
		WinitEvent::MainEventsCleared => {
			core.main_loop(control_flow);
		}
		WinitEvent::WindowEvent {
			ref event,
			window_id,
		} if window_id == core.gfx.window.id() => {
			let resp = core.gfx.egui.state.on_event(&core.gfx.egui.context, event);

			match event {
				WindowEvent::CloseRequested => {
					*control_flow = ControlFlow::Exit;
					return;
				}
				WindowEvent::Resized(psize) => {
					core.on_window_resize(*psize);
				}
				WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
					core.on_window_resize(**new_inner_size);
				}
				WindowEvent::Focused(gained) => {
					let res = if *gained {
						core.audio.resume_all()
					} else {
						core.audio.pause_all()
					};

					if let Err(err) = res {
						let what = if *gained { "resume" } else { "pause" };
						error!("Failed to {what} all audio: {err}");
					}
				}
				_ => {}
			}

			if resp.consumed {
				return;
			}

			match event {
				WindowEvent::KeyboardInput { input, .. } => {
					if input.state == winit::event::ElementState::Pressed
						&& input.virtual_keycode == Some(VirtualKeyCode::Escape)
					{
						*control_flow = ControlFlow::Exit;
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
			let uptime = core.start_time.elapsed();
			let (hh, mm, ss) = duration_to_hhmmss(uptime);
			info!("Uptime: {hh:02}:{mm:02}:{ss:02}");
		}
		_ => {}
	});
}
