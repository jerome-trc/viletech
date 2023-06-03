use std::ops::Deref;

use parking_lot::RwLock;
use util::path::PathExt;

use crate::{file::Content, FileRef};

use super::{VPath, VPathBuf, VirtualFs};

#[derive(Debug, Default)]
pub(super) struct DevGui {
	sel_file: RwLock<VPathBuf>,
}

impl DevGui {
	fn select_file(&self, path: VPathBuf) {
		*self.sel_file.write() = path;
	}
}

impl VirtualFs {
	pub(super) fn ui_impl(&self, ui: &mut egui::Ui) {
		ui.heading("Virtual File System");

		let sel_file = self.gui.sel_file.read();

		if !self
			.files
			.contains_key(AsRef::<VPath>::as_ref(sel_file.as_path()))
		{
			self.gui.select_file(VPathBuf::from("/"));
		}

		egui::ScrollArea::vertical().show(ui, |ui| {
			let fref = self.get(sel_file.as_path()).unwrap();
			self.ui_nav(ui, fref);
			let file = fref.deref();

			match &file.content {
				Content::Text(string) => {
					ui.label("Binary");
					Self::ui_label_file_len(ui, string.len());
				}
				Content::Binary(bytes) => {
					ui.label("Binary");
					Self::ui_label_file_len(ui, bytes.len());
				}
				Content::Empty => {
					ui.label("Empty");
				}
				Content::Directory { children, kind } => {
					if children.len() != 1 {
						ui.label(&format!("Directory: {} children ({kind})", children.len()));
					} else {
						ui.label(&format!("Directory: 1 child ({kind})"));
					}

					for path in children.iter() {
						let label = egui::Label::new(path.to_string_lossy().as_ref())
							.sense(egui::Sense::click());

						let resp = ui.add(label);

						let resp = if resp.hovered() {
							resp.highlight()
						} else {
							resp
						};

						if resp.clicked() {
							self.gui.select_file(path.to_path_buf());
						}

						resp.on_hover_text("View");
					}
				}
			}
		});
	}

	fn ui_nav(&self, ui: &mut egui::Ui, fref: FileRef) {
		ui.horizontal(|ui| {
			ui.add_enabled_ui(!fref.path().is_root(), |ui| {
				if ui
					.button("\u{2B06}")
					.on_hover_text("Go to Parent")
					.clicked()
				{
					self.gui
						.select_file(fref.path().parent().unwrap().to_path_buf());
				}
			});

			for (i, comp) in fref.path().components().enumerate() {
				let label = egui::Label::new(comp.as_os_str().to_string_lossy().as_ref())
					.sense(egui::Sense::click());

				let resp = ui.add(label);

				let resp = if resp.hovered() {
					resp.highlight()
				} else {
					resp
				};

				if resp.clicked() {
					self.gui
						.select_file(fref.path().components().take(i + 1).collect());
				}

				resp.on_hover_text("Go to");

				if !matches!(comp, std::path::Component::RootDir) {
					ui.label("/");
				}
			}
		});
	}

	fn ui_label_file_len(ui: &mut egui::Ui, len: usize) {
		let mut len = len as f64;
		let mut unit = "B";

		if len > 1024.0 {
			len /= 1024.0;
			unit = "KB";
		}

		if len > 1024.0 {
			len /= 1024.0;
			unit = "MB";
		}

		if len > 1024.0 {
			len /= 1024.0;
			unit = "GB";
		}

		ui.label(&format!("{len:.2} {unit}"));
	}
}
