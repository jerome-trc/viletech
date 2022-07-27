use std::{collections::VecDeque, path::PathBuf};

use crate::{file_browser::FileBrowser, utils::version_from_filestem};

#[derive(PartialEq)]
pub enum FrontendAction {
	None,
	Quit,
	Start,
}

/// First thing shown to the user when they start the engine, assuming they
/// haven't passed in launch arguments which bypass it to the sim.
#[derive(Default)]
pub struct FrontendMenu {
	/// `::1` indicates if the item has been disabled, and won't be loaded.
	/// `::2` indicates if the item has been highlighted by clicking on it.
	load_order: VecDeque<(PathBuf, bool, bool)>,
	file_browser: FileBrowser,
}

impl FrontendMenu {
	pub fn ui(&mut self, ctx: &egui::Context) -> FrontendAction {
		let mut ret = FrontendAction::None;

		egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
			egui::menu::bar(ui, |ui| {
				if ui.button("Start").clicked() {
					ret = FrontendAction::Start;
				}

				ui.separator();

				if ui.button("Exit").clicked() {
					ret = FrontendAction::Quit;
				}
			});
		});

		egui::Window::new("Load Order").show(ctx, |ui| {
			let highlighted = self.load_order.iter().filter(|gdo| gdo.2).count();

			egui::menu::bar(ui, |ui| {
				if ui.button("\u{2B}").clicked() {
					self.file_browser.toggle();
				}

				ui.add_enabled_ui(highlighted > 1, |ui| {
					if ui.button("\u{2B06}").clicked() {
						// TODO: Shift all highlighted load order items up once
						for i in 1..self.load_order.len() {
							if !self.load_order[i].2 {
								continue;
							}

							self.load_order.swap(i, i - 1);
						}
					}

					if ui.button("\u{2B07}").clicked() {
						// TODO: Shift all highlighted load order items down once
					}
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
								for gdo in &mut self.load_order {
									let fname = match gdo.0.file_name() {
										Some(f) => f,
										None => {
											continue;
										}
									};
	
									let fname = fname.to_string_lossy();
									ui.checkbox(&mut gdo.1, fname.as_ref());
									ui.end_row();
								}
							});
					});
				});

				// Information panel on highlighted item

				if highlighted == 1 {
					ui.separator();

					let info_item = self.load_order.iter().find(|gdo| gdo.2).unwrap();
					let fstem = info_item.0.file_stem();
					let mut fstem = fstem.unwrap_or_default().to_string_lossy().to_string();
					let vers_opt = version_from_filestem(&mut fstem);

					ui.label(&fstem);

					if let Some(vers) = vers_opt {
						ui.label(vers);
					}
				}
			});
		});

		if self.file_browser.ui(ctx) {
			for pb in self.file_browser.drain() {
				self.load_order.push_front((pb, true, false));
			}

			self.file_browser.toggle();
		}

		ret
	}
}
