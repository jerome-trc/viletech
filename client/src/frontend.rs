//! Functions run when entering, updating, and leaving [`crate::AppState::Frontend`].

use std::{path::PathBuf, sync::Arc, time::Instant};

use bevy::{app::AppExit, prelude::*, window::WindowFocused};
use bevy_egui::EguiContexts;
use viletech::{
	data::{LoadRequest, LoadTracker},
	frontend::{FrontendMenu, Outcome},
};

use crate::{
	core::{ClientCore, GameLoad},
	AppState,
};

pub fn update(
	mut cmds: Commands,
	mut core: ResMut<ClientCore>,
	mut next_state: ResMut<NextState<AppState>>,
	mut frontend: ResMut<FrontendMenu>,
	mut egui: EguiContexts,
	mut exit: EventWriter<AppExit>,
	mut focus: EventReader<WindowFocused>,
) {
	// When re-focusing the window, check to ensure the end user has not deleted
	// or moved any of their load order items.
	for event in focus.iter() {
		if event.focused {
			frontend.validate();
			break;
		}
	}

	let action = frontend.ui(egui.ctx_mut());

	match action {
		Outcome::None => {}
		Outcome::Start => {
			let to_mount = frontend.to_mount();
			let to_mount = to_mount.into_iter().map(|p| p.to_path_buf()).collect();

			cmds.insert_resource(
				core.start_load(to_mount, frontend.dev_mode())
					.unwrap_or_else(|_| {
						unimplemented!("Handling load order errors is currently unimplemented.")
					}),
			);

			next_state.set(AppState::Load);
		}
		Outcome::Exit => {
			exit.send(AppExit);
			on_exit(cmds, frontend, core);
			return;
		}
	}

	core.draw_devgui(egui.ctx_mut());
}

pub fn on_enter(mut cmds: Commands, core: Res<ClientCore>) {
	let globalcfg = core.user.globalcfg();

	cmds.insert_resource(FrontendMenu::new(
		Some((
			globalcfg.load_order_presets.clone(),
			globalcfg.cur_load_order_preset,
		)),
		globalcfg.dev_mode,
	));
}

pub fn on_exit(
	mut cmds: Commands,
	mut frontend: ResMut<FrontendMenu>,
	mut core: ResMut<ClientCore>,
) {
	cmds.remove_resource::<FrontendMenu>();

	let globalcfg = core.user.globalcfg_mut();
	globalcfg.dev_mode = frontend.dev_mode();

	let (loadord_presets, cur_preset) = frontend.consume();
	globalcfg.load_order_presets = loadord_presets;
	globalcfg.cur_load_order_preset = cur_preset;

	if let Err(err) = core.user.write_global_cfg() {
		error!(
			"Failed to write to global config file: {p}\r\n\tDetails: {err}",
			p = core.user.globalcfg_path().display()
		);
	}
}

impl ClientCore {
	fn start_load(&self, mut to_mount: Vec<PathBuf>, dev_mode: bool) -> Result<GameLoad, String> {
		let start_time = Instant::now();
		let catalog = self.catalog.clone();
		let tracker = Arc::new(LoadTracker::default());

		to_mount.dedup();

		let mut mounts = Vec::with_capacity(to_mount.len());

		for real_path in to_mount {
			debug_assert!(real_path.exists());

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

		let mounts_sent = mounts.clone();

		let thread = std::thread::spawn(move || {
			let request = LoadRequest {
				load_order: mounts_sent,
				tracker: Some(tracker_sent),
				dev_mode,
			};

			catalog.write().load(request)
		});

		Ok(GameLoad {
			thread: Some(thread),
			tracker,
			start_time,
			load_order: mounts,
		})
	}
}
