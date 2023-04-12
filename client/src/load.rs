//! Functions run when entering, updating, and leaving [`crate::AppState::Load`].

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use viletech::{data::LoadOutcome, utils::duration_to_hhmmss};

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

	let m_pct = loader.tracker.mount_progress_percent() * 100.0;
	let p_pct = loader.tracker.prep_progress_percent() * 100.0;
	let mut cancelled = false;

	egui::Window::new("Loading...")
		.id(egui::Id::new("vile_gameload"))
		.show(egui.ctx_mut(), |ui| {
			ui.label(&format!("File Mounting: {m_pct:.1}%"));
			ui.label(&format!("Preparing: {p_pct:.1}%"));

			if ui.button("Cancel").clicked() {
				cancelled = true;
			}
		});

	core.draw_devgui(egui.ctx_mut());

	if cancelled {
		loader.tracker.cancel();
		next_state.set(AppState::Frontend);
		return;
	}

	if !loader.tracker.mount_done() || !loader.tracker.prep_done() {
		return;
	}

	let res_join = loader.thread.take().unwrap().join();

	let res_load = match res_join {
		Ok(results) => results,
		Err(_) => {
			next_state.set(AppState::Frontend);
			error!("Failed to join loader thread.");
			return;
		}
	};

	let go_to_frontend = match &res_load {
		LoadOutcome::Cancelled => {
			info!("Game load cancelled.");
			true
		}
		LoadOutcome::MountFail { errors } => {
			for (i, (real_path, _)) in loader.load_order.iter().enumerate() {
				let num_errs = res_load.num_errs();
				let mut msg = String::with_capacity(128 + 256 * num_errs);

				msg.push_str(&format!(
					"{num_errs} errors/warnings while loading: {}",
					real_path.display()
				));

				msg.push_str("\r\n\r\n");

				for err in &errors[i] {
					msg.push_str(&err.to_string());
				}

				error!("{msg}");
			}

			true
		}
		LoadOutcome::PrepFail { errors } => {
			for (i, (real_path, _)) in loader.load_order.iter().enumerate() {
				let num_errs = res_load.num_errs();
				let mut msg = String::with_capacity(128 + 256 * num_errs);

				msg.push_str(&format!(
					"{num_errs} errors/warnings while loading: {}",
					real_path.display()
				));

				msg.push_str("\r\n\r\n");

				for err in &errors[i] {
					msg.push_str(&err.to_string());
				}

				error!("{msg}");
			}

			true
		}
		LoadOutcome::Ok { mount, prep } => {
			for (i, (real_path, _)) in loader.load_order.iter().enumerate() {
				let num_errs = res_load.num_errs();
				let mut msg = String::with_capacity(128 + 256 * num_errs);

				msg.push_str(&format!(
					"{num_errs} errors/warnings while loading: {}",
					real_path.display()
				));

				msg.push_str("\r\n\r\n");

				for err in &mount[i] {
					msg.push_str(&err.to_string());
				}

				for err in &prep[i] {
					msg.push_str(&err.to_string());
				}

				warn!("{msg}");
			}

			let (hh, mm, ss) = duration_to_hhmmss(loader.start_time.elapsed());
			info!("Game loading finished in {hh:02}:{mm:02}:{ss:02}.");

			false
		}
	};

	if go_to_frontend {
		next_state.set(AppState::Frontend);
	} else {
		next_state.set(AppState::Game);
	}
}

pub fn on_exit(mut cmds: Commands) {
	cmds.remove_resource::<GameLoad>();
}
