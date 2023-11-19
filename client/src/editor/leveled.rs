pub(crate) mod load;

use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_egui::egui;
use rustc_hash::FxHashMap;
use viletech::{gfx::Sky2dMaterial, vfs::FileSlot, VirtualFs};

use crate::common::InputParam;

use super::Editor;

pub(super) use load::*;

#[derive(Debug, Default)]
pub(crate) struct LevelEditor {
	pub(crate) current: Option<EditedLevel>,
	pub(crate) viewpoint: Viewpoint,

	pub(crate) materials: FxHashMap<FileSlot, Handle<StandardMaterial>>,
}

#[derive(Debug)]
pub(crate) enum EditedLevel {
	Vanilla { entity: Entity, _marker: FileSlot },
	_Udmf { entity: Entity, _marker: FileSlot },
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Viewpoint {
	#[default]
	TopDown,
	Free,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DrawMode {
	#[default]
	Bsp,
}

#[derive(Component, Debug)]
#[component(storage = "SparseSet")]
pub(crate) struct EdSector(Handle<Mesh>);

#[derive(SystemParam)]
pub(crate) struct SysParam<'w, 's> {
	pub(crate) cmds: Commands<'w, 's>,
	pub(crate) vfs: ResMut<'w, VirtualFs>,
	pub(crate) input: InputParam<'w, 's>,
	pub(crate) time: Res<'w, Time>,
	pub(crate) cameras: Query<'w, 's, &'static mut Transform, With<Camera>>,
	pub(crate) _gizmos: bevy::gizmos::gizmos::Gizmos<'s>,

	pub(crate) meshes: ResMut<'w, Assets<Mesh>>,
	pub(crate) images: ResMut<'w, Assets<Image>>,
	pub(crate) mtrs_std: ResMut<'w, Assets<StandardMaterial>>,
	pub(crate) _mtrs_sky: ResMut<'w, Assets<Sky2dMaterial>>,
}

pub(super) fn ui(ed: &mut Editor, ui: &mut egui::Ui, mut param: SysParam) {
	if ed.level_editor.current.as_ref().is_none() {
		ui.centered_and_justified(|ui| {
			ui.label("No level is currently loaded.");
		});

		return;
	};

	let mut camera = param.cameras.single_mut();

	let (mut yaw, mut pitch, roll) = camera.rotation.to_euler(EulerRot::YXZ);
	let txt_height = ui.text_style_height(&egui::TextStyle::Body);

	let overlay = egui::Window::new("")
		.id("vted_leveled_overlay".into())
		.title_bar(false)
		.anchor(egui::Align2::RIGHT_TOP, [txt_height, txt_height]);

	match ed.level_editor.viewpoint {
		Viewpoint::TopDown => {
			overlay.show(ui.ctx(), |ui| {
				ui.label(format!(
					"X: {:.2} - Y: {:.2} - Z: {:.2}",
					camera.translation.x, camera.translation.y, camera.translation.z
				));
			});
		}
		Viewpoint::Free => {
			overlay.show(ui.ctx(), |ui| {
				ui.label(format!(
					"X: {:.2} - Y: {:.2} - Z: {:.2}",
					camera.translation.x, camera.translation.y, camera.translation.z
				));

				ui.label(format!(
					"Yaw: {:.2} - Pitch: {:.2} - Roll: {:.2}",
					yaw, pitch, roll
				));
			});
		}
	}

	if !ui.rect_contains_pointer(ui.min_rect()) {
		return;
	}

	if param.input.keys.just_released(KeyCode::F) {
		ed.level_editor.viewpoint = match ed.level_editor.viewpoint {
			Viewpoint::TopDown => Viewpoint::Free,
			Viewpoint::Free => Viewpoint::TopDown,
		};
	}

	let dt = param.time.delta_seconds();

	match ed.level_editor.viewpoint {
		Viewpoint::TopDown => {
			if param.input.mouse.pressed(MouseButton::Middle) {
				let mouse_delta = param
					.input
					.mouse_motion
					.read()
					.fold(Vec2::ZERO, |v, mm| v + mm.delta);

				camera.translation.x += dt * mouse_delta.x;
				camera.translation.y -= dt * mouse_delta.y;
			}

			let speed = if param.input.keys.pressed(KeyCode::ShiftLeft) {
				12.0
			} else {
				6.0
			};

			if param.input.keys.pressed(KeyCode::Up) {
				camera.translation.y += dt * speed;
			}

			if param.input.keys.pressed(KeyCode::Down) {
				camera.translation.y -= dt * speed;
			}

			if param.input.keys.pressed(KeyCode::Right) {
				camera.translation.x += dt * speed;
			}

			if param.input.keys.pressed(KeyCode::Left) {
				camera.translation.x -= dt * speed;
			}

			for ev_mwheel in param.input.events.ev_mouse_wheel.read() {
				camera.translation.z -= ev_mwheel.y * 2.0;
			}
		}
		Viewpoint::Free => {
			let mut axis_input = Vec3::ZERO;

			if param.input.keys.pressed(KeyCode::W) {
				axis_input.z += 1.0;
			}

			if param.input.keys.pressed(KeyCode::S) {
				axis_input.z -= 1.0;
			}

			if param.input.keys.pressed(KeyCode::D) {
				axis_input.x += 1.0;
			}

			if param.input.keys.pressed(KeyCode::A) {
				axis_input.x -= 1.0;
			}

			if param.input.keys.pressed(KeyCode::Q) {
				axis_input.y += 1.0;
			}

			if param.input.keys.pressed(KeyCode::E) {
				axis_input.y -= 1.0;
			}

			let mut vel = Vec3::ZERO;

			if axis_input != Vec3::ZERO {
				vel = axis_input.normalize() * 2.0;
			}

			if param.input.keys.pressed(KeyCode::ShiftLeft) {
				vel *= 2.0;
			}

			let f = vel.z * dt * camera.forward();
			let r = vel.x * dt * camera.right();
			let u = vel.y * dt * Vec3::Y;

			camera.translation += f + r + u;

			let mut mouse_delta = Vec2::ZERO;

			for mouse_event in param.input.mouse_motion.read() {
				mouse_delta += mouse_event.delta;
			}

			if mouse_delta != Vec2::ZERO {
				pitch = (pitch - mouse_delta.y * 0.05 * dt)
					.clamp(-std::f32::consts::PI / 2.0, std::f32::consts::PI / 2.0);
				yaw -= mouse_delta.x * 0.1 * dt;

				camera.rotation = Quat::from_euler(EulerRot::ZYX, 0.0, yaw, pitch);
			}
		}
	}
}
