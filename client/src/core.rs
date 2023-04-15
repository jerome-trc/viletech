//! The struct representing the application's state and its related symbols.

use std::{path::PathBuf, sync::Arc, thread::JoinHandle, time::Instant};

use bevy::prelude::*;
use bevy_egui::egui;
use nanorand::WyRand;
use parking_lot::RwLock;
use viletech::{
	audio::AudioCore,
	console::Console,
	data::{Catalog, CatalogAL, LoadOutcome, LoadTracker},
	input::InputCore,
	rng::RngCore,
	user::UserCore,
	vzs,
};

use crate::ccmd;

pub type DeveloperGui = viletech::devgui::DeveloperGui<DevGuiStatus>;

#[derive(Debug, Resource)]
pub struct ClientCore {
	pub audio: AudioCore,
	pub catalog: CatalogAL,
	pub console: Console<ccmd::Command>,
	pub devgui: DeveloperGui,
	pub input: InputCore,
	pub runtime: vzs::Runtime,
	pub rng: RngCore<WyRand>,
	pub user: UserCore,
}

impl ClientCore {
	pub fn new(
		catalog: Catalog,
		console: Console<ccmd::Command>,
		user: UserCore,
	) -> Result<Self, Box<dyn std::error::Error>> {
		let catalog = Arc::new(RwLock::new(catalog));
		let catalog_audio = catalog.clone();
		let audio = AudioCore::new(catalog_audio, None)?;

		let mut ret = Self {
			audio,
			catalog,
			console,
			devgui: DeveloperGui {
				#[cfg(debug_assertions)]
				open: true,
				#[cfg(not(debug_assertions))]
				open: false,
				left: DevGuiStatus::Vfs,
				right: DevGuiStatus::Console,
			},
			input: InputCore::default(),
			runtime: vzs::Runtime::default(),
			rng: RngCore::default(),
			user,
		};

		ret.register_console_commands();

		Ok(ret)
	}

	fn register_console_commands(&mut self) {
		self.console.register_command(
			"alias",
			ccmd::Command {
				func: ccmd::ccmd_alias,
			},
			true,
		);

		self.console.register_command(
			"args",
			ccmd::Command {
				func: ccmd::ccmd_args,
			},
			true,
		);

		self.console.register_command(
			"clear",
			ccmd::Command {
				func: ccmd::ccmd_clear,
			},
			true,
		);

		self.console.register_command(
			"exit",
			ccmd::Command {
				func: ccmd::ccmd_exit,
			},
			true,
		);

		self.console.register_command(
			"hclear",
			ccmd::Command {
				func: ccmd::ccmd_hclear,
			},
			true,
		);

		self.console.register_command(
			"help",
			ccmd::Command {
				func: ccmd::ccmd_help,
			},
			true,
		);

		self.console.register_command(
			"version",
			ccmd::Command {
				func: ccmd::ccmd_version,
			},
			true,
		);

		self.console
			.register_alias("quit".to_string(), "exit".to_string());
	}

	pub fn draw_devgui(&mut self, ctx: &mut egui::Context) {
		// TODO:
		// - Developer GUI toggle key-binding.
		// - Localize these strings?
		if self.input.keys_virt.just_pressed(KeyCode::Grave) {
			self.devgui.open = !self.devgui.open;
		}

		if !self.devgui.open {
			return;
		}

		let mut devgui_open = true;
		let screen_rect = ctx.input(|inps| inps.screen_rect);

		DeveloperGui::window(ctx)
			.open(&mut devgui_open)
			.show(ctx, |ui| {
				// Prevent window from overflowing off the screen's sides
				ui.set_max_width(screen_rect.width());

				egui::menu::bar(ui, |ui| {
					self.devgui.selectors(
						ui,
						&[
							(DevGuiStatus::Assets, "Assets"),
							(DevGuiStatus::Audio, "Audio"),
							(DevGuiStatus::Console, "Console"),
							(DevGuiStatus::VzsRepl, "REPL"),
							(DevGuiStatus::Vfs, "VFS"),
						],
					);
				});

				self.devgui.panel_left(ctx).show_inside(ui, |ui| {
					match self.devgui.left {
						DevGuiStatus::Assets => {
							self.catalog.read().ui_assets(ctx, ui);
						}
						DevGuiStatus::Audio => {
							self.audio.ui(ctx, ui);
						}
						DevGuiStatus::Console => {
							self.console.ui(ctx, ui);
						}
						DevGuiStatus::VzsRepl => {
							// Soon!
						}
						DevGuiStatus::Vfs => {
							self.catalog.read().vfs().ui(ctx, ui);
						}
					};
				});

				self.devgui.panel_right(ctx).show_inside(ui, |ui| {
					match self.devgui.right {
						DevGuiStatus::Assets => {
							self.catalog.read().ui_assets(ctx, ui);
						}
						DevGuiStatus::Audio => {
							self.audio.ui(ctx, ui);
						}
						DevGuiStatus::Console => {
							self.console.ui(ctx, ui);
						}
						DevGuiStatus::VzsRepl => {
							// Soon!
						}
						DevGuiStatus::Vfs => {
							self.catalog.read().vfs().ui(ctx, ui);
						}
					};
				});
			});

		self.devgui.open = devgui_open;
	}
}

impl Drop for ClientCore {
	/// (RAT) In my experience, a runtime log is much more informative if it
	/// states the duration for which the program executed. Messages are already
	/// stamped with the current uptime, so just state that the program is closing.
	fn drop(&mut self) {
		info!("Shutting down.");
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevGuiStatus {
	Assets,
	Audio,
	Console,
	VzsRepl,
	Vfs,
}

#[derive(Debug, Resource)]
pub struct FirstStartup {
	/// Radio button state. `true` is the default presented to the user.
	/// `false` is no even an option if `home_path` is `None`.
	pub portable: bool,
	pub portable_path: PathBuf,
	pub home_path: Option<PathBuf>,
}

#[derive(Debug, Resource)]
pub struct GameLoad {
	/// The mount thread takes a write guard to the catalog and another
	/// pointer to `tracker`. This is `Some` from initialization up until it
	/// gets taken to be joined.
	pub thread: Option<JoinHandle<LoadOutcome>>,
	/// How far along the mount/load process is `thread`?
	pub tracker: Arc<LoadTracker>,
	/// Print to the log how long the mount takes for diagnostic purposes.
	pub start_time: Instant,
	pub load_order: Vec<(PathBuf, String)>,
}
