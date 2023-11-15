//! The developer/debug GUI, which pairs the console with other useful functions.

use bevy::{
	ecs::system::{EntityCommands, SystemParam},
	prelude::*,
};
use bevy_egui::{egui, EguiContexts};
use viletech::{
	audio::AudioCore,
	input::InputCore,
	util::{self, string::subdivide_file_len},
	vfs::{self, VPathBuf},
	VirtualFs,
};

use crate::{ccmd, playground::Playground};

pub(crate) type Console = viletech::console::Console<ccmd::Command>;

/// The developer/debug GUI, which pairs the console with other useful functions.
#[derive(Debug, Component)]
pub(crate) struct DevGui {
	pub(crate) open: bool,
	pub(crate) side: SideMenu,
}

#[derive(Debug, Resource)]
pub(crate) struct State {
	pub(crate) vfs_selection: vfs::Slot,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SideMenu {
	Audio,
	Lith,
	#[default]
	Vfs,
}

impl std::fmt::Display for SideMenu {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Audio => write!(f, "Audio"),
			Self::Lith => write!(f, "Script Playground"),
			Self::Vfs => write!(f, "VFS"),
		}
	}
}

#[derive(SystemParam)]
pub(crate) struct SysParam<'w, 's> {
	pub(crate) egui: EguiContexts<'w, 's>,
	pub(crate) dguis: Query<'w, 's, (Entity, &'static Window, &'static mut DevGui)>,

	pub(crate) audio: ResMut<'w, AudioCore>,
	pub(crate) console: ResMut<'w, Console>,
	pub(crate) input: ResMut<'w, InputCore>,
	pub(crate) playground: ResMut<'w, Playground>,
	pub(crate) vfs: ResMut<'w, VirtualFs>,
}

pub(crate) fn draw(mut param: SysParam, mut state: ResMut<State>) {
	let toggle_key = param.input.keys_virt.just_pressed(KeyCode::Grave);

	for (e_window, window, mut dgui) in &mut param.dguis {
		if window.focused && toggle_key {
			dgui.open = !dgui.open;
		}

		if !dgui.open {
			continue;
		}

		let ctx = param.egui.ctx_for_window_mut(e_window);
		let screen_rect = ctx.input(|inps| inps.screen_rect);
		let mut dgui_open = dgui.open;

		egui::Window::new("Developer Tools")
			.id(egui::Id::new("viletech_dgui"))
			.anchor(egui::Align2::CENTER_TOP, [0.0, 0.0])
			.fixed_pos([0.0, 0.0])
			.collapsible(false)
			.resizable(true)
			.min_width(screen_rect.width())
			.min_height(screen_rect.height() * 0.1)
			.frame(egui::Frame::window(&ctx.style()).multiply_with_opacity(0.8))
			.open(&mut dgui_open)
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

					side_menu_selector(
						&mut dgui,
						ui,
						&[
							(SideMenu::Audio, "Audio"),
							(SideMenu::Lith, "Lithica Playground"),
							(SideMenu::Vfs, "VFS"),
						],
					);
				});

				egui::SidePanel::left("viletech_dgui_console")
					.default_width(screen_rect.width() * 0.5)
					.resizable(true)
					.width_range((screen_rect.width() * 0.1)..=(screen_rect.width() * 0.9))
					.frame(egui::Frame::side_top_panel(&ctx.style()).multiply_with_opacity(0.8))
					.show_inside(ui, |ui| {
						param.console.ui(ctx, ui);
					});

				egui::CentralPanel::default()
					.frame(egui::Frame::central_panel(&ctx.style()).multiply_with_opacity(0.8))
					.show_inside(ui, |ui| match dgui.side {
						SideMenu::Audio => {
							param.audio.ui(ctx, ui, &param.vfs);
						}
						SideMenu::Lith => {
							param.playground.ui(ctx, ui);
						}
						SideMenu::Vfs => {
							ui_vfs(ui, &mut state, &mut param.vfs);
						}
					});
			});

		dgui.open = dgui_open;
	}
}

