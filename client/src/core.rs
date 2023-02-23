//! The struct representing the application's state and its related symbols.

mod detail;
mod event;
mod init;

use std::{path::PathBuf, sync::Arc, thread::JoinHandle, time::Instant};

use crate::ccmd::Command as ConsoleCommand;
use nanorand::WyRand;
use parking_lot::RwLock;
use vile::{
	audio::AudioCore,
	console::Console,
	data::{Catalog, LoadError, LoadTracker},
	frontend::FrontendMenu,
	gfx::{camera::Camera, core::GraphicsCore},
	input::InputCore,
	lith,
	rng::RngCore,
	sim::{self},
	user::UserCore,
};

type DeveloperGui = vile::DeveloperGui<DevGuiStatus>;

/// All of the client application's state wrapped up for ease of use.
/// `main` only manipulates this through methods defined in [`crate::event`].
#[derive(Debug)]
pub struct ClientCore {
	/// (RAT) In my experience, a runtime log is much more informative if it
	/// states the duration for which the program executed.
	pub start_time: Instant,
	pub user: UserCore,
	pub catalog: Arc<RwLock<Catalog>>,
	pub runtime: Arc<RwLock<lith::Runtime>>,
	pub gfx: GraphicsCore,
	pub audio: AudioCore,
	pub input: InputCore,
	/// Kept behind an arc-lock in case the client's script API ends up needing
	/// to call into it from multiple threads. If this proves to never happen,
	/// it will be unwrapped.
	pub rng: RngCore<WyRand>,
	pub console: Console<ConsoleCommand>,
	// TODO: A menu stack.
	pub camera: Camera,
	pub(self) devgui: DeveloperGui,
	pub(self) scene: Scene,
}

/// The general status of the entire client application.
///
/// Also see [`Transition`] to understand the paths of the client state machine.
#[derive(Debug)]
pub(self) enum Scene {
	/// The user needs to choose whether their information should be stored
	/// portably or at a "home" directory.
	FirstStartup {
		/// Radio button state. `true` is the default presented to the user.
		/// `false` is no even an option if `home_path` is `None`.
		portable: bool,
		portable_path: PathBuf,
		home_path: Option<PathBuf>,
	},
	/// The user has not entered the game yet. From here they can select a user
	/// profile and engine-global/cross-game preferences, assemble a load order,
	/// and begin the game launch process.
	Frontend { menu: FrontendMenu },
	/// A loading screen which draws progress bars.
	GameLoad {
		/// The mount thread takes a write guard to the catalog and another
		/// pointer to `tracker`.
		thread: JoinHandle<Vec<Result<(), Vec<LoadError>>>>,
		/// How far along the mount/load process is `thread`?
		tracker: Arc<LoadTracker>,
		/// Print to the log how long the mount takes for diagnostic purposes.
		start_time: Instant,
	},
	/// The title screen, gameplay, intermissions, casting calls...
	/// - `sim` is only `Some` at the title if the game calls for a title map.
	/// If the script controlling game flow chooses, it can last into gameplay,
	/// but in most cases it will be swapped out with an entirely new instance.
	/// - `sim` stays between intermissions, although much of its state is altered.
	/// - `sim` is put back to `None` when the game finishes and the cast call starts.
	Game { sim: Option<sim::Handle> },
	/// A temporary value. Sometimes it is necessary to extract the current scene
	/// variant and not replace it with another valid variant until some validation
	/// has been performed.
	Transition,
}

#[derive(Debug)]
pub(self) enum Transition {
	None,
	/// The user has requested an immediate exit from any other scene.
	/// Stop everything, drop everything, and close the window post-haste.
	Exit,

	/// From [`Scene::FirstStartup`] to [`Scene::Frontend`].
	FirstTimeFrontend,

	/// From [`Scene::Frontend`] to [`Scene::GameLoad`].
	StartGameLoad {
		/// The user's load order. Gets handed off to the mount thread.
		to_mount: Vec<PathBuf>,
	},
	/// From [`Scene::GameLoad`] to [`Scene::Game`].
	FinishGameLoad,

	/// From [`Scene::Game`] or [`Scene::GameLoad`].
	ReturnToFrontend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(self) enum DevGuiStatus {
	Console,
	LithRepl,
	Vfs,
	Graphics,
	Audio,
}
