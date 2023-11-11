use bevy_egui::egui;
use viletech::vfs::{self, FileSlot};

use super::Editor;

pub(super) fn ui(ed: &mut Editor, ui: &mut egui::Ui) {
	match ed.file_viewer.selected.len() {
		0 => {}
		1 => {
			let slot = ed.file_viewer.selected.iter().copied().next().unwrap();

			let vfs::Slot::File(islot) = slot else {
				return;
			};

			ui_inspect(ed, ui, islot);
		}
		n => {
			ui.centered_and_justified(|ui| {
				ui.label(&format!("{n} files selected"));
			});
		}
	}
}

fn ui_inspect(ed: &mut Editor, _: &mut egui::Ui, slot: FileSlot) {
	let _ = ed.file_viewer.content_id.get(&slot).unwrap();
}