pub(crate) fn on_app_startup(
	mut cmds: Commands,
	windows: Query<Entity, (With<Window>, Without<DevGui>)>,
) {
	for window in &windows {
		add_to_window(cmds.entity(window));
	}
}

pub(crate) fn add_to_window(mut ecmds: EntityCommands) {
	ecmds.insert(DevGui {
		#[cfg(debug_assertions)]
		open: true,
		#[cfg(not(debug_assertions))]
		open: false,
		side: SideMenu::default(),
	});
}

fn ui_vfs(ui: &mut egui::Ui, state: &mut State, vfs: &mut VirtualFs) {
	fn nav(ui: &mut egui::Ui, state: &mut State, fref: vfs::Ref) {
		if fref.slot() == fref.vfs().root().slot() {
			ui.label("/");
			return;
		}

		ui.horizontal(|ui| {
			for (i, comp) in fref.path().components().enumerate() {
				ui.label("/");

				let label = egui::Label::new(comp.as_str()).sense(egui::Sense::click());

				let resp = ui.add(label);

				let resp = if resp.hovered() {
					resp.highlight()
				} else {
					resp
				};

				if resp.clicked() {
					let p: VPathBuf = fref.path().components().take(i + 1).collect();
					state.vfs_selection = fref.vfs().lookup(&p).unwrap().slot();
				}

				resp.on_hover_text("Go to");
			}
		});
	}

	ui.heading("Virtual File System");

	let sel_valid = match state.vfs_selection {
		vfs::Slot::File(islot) => vfs.file_exists(islot),
		vfs::Slot::Folder(oslot) => vfs.folder_exists(oslot),
	};

	if !sel_valid {
		state.vfs_selection = vfs::Slot::Folder(vfs.root().slot());
	}

	egui::ScrollArea::vertical().show(ui, |ui| match state.vfs_selection {
		vfs::Slot::File(islot) => {
			let vfile = vfs.get_file(islot).unwrap();
			nav(ui, state, vfs::Ref::File(vfile));

			if vfile.size() != 0 {
				ui.label(&subdivide_file_len(vfile.size()));
			} else {
				ui.label("Empty");
			}
		}
		vfs::Slot::Folder(oslot) => {
			let vfolder = vfs.get_folder(oslot).unwrap();
			nav(ui, state, vfs::Ref::Folder(vfolder));

			ui.horizontal(|ui| {
				ui.label("Folder");

				ui.separator();

				match vfolder.subfolder_count() {
					0 => {
						ui.label("No subfolders");
					}
					1 => {
						ui.label("1 subfolder");
					}
					n => {
						ui.label(&format!("{n} subfolders"));
					}
				}

				ui.separator();

				match vfolder.file_count() {
					0 => {
						ui.label("No files");
					}
					1 => {
						ui.label("1 file");
					}
					n => {
						ui.label(&format!("{n} files"));
					}
				}
			});

			for child in vfolder.children() {
				ui.horizontal(|ui| {
					if child.is_folder() {
						ui.weak("/");
					}

					let resp = ui.add(egui::Label::new(child.name()).sense(egui::Sense::click()));

					let resp = if resp.hovered() {
						resp.highlight()
					} else {
						resp
					};

					if resp.clicked() {
						state.vfs_selection = child.slot();
					}

					resp.on_hover_text("View");
				});
			}
		}
	});
}

// Helpers /////////////////////////////////////////////////////////////////////

fn side_menu_selector(dgui: &mut DevGui, ui: &mut egui::Ui, choices: &[(SideMenu, &'static str)]) {
	egui::ComboBox::new("viletech_dgui_selector", "Side Menu")
		.selected_text(format!("{}", dgui.side))
		.show_ui(ui, |ui| {
			for (choice, label) in choices.iter().copied() {
				ui.selectable_value(&mut dgui.side, choice, label);
			}
		});
}
