/*

Copyright (C) 2022 ***REMOVED***

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.

*/

use std::{
	collections::VecDeque,
	path::{Path, PathBuf},
};

use crate::utils::string::*;

#[derive(PartialEq, Eq)]
pub enum FrontendAction {
	None,
	Quit,
	Start,
}

pub enum LoadOrderEntryKind {
	Item {
		path: PathBuf,
		enabled: bool,
	},
	Group {
		name: String,
		children: VecDeque<LoadOrderEntry>,
	},
}

pub struct LoadOrderEntry {
	selected: bool,
	kind: LoadOrderEntryKind,
}

impl LoadOrderEntry {
	fn get_paths<'a>(&'a self, paths: &mut Vec<&'a Path>) {
		match &self.kind {
			LoadOrderEntryKind::Item { path, enabled } => {
				if *enabled {
					paths.push(path);
				}
			}
			LoadOrderEntryKind::Group { children, .. } => {
				for child in children {
					child.get_paths(paths);
				}
			}
		}
	}

	fn remove_selected(&mut self) {
		match &mut self.kind {
			LoadOrderEntryKind::Item { .. } => {}
			LoadOrderEntryKind::Group { children, .. } => {
				for child in children.iter_mut() {
					child.remove_selected();
				}

				children.retain(|child| !child.selected);
			}
		};
	}
}

struct LoadOrderPreset {
	entries: VecDeque<LoadOrderEntry>,
}

/// First thing shown to the user when they start the engine, assuming they
/// haven't passed in launch arguments which bypass it to the sim.
#[derive(Default)]
pub struct FrontendMenu {
	presets: Vec<LoadOrderPreset>,
	load_order: VecDeque<LoadOrderEntry>,
}

// Public interface.
impl FrontendMenu {
	#[must_use]
	pub fn ui(&mut self, ctx: &egui::Context) -> FrontendAction {
		let mut ret = FrontendAction::None;

		egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
			egui::menu::bar(ui, |ui| {
				if ui.button("Start").clicked() {
					ret = FrontendAction::Start;
				}

				ui.separator();

				if ui.button("Profiles").clicked() {}

				ui.separator();

				if ui.button("Exit").clicked() {
					ret = FrontendAction::Quit;
				}

				ui.separator();

				egui::widgets::global_dark_light_mode_buttons(ui);
			});
		});

		egui::Window::new("Load Order").show(ctx, |ui| {
			let mut selected = self.load_order.iter().filter(|loe| loe.selected).count();

			egui::menu::bar(ui, |ui| {
				if ui.button("\u{2B}\u{1F4C4}").clicked() {
					if let Some(files) = rfd::FileDialog::new()
						.set_directory(
							std::env::current_dir()
								.expect("Failed to get program's working directory."),
						)
						.pick_files()
					{
						for file in files {
							self.load_order.push_front(LoadOrderEntry {
								selected: false,
								kind: LoadOrderEntryKind::Item {
									path: file,
									enabled: true,
								},
							})
						}
					}
				}

				if ui.button("\u{2B}\u{1F4C1}").clicked() {
					if let Some(dirs) = rfd::FileDialog::new()
						.set_directory(
							std::env::current_dir()
								.expect("Failed to get program's working directory."),
						)
						.pick_folders()
					{
						for dir in dirs {
							self.load_order.push_front(LoadOrderEntry {
								selected: false,
								kind: LoadOrderEntryKind::Item {
									path: dir,
									enabled: true,
								},
							})
						}
					}
				}

				ui.add_enabled_ui(selected > 0, |ui| {
					if ui.button("\u{2796}").clicked() {
						self.remove_selected();
						selected = 0;
					}

					if ui.button("\u{2B06}").clicked() {
						// TODO: Shift all highlighted load order items up once
					}

					if ui.button("\u{2B07}").clicked() {
						// TODO: Shift all highlighted load order items down once
					}

					if ui.button("To Top").clicked() {}

					if ui.button("To Bottom").clicked() {}
				});
			});

			ui.horizontal(|ui| {
				// Load order list

				egui::ScrollArea::new([false, true]).show(ui, |ui| {
					ui.vertical(|ui| {
						egui::Grid::new("load_order")
							.num_columns(self.load_order.len())
							.striped(true)
							.show(ui, |ui| {
								self.ui_load_order(ui);
							});
					});
				});

				// Information panel on first selected item, if it's not a group

				if selected == 1 {
					let entry = self.load_order.iter().find(|loe| loe.selected).unwrap();

					match &entry.kind {
						LoadOrderEntryKind::Group { .. } => {}
						LoadOrderEntryKind::Item { path, .. } => {
							ui.separator();

							let fstem = path.file_stem();
							let mut fstem = fstem.unwrap_or_default().to_string_lossy().to_string();
							let vers_opt = version_from_string(&mut fstem);

							ui.label(&fstem);

							if let Some(vers) = vers_opt {
								ui.label(vers);
							}
						}
					};
				}
			});
		});

		ret
	}

	#[must_use]
	pub fn to_mount(&mut self) -> Vec<&Path> {
		let mut ret = Vec::<&Path>::default();

		for entry in &self.load_order {
			entry.get_paths(&mut ret);
		}

		ret
	}
}

// State-mutating helpers.
impl FrontendMenu {
	fn remove_selected(&mut self) {
		let mut i = 0;

		while i < self.load_order.len() {
			self.load_order[i].remove_selected();

			if self.load_order[i].selected {
				self.load_order.remove(i);
			} else {
				i += 1;
			}
		}
	}

	fn clear_selection(&mut self) {
		for entry in &mut self.load_order {
			entry.selected = false;
		}
	}
}

// UI drawing helpers.
impl FrontendMenu {
	fn ui_load_order(&mut self, ui: &mut egui::Ui) {
		let mut to_select: usize = self.load_order.len();

		for (i, loe) in self.load_order.iter_mut().enumerate() {
			let draggable = egui::Label::new("=").sense(egui::Sense::click());
			if ui.add(draggable).clicked() {
				to_select = i;
			}

			match &mut loe.kind {
				LoadOrderEntryKind::Group { name, .. } => {
					let mut enable_grp = false;

					ui.checkbox(&mut enable_grp, "");
					ui.text_edit_multiline(name);

					if enable_grp {}
				}
				LoadOrderEntryKind::Item { path, enabled } => {
					let fname = match path.file_name() {
						Some(f) => f,
						None => {
							continue;
						}
					};

					ui.checkbox(enabled, "");

					let fname = fname.to_string_lossy();
					let name_label = egui::Label::new(fname.as_ref()).sense(egui::Sense::click());

					if ui.add(name_label).clicked() {
						to_select = i;
					}
				}
			};

			ui.end_row();
		}

		if to_select != self.load_order.len() {
			if ui.input().modifiers.ctrl {
				self.load_order[to_select].selected = !self.load_order[to_select].selected;
			} else {
				self.clear_selection();
				self.load_order[to_select].selected = true;
			}
		}
	}
}
