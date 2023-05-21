use std::sync::atomic::{self, AtomicUsize};

use bevy_egui::egui;

use crate::{VPath, VPathBuf};

use super::{file::FileKey, File, VirtualFs};

#[derive(Debug, Default)]
pub(super) struct DevGui {
	sel_file: AtomicUsize,
}

impl DevGui {
	fn select_file(&self, index: usize) {
		self.sel_file.store(index, atomic::Ordering::Relaxed);
	}
}

impl VirtualFs {
	pub(super) fn ui_impl(&self, ui: &mut egui::Ui) {
		ui.heading("Virtual File System");

		let sel_file = self.gui.sel_file.load(atomic::Ordering::Relaxed);

		if sel_file >= self.files.len() {
			self.gui.select_file(0);
		}

		egui::ScrollArea::vertical().show(ui, |ui| {
			let kvp = self.files.get_index(sel_file).unwrap();
			self.ui_nav(ui, kvp, sel_file);
			let (_, file) = kvp;

			match &file {
				File::Binary(bytes) => {
					ui.label("Binary");
					let mut unit = "B";
					let mut len = bytes.len() as f64;

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
				File::Text(string) => {
					ui.label("Text");
					ui.label(&format!("{} B", string.len()));
				}
				File::Empty => {
					ui.label("Empty");
				}
				File::Directory(dir) => {
					if dir.len() == 1 {
						ui.label("Directory: 1 child");
					} else {
						ui.label(&format!("Directory: {} children", dir.len()));
					}

					for path in dir {
						let label = egui::Label::new(path.to_string_lossy().as_ref())
							.sense(egui::Sense::click());

						let resp = ui.add(label);

						let resp = if resp.hovered() {
							resp.highlight()
						} else {
							resp
						};

						if resp.clicked() {
							let idx = self.files.get_index_of(path);
							self.gui.select_file(idx.unwrap());
						}

						resp.on_hover_text("View");
					}
				}
			}
		});
	}

	fn ui_nav(&self, ui: &mut egui::Ui, kvp: (&FileKey, &File), gui_sel: usize) {
		let (path, _) = kvp;

		ui.horizontal(|ui| {
			ui.add_enabled_ui(gui_sel != 0, |ui| {
				if ui
					.button("\u{2B06}")
					.on_hover_text("Go to Parent")
					.clicked()
				{
					let idx = self.files.get_index_of(path.parent().unwrap());
					self.gui.select_file(idx.unwrap());
				}
			});

			for (i, comp) in path.components().enumerate() {
				let label = egui::Label::new(comp.as_os_str().to_string_lossy().as_ref())
					.sense(egui::Sense::click());

				let resp = ui.add(label);

				let resp = if resp.hovered() {
					resp.highlight()
				} else {
					resp
				};

				if resp.clicked() {
					let p: VPathBuf = path.components().take(i + 1).collect();
					let idx = self.files.get_index_of::<VPath>(p.as_ref());
					self.gui.select_file(idx.unwrap());
				}

				resp.on_hover_text("Go to");

				if !matches!(comp, std::path::Component::RootDir) {
					ui.label("/");
				}
			}
		});
	}
}
