//! File browser window like those common to GUI-based OS, implemented in egui.

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

use std::{fs, path::PathBuf};

use crate::utils::path::*;

pub struct FileBrowser {
	open: bool,
	show_hidden: bool,
	current_path: PathBuf,
	selected: Vec<PathBuf>,
}

impl Default for FileBrowser {
	#[must_use]
	fn default() -> Self {
		FileBrowser {
			open: false,
			show_hidden: Default::default(),
			current_path: exe_dir(),
			selected: Default::default(),
		}
	}
}

impl FileBrowser {
	#[must_use]
	pub fn new(mut starting_path: PathBuf) -> Self {
		if starting_path.is_empty() {
			starting_path = exe_dir();
		}

		FileBrowser {
			open: false,
			show_hidden: Default::default(),
			current_path: starting_path,
			selected: Default::default(),
		}
	}

	/// Returns true if the caller should call `drain()` and then `toggle()`.
	#[must_use]
	pub fn ui(&mut self, ctx: &egui::Context) -> bool {
		let mut open = self.open;
		let mut done = false;
		let modifiers = ctx.input().modifiers;

		egui::Window::new("File Browser")
			.open(&mut open)
			.show(ctx, |ui| {
				done = self.ui_impl(ui, modifiers);
			});

		self.open = open;
		done
	}

	#[must_use]
	fn ui_impl(&mut self, ui: &mut egui::Ui, modifiers: egui::Modifiers) -> bool {
		let mut ret = false;

		ui.horizontal(|ui| {
			ui.add_enabled_ui(false, |ui| {
				if ui.button("\u{2B05}").clicked() {
					// TODO: Back button
				}
				if ui.button("\u{27A1}").clicked() {
					// TODO: Forward button
				}
			});

			let parent = self.current_path.parent();

			let resp = ui.add_enabled_ui(parent.is_some(), |ui| {
				let up = ui.button("\u{2B06}");
				up.clicked()
			});

			if let Some(p) = parent {
				if resp.inner {
					self.current_path = p.to_path_buf();
				}
			}
		});

		let entries = match fs::read_dir(&self.current_path) {
			Ok(e) => e,
			Err(_) => {
				return ret;
			}
		};

		let entries = entries.filter_map(|res| match res {
			Ok(r) => Some(r),
			Err(_) => None,
		});

		let num_cols = fs::read_dir(&self.current_path).unwrap().count();

		egui::ScrollArea::new([false, true]).show(ui, |ui| {
			egui::Grid::new("file_list")
				.num_columns(num_cols)
				.striped(true)
				.show(ui, |ui| {
					for entry in entries {
						let ftype = match entry.file_type() {
							Ok(ft) => ft,
							Err(_) => {
								continue;
							}
						};

						if ftype.is_symlink() {
							continue;
						}

						let p = entry.path();
						let is_selected = self.selected.contains(&p);
						let fname = entry.file_name();
						let fnamestr = fname.to_string_lossy();

						let row = move |ui: &mut egui::Ui| {
							let text = format!(
								"{}{}{}",
								if is_selected { "\u{2705} " } else { " " },
								if ftype.is_dir() {
									"\u{1F5C1} "
								} else {
									"\u{1F5CB} "
								},
								fnamestr
							);

							ui.add(egui::Label::new(text).sense(egui::Sense::click()))
						};

						let resp = ui.add(row);

						if resp.double_clicked() {
							if ftype.is_dir() {
								self.current_path = p;
							} else {
								self.selected.clear();
								self.selected.push(p);
								ret = true;
								return;
							}
						} else if resp.clicked() {
							if modifiers.ctrl {
								if is_selected {
									self.selected.retain(|f| *f != p);
								} else {
									self.selected.push(p);
								}
							} else {
								self.selected.clear();

								if !is_selected && !ftype.is_dir() {
									self.selected.push(p);
								}
							}

							// TODO: shift support
						}

						ui.end_row();
					}
				});
		});

		ui.separator();

		ui.horizontal(|ui| {
			if ui.button("Confirm").clicked() {
				ret = true;
				return;
			}

			if ui.button("Cancel").clicked() {
				self.selected.clear();
				ret = true;
			}
		});

		ret
	}

	#[must_use]
	pub fn is_open(&self) -> bool {
		self.open
	}

	pub fn toggle(&mut self) {
		self.selected.clear();
		self.open = !self.open;
	}

	#[must_use]
	pub fn drain(&mut self) -> std::vec::Drain<PathBuf> {
		self.selected.drain(..)
	}
}
