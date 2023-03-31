//! Functions run when entering, updating, and leaving [`crate::AppState::Load`].

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
	core::{ClientCore, GameLoad},
	AppState,
};

pub fn update(
	mut core: ResMut<ClientCore>,
	mut next_state: ResMut<NextState<AppState>>,
	loader: Res<GameLoad>,
	mut ctxs: EguiContexts,
) {
	// TODO: Localize these strings.

	let m_pct = loader.tracker.mount_progress_percent();
	let p_pct = loader.tracker.pproc_progress_percent();
	let mut cancelled = false;

	egui::Window::new("Loading...")
		.id(egui::Id::new("vile_gameload"))
		.show(ctxs.ctx_mut(), |ui| {
			ui.label(&format!("File Mounting: {m_pct}%"));
			ui.label(&format!("Processing: {p_pct}%"));

			if ui.button("Cancel").clicked() {
				cancelled = true;
			}
		});

	if loader.tracker.mount_done() && loader.tracker.pproc_done() && !cancelled {
		debug_assert!(loader.thread.is_finished());
		next_state.set(AppState::Game);
	} else if cancelled {
		next_state.set(AppState::Frontend);
	}

	core.draw_devgui(ctxs.ctx_mut());
}

pub fn on_exit(mut cmds: Commands) {
	cmds.remove_resource::<GameLoad>();
}
