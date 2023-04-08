//! Functions run when entering, updating, and leaving [`crate::AppState::Game`].

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use viletech::sim::Sim;

use crate::{core::ClientCore, AppState};

pub fn update(
	mut core: ResMut<ClientCore>,
	mut _next_state: ResMut<NextState<AppState>>,
	mut _sim: Option<ResMut<Sim>>,
	mut egui: EguiContexts,
) {
	core.draw_devgui(egui.ctx_mut());
}

pub fn on_exit(mut cmds: Commands) {
	cmds.remove_resource::<Sim>();
}
