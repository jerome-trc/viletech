//! Internal implementation details backing [`super::event`].

use std::{
	borrow::Cow,
	path::{Path, PathBuf},
	sync::Arc,
	time::Instant,
};

use log::{error, info};
use vile::{
	data::{LoadRequest, LoadTracker},
	frontend::FrontendMenu,
	sim::{self, PlaySim},
	user::UserCore,
	utils::duration_to_hhmmss,
};

use crate::{
	ccmd,
	core::{ClientCore, Scene, Transition},
};

impl ClientCore {
	pub(super) fn scene_change(&mut self, next_scene: Transition) {
		// TODO: This needs to be an `if` expr instead of a match arm to take
		// advantage of `std::intrinsics::likely` (when it stabilizes).
		if matches!(next_scene, Transition::None) {
			return;
		}

		match next_scene {
			Transition::StartGameLoad { to_mount } => {
				self.scene = match self.mount_load_order(to_mount) {
					Ok(s) => s,
					Err(err) => {
						error!("Game load failed. Reason: {err}");

						Scene::Frontend {
							menu: FrontendMenu::default(),
						}
					}
				};
			}
			Transition::FinishGameLoad => {
				let prev = std::mem::replace(&mut self.scene, Scene::Transition);

				let (thread, time_taken) = if let Scene::GameLoad {
					thread, start_time, ..
				} = prev
				{
					(thread, start_time.elapsed())
				} else {
					unreachable!()
				};

				let results = thread.join().expect("Failed to join game load thread.");
				let mut failed = false;

				for errs in results.into_iter().filter_map(|res| res.err()) {
					failed = true;

					for err in errs {
						error!("{err}");
					}
				}

				if failed {
					self.scene = Scene::Frontend {
						menu: FrontendMenu::default(),
					};
					return;
				}

				let (hh, mm, ss) = duration_to_hhmmss(time_taken);
				info!("Game loading finished in {hh:02}:{mm:02}:{ss:02}.");

				self.scene = Scene::Game {
					sim: Some(self.start_sim()),
				};
			}
			Transition::ReturnToFrontend => {
				let scene = std::mem::replace(
					&mut self.scene,
					Scene::Frontend {
						menu: FrontendMenu::default(),
					},
				);

				match scene {
					Scene::GameLoad { thread, .. } => {
						let _ = thread.join().expect("Failed to join game load thread.");
					}
					Scene::Game { sim } => {
						if let Some(sim) = sim {
							self.end_sim(sim);
						}
					}
					_ => unreachable!("Illegal `ReturnToFrontend` transition."),
				}

				self.catalog.write().truncate(1); // Keep the base data.
			}
			Transition::FirstTimeFrontend => {
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
					// is not even possible, there's no reason to go further.

					self.user = match UserCore::new(path) {
						Ok(u) => u,
						Err(err) => panic!("User information setup failed: {err}"),
					};
				} else {
					unreachable!();
				}
			}
			// - We already checked for `Transition::None` in an `if` expr above.
			// - `Transition::Exit` should be an early-out from this function.
			Transition::None | Transition::Exit => unreachable!(),
		}
	}

	pub(super) fn mount_load_order(&self, to_mount: Vec<PathBuf>) -> Result<Scene, String> {
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

		let thread = std::thread::spawn(move || {
			let request = LoadRequest {
				paths: mounts,
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

	pub(self) fn start_sim(&mut self) -> sim::Handle {
		let (txout, rxout) = crossbeam::channel::unbounded();
		let (txin, rxin) = crossbeam::channel::unbounded();

		let catalog = self.catalog.clone();
		let runtime = self.runtime.clone();
		let sim = Arc::new(PlaySim::new(catalog, runtime, txout, rxin));

		self.console
			.enable_commands(|ccmd| ccmd.flags.contains(ccmd::Flags::SIM));

		sim::Handle {
			sim: sim.clone(),
			sender: txin,
			receiver: rxout,
			thread: std::thread::Builder::new()
				.name("vile-playsim".to_string())
				.spawn(move || vile::sim::run(sim))
				.expect("Failed to spawn OS thread for playsim."),
		}
	}

	pub(self) fn end_sim(&mut self, sim: sim::Handle) {
		self.console
			.disable_commands(|ccmd| ccmd.flags.contains(ccmd::Flags::SIM));

		sim.sender
			.send(sim::InMessage::Stop)
			.expect("Sim sender channel unexpectedly disconnected.");

		match sim.thread.join() {
			Ok(()) => {}
			Err(err) => panic!("Sim thread panicked: {err:#?}"),
		};

		// The arc-locked playsim object is meant to be dropped upon scene change,
		// so ensure no references have survived thread teardown.

		debug_assert_eq!(
			Arc::strong_count(&sim.sim),
			1,
			"Sim state has {} illegal extra references.",
			Arc::strong_count(&sim.sim) - 1
		);
	}

	pub(super) fn first_startup_screen(
		ctx: &egui::Context,
		portable: &mut bool,
		portable_path: &Path,
		home_path: &Option<PathBuf>,
	) -> Transition {
		let mut ret = Transition::None;

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
					ret = Transition::FirstTimeFrontend;
				}

				if ui.button("Exit").clicked() {
					ret = Transition::Exit;
				}
			});
		});

		ret
	}
}
