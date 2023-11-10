//! # VileTech Client

mod ccmd;
mod common;
mod editor;
mod first;
mod frontend;
mod game;
mod load;
mod playground;
mod setup;

use std::time::{Duration, Instant};

use bevy::{
	app::AppExit, diagnostic::LogDiagnosticsPlugin, input::InputSystem,
	pbr::wireframe::WireframePlugin, prelude::*,
};
use bevy_egui::{systems::InputEvents, EguiPlugin};
use clap::Parser;
use common::ClientCommon;
use indoc::printdoc;
use viletech::{
	audio::AudioCore,
	input::InputCore,
	tracing::info,
	user::UserCore,
	vfs::{self, VPath},
	VirtualFs,
};

use crate::{
	common::{DevGuiStatus, DeveloperGui},
	first::FirstStartup,
	playground::Playground,
	setup::LaunchArgs,
};

// TODO:
// - Pause all audio when focus is lost, and resume when focus is regained.
// - Write log messages when Winit reports application suspension or resume,
// for the benefit of diagnostics.

fn main() -> Result<(), Box<dyn std::error::Error>> {
	viletech::START_TIME.set(Instant::now()).unwrap();
	let args = LaunchArgs::parse();

	if args.version_full {
		let c_vers = env!("CARGO_PKG_VERSION");
		let [e_vers, commit, comp_datetime] = viletech::version_info();

		printdoc! {"
VileTech Client {c_vers}
{e_vers}
{commit}
{comp_datetime}
"};

		return Ok(());
	}

	viletech::thread_pool_init(args.threads);

	let mut app = App::new();

	// Common //////////////////////////////////////////////////////////////////

	app.add_plugins(LogDiagnosticsPlugin::default());
	let (log_sender, log_receiver) = crossbeam::channel::unbounded();
	app.world.insert_resource(ExitHandler);

	let mut vfs = VirtualFs(vfs::VirtualFs::default());
	vfs.mount(&viletech::basedata::path(), VPath::new("viletech"))?;
	app.world.insert_resource(vfs);
	info!("Virtual file system initialized.");

	app.add_state::<AppState>()
		.insert_resource(setup::winit_settings())
		.add_plugins(setup::default_plugins(&args, log_sender))
		.add_systems(Startup, setup::set_window_icon)
		.add_plugins((WireframePlugin, EguiPlugin))
		.add_systems(Update, common_updates)
		.add_systems(PreUpdate, update_input.in_set(InputSystem));

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

	app.insert_resource(user);
	info!("User info initialized.");
	app.insert_resource(setup::console(log_receiver));
	info!("Developer console initialized.");
	app.insert_resource(AudioCore::new(None)?);
	info!("Audio manager initialized.");
	app.insert_resource(InputCore::default());
	info!("Input manager initialized.");

	app.insert_resource(DeveloperGui {
		#[cfg(debug_assertions)]
		open: true,
		#[cfg(not(debug_assertions))]
		open: false,
		left: DevGuiStatus::Vfs,
		right: DevGuiStatus::Console,
	});

	app.insert_resource(Playground::default());

	app.add_systems(OnEnter(AppState::Init), first::init_on_enter);

	// First-time startup //////////////////////////////////////////////////////

	app.add_systems(
		Update,
		first::first_startup.run_if(in_state(AppState::FirstStartup)),
	);

	app.add_systems(OnEnter(AppState::Frontend), frontend::on_enter);
	app.add_systems(OnExit(AppState::Frontend), frontend::on_exit);
	app.add_systems(
		Update,
		frontend::update.run_if(in_state(AppState::Frontend)),
	);

	// Load ////////////////////////////////////////////////////////////////////

	app.add_systems(Update, load::update.run_if(in_state(AppState::Load)));
	app.add_systems(OnExit(AppState::Load), load::on_exit);

	// Game ////////////////////////////////////////////////////////////////////

	app.insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f32(
		1.0 / 35.0,
	)));
	app.add_systems(Update, game::update.run_if(in_state(AppState::Game)));

	/*
	app.add_systems(
		FixedUpdate,
		viletech::sim::tick.run_if(
			in_state(AppState::Game).and_then(|sim: Option<Res<viletech::sim::Sim>>| sim.is_some()),
		),
	);
	*/

	app.add_systems(OnEnter(AppState::Game), game::on_enter);
	app.add_systems(OnExit(AppState::Game), game::on_exit);

	// Editor //////////////////////////////////////////////////////////////////

	app.add_systems(Update, editor::update.run_if(in_state(AppState::Editor)));
	app.add_systems(OnEnter(AppState::Editor), editor::on_enter);
	app.add_systems(OnExit(AppState::Editor), editor::on_exit);

	// Run /////////////////////////////////////////////////////////////////////

	viletech::log::init_diag(&version_string())?;

	app.run();

	unreachable!("unexpected return from Winit event loop")
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States)]
pub(crate) enum AppState {
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
	Editor,
}

#[derive(Debug, Resource)]
struct ExitHandler;

impl Drop for ExitHandler {
	/// (RAT) In my experience, a runtime log is much more informative if it
	/// states the duration for which the program executed. Messages are already
	/// stamped with the current uptime, so just state that the program is closing.
	fn drop(&mut self) {
		info!("Shutting down.");
	}
}

#[must_use]
fn version_string() -> String {
	format!("VileTech Client {}", env!("CARGO_PKG_VERSION"))
}

fn common_updates(mut core: ClientCommon, mut exit: EventWriter<AppExit>) {
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

fn update_input(mut core: ClientCommon, input: InputEvents) {
	core.input.update(input);

	let up_pressed = core.input.keys_virt.just_pressed(KeyCode::Up);
	let down_pressed = core.input.keys_virt.just_pressed(KeyCode::Down);
	let esc_pressed = core.input.keys_virt.just_pressed(KeyCode::Escape);
	let enter_pressed = core.input.keys_virt.just_pressed(KeyCode::Return);

	core.console
		.key_input(up_pressed, down_pressed, esc_pressed, enter_pressed);
}
