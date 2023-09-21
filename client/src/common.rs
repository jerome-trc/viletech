use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_egui::{egui, EguiContexts};
use viletech::{audio::AudioCore, catalog::Catalog, console::Console, input::InputCore, util};

use crate::ccmd;

pub(crate) type DeveloperGui = viletech::devgui::DeveloperGui<DevGuiStatus>;

#[derive(SystemParam)]
pub(crate) struct ClientCommon<'w, 's> {
	pub(crate) catalog: ResMut<'w, Catalog>,
	pub(crate) input: ResMut<'w, InputCore>,
	pub(crate) audio: ResMut<'w, AudioCore>,
	pub(crate) console: ResMut<'w, Console<ccmd::Command>>,
	pub(crate) devgui: ResMut<'w, DeveloperGui>,
	pub(crate) egui: EguiContexts<'w, 's>,
}

impl ClientCommon<'_, '_> {
	pub(crate) fn draw_devgui(&mut self) {
		let ctx = self.egui.ctx_mut();

		// TODO:
		// - Developer GUI toggle key-binding.
		// - Localize these strings?
		if self.input.keys_virt.just_pressed(KeyCode::Grave) {
			self.devgui.open = !self.devgui.open;
		}

		if !self.devgui.open {
			return;
		}

		let mut devgui_open = true;
		let screen_rect = ctx.input(|inps| inps.screen_rect);

		DeveloperGui::window(ctx)
			.open(&mut devgui_open)
			.show(ctx, |ui| {
				// Prevent window from overflowing off the screen's sides.
				ui.set_max_width(screen_rect.width());

				egui::menu::bar(ui, |ui| {
					let uptime = viletech::START_TIME.get().unwrap().elapsed();
					let (hh, mm, ss) = util::duration_to_hhmmss(uptime);

					ui.label(format!("{hh:02}:{mm:02}:{ss:02}"))
						.on_hover_ui(|ui| {
							ui.label(
								format!("Engine has been running for {hh:02} hours, {mm:02} minutes, {ss:02} seconds")
							);
						});

					ui.separator();

					self.devgui.selectors(
						ui,
						&[
							(DevGuiStatus::Audio, "Audio"),
							(DevGuiStatus::Console, "Console"),
							(DevGuiStatus::Catalog, "Data"),
							(DevGuiStatus::VzsRepl, "REPL"),
							(DevGuiStatus::Vfs, "VFS"),
						],
					);
				});

				self.devgui.panel_left(ctx).show_inside(ui, |ui| {
					match self.devgui.left {
						DevGuiStatus::Audio => {
							self.audio.ui(ctx, ui, &self.catalog);
						}
						DevGuiStatus::Catalog => {
							self.catalog.ui(ctx, ui);
						}
						DevGuiStatus::Console => {
							self.console.ui(ctx, ui);
						}
						DevGuiStatus::VzsRepl => {
							// Soon!
						}
						DevGuiStatus::Vfs => {
							self.catalog.vfs_mut().ui(ctx, ui);
						}
					}
				});

				self.devgui.panel_right(ctx).show_inside(ui, |ui| {
					match self.devgui.right {
						DevGuiStatus::Audio => {
							self.audio.ui(ctx, ui, &self.catalog);
						}
						DevGuiStatus::Catalog => {
							self.catalog.ui(ctx, ui);
						}
						DevGuiStatus::Console => {
							self.console.ui(ctx, ui);
						}
						DevGuiStatus::VzsRepl => {
							// Soon!
						}
						DevGuiStatus::Vfs => {
							self.catalog.vfs_mut().ui(ctx, ui);
						}
					}
				});
			});

		self.devgui.open = devgui_open;
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DevGuiStatus {
	Audio,
	Catalog,
	Console,
	VzsRepl,
	Vfs,
}

impl std::fmt::Display for DevGuiStatus {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			DevGuiStatus::Audio => write!(f, "Audio"),
			DevGuiStatus::Catalog => write!(f, "Catalog"),
			DevGuiStatus::Console => write!(f, "Console"),
			DevGuiStatus::VzsRepl => write!(f, "VZScript REPL"),
			DevGuiStatus::Vfs => write!(f, "VFS"),
		}
	}
}
