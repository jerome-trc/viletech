//! Functions run when entering, updating, and leaving [`crate::AppState::Load`].

use std::path::Path;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use viletech::{
	catalog::{LoadOutcome, PrepError},
	util::duration_to_hhmmss,
	vfs::MountError,
};

use crate::{
	core::{ClientCore, GameLoad},
	AppState,
};

pub fn update(
	mut core: ResMut<ClientCore>,
	mut next_state: ResMut<NextState<AppState>>,
	mut loader: ResMut<GameLoad>,
	mut egui: EguiContexts,
) {
	// TODO: Localize these strings.

	let m_pct = loader.tracker_m.progress_percent() * 100.0;
	let p_pct = loader.tracker_p.progress_percent() * 100.0;
	let mut cancelled = false;

	egui::Window::new("Loading...")
		.id(egui::Id::new("viletech_gameload"))
		.show(egui.ctx_mut(), |ui| {
			ui.label(&format!("File Mounting: {m_pct:.1}%"));
			ui.label(&format!("Preparing: {p_pct:.1}%"));

			if ui.button("Cancel").clicked() {
				cancelled = true;
			}
		});

	core.draw_devgui(egui.ctx_mut());

	if cancelled {
		loader.tracker_m.cancel();
		loader.tracker_p.cancel();
	} else if !loader.tracker_m.is_done() || !loader.tracker_p.is_done() {
		return;
	}

	let res_join = loader.thread.take().unwrap().join();

	let mut res_load = match res_join {
		Ok(results) => results,
		Err(_) => {
			next_state.set(AppState::Frontend);
			error!("Failed to join loader thread.");
			return;
		}
	};

	res_load.sort_errors();

	let go_to_frontend = match &res_load {
		LoadOutcome::Ok { mount, prep } => {
			for (i, (real_path, _)) in loader.load_order.iter().enumerate() {
				if let Some(msg) = error_message(real_path, &mount[i], &prep[i]) {
					warn!("{msg}");
				}
			}

			let (hh, mm, ss) = duration_to_hhmmss(loader.start_time.elapsed());
			info!("Game loading finished in {hh:02}:{mm:02}:{ss:02}.");

			false
		}
		LoadOutcome::PrepFail { errors } => {
			for (i, (real_path, _)) in loader.load_order.iter().enumerate() {
				if let Some(msg) = error_message(real_path, &[], &errors[i]) {
					warn!("{msg}");
				}
			}

			true
		}
		LoadOutcome::MountFail { errors } => {
			for (i, (real_path, _)) in loader.load_order.iter().enumerate() {
				if let Some(msg) = error_message(real_path, &errors[i], &[]) {
					warn!("{msg}");
				}
			}

			true
		}
		LoadOutcome::Cancelled => {
			info!("Game load cancelled.");
			true
		}
		LoadOutcome::NoOp => unreachable!(),
	};

	if go_to_frontend {
		next_state.set(AppState::Frontend);
	} else {
		next_state.set(AppState::Game);
	}
}

#[must_use]
fn error_message(real_path: &Path, mount: &[MountError], prep: &[PrepError]) -> Option<String> {
	let num_errs = mount.len() + prep.len();

	if num_errs == 0 {
		return None;
	}

	let mut msg = String::with_capacity(128 + (128 * num_errs));

	msg.push_str(&format!(
		"{num_errs} errors/warnings while loading: {}",
		real_path.display()
	));

	for err in mount {
		msg.push_str("\r\n\r\n");
		msg.push_str(&err.to_string());
	}

	for err in prep {
		msg.push_str("\r\n\r\n");
		msg.push_str(&err.to_string());
	}

	msg.push_str("\r\n");

	Some(msg)
}

pub fn on_exit(mut cmds: Commands) {
	cmds.remove_resource::<GameLoad>();
}
