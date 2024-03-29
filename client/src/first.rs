//! Functions and state for handling first-time client startup.

use std::{borrow::Cow, path::PathBuf};

use bevy::{app::AppExit, prelude::*, render::renderer::RenderDevice, winit::WinitWindows};
use bevy_egui::egui;
use viletech::{tracing::info, user::UserCore, VirtualFs};

use crate::{
	common::{set_window_icon, ClientCommon},
	AppState,
};

#[derive(Debug, Resource)]
pub(crate) struct FirstStartup {
	/// Radio button state. `true` is the default presented to the user.
	/// `false` is not even an option if `home_path` is `None`.
	pub(crate) portable: bool,
	pub(crate) portable_path: PathBuf,
	pub(crate) home_path: Option<PathBuf>,
}

/// See [`AppState::Init`].
pub(crate) fn init_on_enter(
	startup: Option<Res<FirstStartup>>,
	mut next_state: ResMut<NextState<AppState>>,
	winits: NonSend<WinitWindows>,
	windows: Query<Entity, With<Window>>,
	vfs: Res<VirtualFs>,
	rdevice: Res<RenderDevice>,
) {
	let e_window = windows.single();
	let window_id = winits.entity_to_winit.get(&e_window).unwrap();
	let window = winits.windows.get(window_id).unwrap();
	set_window_icon(&vfs, window);

	{
		let rdev_limits = rdevice.limits();

		info!(
			concat!(
				"WGPU render device information:\n",
				"\t- Max. vertex attributes: {vattrs}\n",
				"\t- Max. 2D texture width and height: {tex2d_dim}\n",
				"\t- Max. texture array layers: {tex_arr_layers}\n",
				"\t- Max. samplers per shader stage: {samplers}\n",
				"\t- Max. sampled textures per shader stage: {sampled_tex}\n",
				"\t- Max. push constant size: {pushconst} bytes\n",
			),
			vattrs = rdev_limits.max_vertex_attributes,
			tex2d_dim = rdev_limits.max_texture_dimension_2d,
			tex_arr_layers = rdev_limits.max_texture_array_layers,
			samplers = rdev_limits.max_samplers_per_shader_stage,
			sampled_tex = rdev_limits.max_sampled_textures_per_shader_stage,
			pushconst = rdev_limits.max_push_constant_size,
		);
	}

	if startup.is_none() {
		next_state.set(AppState::Frontend);
	} else {
		next_state.set(AppState::FirstStartup);
	}
}

pub(crate) fn first_startup(
	mut cmds: Commands,
	mut startup: ResMut<FirstStartup>,
	mut core: ClientCommon,
	mut next_state: ResMut<NextState<AppState>>,
	mut exit: EventWriter<AppExit>,
) {
	// TODO: Localize these strings.

	egui::Window::new("Initial Setup").show(core.egui.ctx_mut(), |ui| {
		ui.label(
			"Select where you want user information \
			- saved games, preferences, screenshots - \
			to be stored.",
		);

		ui.separator();

		ui.horizontal(|ui| {
			ui.radio_value(&mut startup.portable, true, "Portable: ");
			let p_path = startup.portable_path.to_string_lossy();
			ui.code(p_path.as_ref());
		});

		ui.horizontal(|ui| {
			ui.add_enabled_ui(startup.home_path.is_some(), |ui| {
				let label;
				let h_path;

				if let Some(home) = &startup.home_path {
					label = "Home: ";
					h_path = home.to_string_lossy();
				} else {
					label = "No home folder found.";
					h_path = Cow::Borrowed("");
				}

				let mut portable = startup.portable;

				ui.radio_value(&mut portable, false, label);
				ui.code(h_path.as_ref());

				startup.portable = portable;
			});
		});

		ui.separator();

		ui.horizontal(|ui| {
			if ui.button("Continue").clicked() {
				let path = if startup.portable {
					startup.portable_path.clone()
				} else {
					startup.home_path.clone().unwrap()
				};

				if path.exists() {
					panic!(
						"could not create user info folder; \
						something already exists at path: {p}",
						p = path.display(),
					);
				}

				std::fs::create_dir(&path)
					.expect("user information setup failed: directory creation error");

				// If the basic file IO needed to initialize user information
				// is not even possible, there's no reason to go further.

				match UserCore::new(path) {
					Ok(u) => cmds.insert_resource(u),
					Err(err) => panic!("user information setup failed: {err}"),
				};

				next_state.set(AppState::Frontend);
			}

			if ui.button("Exit").clicked() {
				exit.send(AppExit);
			}
		});
	});
}
