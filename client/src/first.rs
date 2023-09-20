//! Functions and state for handling first-time client startup.

use std::{borrow::Cow, path::PathBuf};

use bevy::{app::AppExit, prelude::*};
use bevy_egui::{egui, EguiContexts};
use viletech::user::UserCore;

use crate::{core::ClientCore, AppState};

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
) {
	if startup.is_none() {
		next_state.set(AppState::Frontend);
	} else {
		next_state.set(AppState::FirstStartup);
	}
}

pub(crate) fn first_startup(
	mut startup: ResMut<FirstStartup>,
	mut core: ResMut<ClientCore>,
	mut ctxs: EguiContexts,
	mut next_state: ResMut<NextState<AppState>>,
	mut exit: EventWriter<AppExit>,
) {
	// TODO: Localize these strings.

	egui::Window::new("Initial Setup").show(ctxs.ctx_mut(), |ui| {
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

				core.user = match UserCore::new(path) {
					Ok(u) => u,
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
