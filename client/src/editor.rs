//! Functions run when entering, updating, and leaving [`AppState::Editor`].

mod contentid;
mod fileview;
mod inspector;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use viletech::VirtualFs;

use crate::AppState;

use self::fileview::FileViewer;

#[derive(Resource, Debug)]
pub(crate) struct Editor {
	panel_l: Option<Dialog>,
	panel_r: Option<Dialog>,
	panel_m: Dialog,
	panel_b: Option<Dialog>,

	file_viewer: FileViewer,
}

/// What content is occupying a panel?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Dialog {
	DecoViz,
	Files,
	Inspector,
	Messages,
}

impl std::fmt::Display for Dialog {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DecoViz => write!(f, "\u{1F500} DecoViz"),
			Self::Files => write!(f, "\u{1F5C0} Files"),
			Self::Inspector => write!(f, "\u{1F50E} Inspector"),
			Self::Messages => write!(f, "\u{1F4E7} Message Log"),
		}
	}
}

pub(crate) fn update(
	mut next_state: ResMut<NextState<AppState>>,
	mut ed: ResMut<Editor>,
	mut egui: EguiContexts,
	mut vfs: ResMut<VirtualFs>,
) {
	let guictx = egui.ctx_mut();

	egui::TopBottomPanel::top("viletech_ed_panel_t").show(guictx, |ui| {
		ui.horizontal_wrapped(|ui| {
			ui.visuals_mut().button_frame = false;

			if ui
				.button("Back")
				.on_hover_text("Return to the frontend.")
				.clicked()
			{
				next_state.set(AppState::Frontend);
				return;
			}

			ui.separator();
			egui::widgets::global_dark_light_mode_switch(ui);
		});
	});

	if let Some(panel_l) = ed.panel_l {
		egui::SidePanel::left("viletech_ed_panel_l").show(guictx, |ui| {
			egui::menu::bar(ui, |ui| {
				ed.panel_l = Some(dialog_combo(ui, panel_l));
			});

			match panel_l {
				Dialog::DecoViz => {}
				Dialog::Files => fileview::ui(&mut ed, ui, &mut vfs),
				Dialog::Inspector => inspector::ui(&mut ed, ui),
				Dialog::Messages => {}
			}
		});
	}

	if let Some(panel_r) = ed.panel_r {
		egui::SidePanel::right("viletech_ed_panel_r").show(guictx, |ui| {
			egui::menu::bar(ui, |ui| {
				ed.panel_r = Some(dialog_combo(ui, panel_r));
			});

			match panel_r {
				Dialog::DecoViz => {}
				Dialog::Files => fileview::ui(&mut ed, ui, &mut vfs),
				Dialog::Inspector => inspector::ui(&mut ed, ui),
				Dialog::Messages => {}
			}
		});
	}

	if let Some(panel_b) = ed.panel_b {
		egui::TopBottomPanel::bottom("viletech_ed_panel_b").show(guictx, |ui| {
			egui::menu::bar(ui, |ui| {
				ed.panel_b = Some(dialog_combo(ui, panel_b));
			});

			match panel_b {
				Dialog::DecoViz => {}
				Dialog::Files => fileview::ui(&mut ed, ui, &mut vfs),
				Dialog::Inspector => inspector::ui(&mut ed, ui),
				Dialog::Messages => {}
			}
		});
	}

	egui::CentralPanel::default().show(guictx, |ui| {
		egui::menu::bar(ui, |ui| {
			ed.panel_m = dialog_combo(ui, ed.panel_m);
		});

		match ed.panel_m {
			Dialog::DecoViz => {}
			Dialog::Files => fileview::ui(&mut ed, ui, &mut vfs),
			Dialog::Inspector => inspector::ui(&mut ed, ui),
			Dialog::Messages => {}
		}
	});
}

#[must_use]
fn dialog_combo(ui: &mut egui::Ui, mut dialog: Dialog) -> Dialog {
	egui::ComboBox::new("viletech_ed_dialog_combo", "")
		.selected_text(format!("{}", dialog))
		.show_ui(ui, |ui| {
			ui.selectable_value(&mut dialog, Dialog::Files, "\u{1F5C0} Files");
			ui.selectable_value(&mut dialog, Dialog::Inspector, "\u{1F50E} Inspector");
			ui.selectable_value(&mut dialog, Dialog::Messages, "\u{1F4E7} Message Log");
			ui.selectable_value(&mut dialog, Dialog::DecoViz, "\u{1F500} DecoViz");
		});

	dialog
}

pub(crate) fn on_enter(mut cmds: Commands, vfs: Res<VirtualFs>) {
	cmds.insert_resource(Editor {
		panel_l: Some(Dialog::Files),
		panel_r: None,
		panel_m: Dialog::Inspector,
		panel_b: Some(Dialog::Messages),

		file_viewer: FileViewer::new(&vfs),
	});
}

pub(crate) fn on_exit(mut cmds: Commands) {
	cmds.remove_resource::<Editor>();
}
