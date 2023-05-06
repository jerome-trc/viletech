//! Functions run when entering, updating, and leaving [`crate::AppState::Game`].

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use viletech::{
	data::Level,
	sim::{self, Sim},
};

use crate::{core::ClientCore, AppState};

pub fn update(
	mut core: ResMut<ClientCore>,
	mut _next_state: ResMut<NextState<AppState>>,
	mut _sim: Option<ResMut<Sim>>,
	mut egui: EguiContexts,
	mut cameras: Query<&mut Transform, With<Camera>>,
) {
	for mut camera in &mut cameras {
		let mut new_pos = camera.translation;

		const CAM_SPEED: f32 = 0.1;

		if core.input.keys_virt.pressed(KeyCode::W) {
			new_pos += camera.forward() * CAM_SPEED;
		} else if core.input.keys_virt.pressed(KeyCode::S) {
			new_pos += camera.back() * CAM_SPEED;
		}

		if core.input.keys_virt.pressed(KeyCode::A) {
			new_pos += camera.left() * CAM_SPEED;
		} else if core.input.keys_virt.pressed(KeyCode::D) {
			new_pos += camera.right() * CAM_SPEED;
		}

		if core.input.keys_virt.pressed(KeyCode::Q) {
			new_pos += camera.up() * CAM_SPEED;
		} else if core.input.keys_virt.pressed(KeyCode::E) {
			new_pos += camera.down() * CAM_SPEED;
		}

		camera.translation = new_pos;

		camera.rotate_local_x((core.input.cursor_pos.y - core.input.cursor_pos_prev.y) * 0.005);
		camera.rotate_local_y((core.input.cursor_pos.x - core.input.cursor_pos_prev.x) * -0.005);
	}

	core.draw_devgui(egui.ctx_mut());
}

pub fn on_enter(core: ResMut<ClientCore>, cmds: Commands, meshes: ResMut<Assets<Mesh>>) {
	let catalog = core.catalog.read();
	let base = catalog.get_asset_handle::<Level>("DOOM/E1M1").unwrap();
	sim::start(cmds, meshes, base);
}

pub fn on_exit(mut cmds: Commands) {
	cmds.remove_resource::<Sim>();
}
