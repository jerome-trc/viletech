//! # VileTech Client

mod ccmd;
mod common;
mod dgui;
mod editor;
mod first;
mod frontend;
mod game;
mod load;
mod playground;
mod setup;
mod types;

use std::time::{Duration, Instant};

use bevy::{
	diagnostic::LogDiagnosticsPlugin, ecs::schedule::Condition, input::InputSystem,
	pbr::wireframe::WireframePlugin, prelude::*,
};
use bevy_egui::EguiPlugin;
use clap::Parser;
use viletech::{
	audio::AudioCore,
	tracing::info,
	user::UserCore,
	vfs::{self, VPath},
	VirtualFs,
};

use crate::{first::FirstStartup, playground::Playground, setup::LaunchArgs};

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
		print!("VileTech Client {c_vers}\n{e_vers}\n{commit}\n{comp_datetime}\n",);
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
	let vfs_root_slot = vfs.root().slot();
	app.world.insert_resource(vfs);
	info!("Virtual file system initialized.");

	app.add_state::<AppState>()
		.insert_resource(setup::winit_settings())
		.add_plugins(setup::default_plugins(&args, log_sender))
		.add_plugins((WireframePlugin, viletech::gfx::GraphicsPlugin, EguiPlugin));

	app.add_event::<editor::Event>()
		.add_event::<common::NewWindow>();

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

	app.insert_resource(dgui::State {
		vfs_selection: vfs::Slot::Folder(vfs_root_slot),
	});

	app.insert_resource(user);
	info!("User info initialized.");
	app.insert_resource(setup::console(log_receiver));
	info!("Developer console initialized.");
	app.insert_resource(AudioCore::new(None)?);
	info!("Audio manager initialized.");
	app.insert_resource(Playground::default());
	info!("Lithica scripting playground initialized.");

	app.add_systems(Startup, dgui::on_app_startup)
		.add_systems(Update, common::update)
		.add_systems(PreUpdate, common::pre_update.in_set(InputSystem))
		.add_systems(PostUpdate, common::post_update)
		.add_systems(OnEnter(AppState::Init), first::init_on_enter)
		.add_systems(
			Update,
			dgui::draw
				.run_if(
					not(in_state(AppState::Init)).and_then(not(in_state(AppState::FirstStartup))),
				)
				.after(frontend::update)
				.after(game::update)
				.after(editor::update),
		);

	// First-time startup //////////////////////////////////////////////////////

	app.add_systems(
		Update,
		first::first_startup.run_if(in_state(AppState::FirstStartup)),
	);

	// Frontend ////////////////////////////////////////////////////////////////

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
	app.add_systems(
		PostUpdate,
		editor::post_update.run_if(in_state(AppState::Editor)),
	);
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
