//! Functions run when entering, updating, and leaving [`AppState::Game`].

use bevy::prelude::*;
use bevy_egui::egui;

use crate::{common::ClientCommon, AppState};

pub(crate) fn update(
	mut core: ClientCommon,
	mut _next_state: ResMut<NextState<AppState>>,
	mut cameras: Query<&Transform, With<Camera>>,
) {
	let camera = cameras.get_single_mut().unwrap();

	egui::Window::new("")
		.id("viletech_devoverlay_pos".into())
		.title_bar(false)
		.show(core.egui.ctx_mut(), |ui| {
			ui.label(format!(
				"{} {} {}",
				camera.translation.x, camera.translation.y, camera.translation.z
			));
		});
}

pub(crate) fn on_enter() {
	// TODO: add `Sim` resource.
}

pub(crate) fn on_exit(_: Commands) {
	// TODO: remove `Sim` resource.
}
