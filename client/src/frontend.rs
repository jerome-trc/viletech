//! Functions run when entering, updating, and leaving [`crate::AppState::Frontend`].

use std::{path::PathBuf, sync::Arc, time::Instant};

use bevy::{app::AppExit, prelude::*};
use bevy_egui::EguiContexts;
use viletech::{
	data::{LoadRequest, LoadTracker},
	frontend::{FrontendAction, FrontendMenu},
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
	mut ctxs: EguiContexts,
	mut exit: EventWriter<AppExit>,
) {
	let action = frontend.ui(ctxs.ctx_mut());

	match action {
		FrontendAction::None => {}
		FrontendAction::Quit => {
			exit.send(AppExit);
			return;
		}
		FrontendAction::Start => {
			let to_mount = frontend.to_mount();
			let to_mount = to_mount.into_iter().map(|p| p.to_path_buf()).collect();
			cmds.insert_resource(core.start_load(to_mount).unwrap_or_else(|_| {
				unimplemented!("Handling load order errors is currently unimplemented.")
			}));
			next_state.set(AppState::Load);
		}
	}

	core.draw_devgui(ctxs.ctx_mut());
}

pub fn on_enter(mut cmds: Commands) {
	cmds.insert_resource(FrontendMenu::default());
}

pub fn on_exit(mut cmds: Commands) {
	cmds.remove_resource::<FrontendMenu>();
}

impl ClientCore {
	fn start_load(&self, to_mount: Vec<PathBuf>) -> Result<GameLoad, String> {
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

		let mounts_sent = mounts.clone();

		let thread = std::thread::spawn(move || {
			let request = LoadRequest {
				paths: mounts_sent,
				tracker: Some(tracker_sent),
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
