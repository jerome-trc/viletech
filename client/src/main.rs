//! # VileTech Client

mod ccmd;
mod core;
mod frontend;
mod game;
mod load;

use std::{
	borrow::Cow,
	path::PathBuf,
	time::{Duration, Instant},
};

use bevy::{
	app::AppExit,
	diagnostic::LogDiagnosticsPlugin,
	input::InputSystem,
	log::LogPlugin,
	prelude::*,
	window::WindowMode,
	winit::{UpdateMode, WinitSettings},
};
use bevy_egui::{egui, systems::InputEvents, EguiContexts, EguiPlugin};
use clap::Parser;
use indoc::printdoc;
use viletech::{console::Console, data::Catalog, log::TracingPlugin, user::UserCore};

use self::core::*;

// TODO:
// - Pause all audio when focus is lost, and resume when focus is regained.
// - Write log messages when Winit reports application suspension or resume,
// for the benefit of diagnostics.

fn main() -> Result<(), Box<dyn std::error::Error>> {
	viletech::START_TIME.set(Instant::now()).unwrap();
	let args = Clap::parse();

	if args.version {
		println!("{}", viletech::short_version_string());
		println!("{}", &version_string());
		return Ok(());
	}

	if args.about {
		printdoc! {"
VileTech Client - Copyright (C) 2022-2023 - jerome-trc

This program comes with ABSOLUTELY NO WARRANTY.

This is free software, and you are welcome to redistribute it under certain
conditions. See the license document that comes with your installation."
		};

		return Ok(());
	}

	viletech::thread_pool_init(args.threads);

	let mut app = App::new();

	// Common //////////////////////////////////////////////////////////////////

	app.add_plugin(LogDiagnosticsPlugin::default());

	let (log_sender, log_receiver) = crossbeam::channel::unbounded();

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
				})
				.disable::<LogPlugin>()
				.disable::<bevy::input::InputPlugin>()
				.add_before::<WindowPlugin, _>(viletech::input::InputPlugin)
				.add_before::<TaskPoolPlugin, _>(TracingPlugin {
					console_sender: Some(log_sender),
					level: args.verbosity,
					..Default::default()
				}),
		)
		.add_plugin(EguiPlugin)
		.add_system(common_updates)
		.add_system(update_input.in_set(InputSystem));

	let catalog = Catalog::new([(viletech::basedata_path(), PathBuf::from("/viletech"))]);

	let user_dir_portable = viletech::user::user_dir_portable();
	let user_dir_home = viletech::user::user_dir_home();
	let user_dir = viletech::user::select_user_dir(&user_dir_portable, &user_dir_home);

	let user = if let Some(udir) = user_dir {
		UserCore::new(udir)?
	} else {
		app.insert_resource(FirstStartup {
			portable: true,
			portable_path: user_dir_portable,
			home_path: user_dir_home,
		});

		UserCore::uninit()
	};

	app.insert_resource(ClientCore::new(catalog, Console::new(log_receiver), user)?);

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
				.run_if(|sim: Option<Res<viletech::sim::Sim>>| sim.is_some())
				.in_schedule(CoreSchedule::FixedUpdate),
		)
		.add_system(game::on_exit.in_schedule(OnExit(AppState::Game)));

	// Run /////////////////////////////////////////////////////////////////////

	viletech::log::init_diag(&version_string())?;

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

/// See [`AppState::Init`].
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

fn common_updates(mut core: ResMut<ClientCore>, mut exit: EventWriter<AppExit>) {
	core.audio.update();

	while !core.console.requests.is_empty() {
		match core.console.requests.pop_front().unwrap() {
			ccmd::Request::Callback(func) => {
				(func)(&mut core);
			}
			ccmd::Request::Exit => {
				exit.send(AppExit);
			}
			ccmd::Request::None => {}
		}
	}
}

fn update_input(mut core: ResMut<ClientCore>, input: InputEvents) {
	core.input.update(input);
	// TODO: Console keyboard input.
	// Requires either a splitting borrow or turning `ClientCore` into a `SystemParam`.
}

#[derive(Debug, clap::Parser)]
struct Clap {
	/// Prints the client and engine versions and then exits.
	#[arg(short = 'V', long = "version")]
	version: bool,
	/// Prints license information and then exits.
	#[arg(short = 'A', long = "about")]
	about: bool,
	/// Sets the number of threads used by the global thread pool
	///
	/// If set to 0 or not set, this will be automatically selected based on the
	/// number of logical CPUs your computer has.
	#[arg(short, long)]
	threads: Option<usize>,
	/// Sets how much logging goes to stdout, the console, and log files.
	///
	/// Possible values: ERROR, WARN, INFO, DEBUG, or TRACE.
	#[arg(short, long, default_value_t = viletech::log::Level::INFO)]
	verbosity: viletech::log::Level,
}
