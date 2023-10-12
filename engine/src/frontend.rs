//! A menu for changing settings and choosing files to load.

use std::{
	borrow::Cow,
	collections::VecDeque,
	path::{Path, PathBuf},
};

use bevy::prelude::Resource;
use bevy_egui::egui;
use serde::{Deserialize, Serialize};

/// A menu for changing settings and choosing files to load.
///
/// This is the first thing a client end user sees assuming they have not passed
/// in launch arguments which bypass it to start loading a game.
#[derive(Debug, Resource)]
pub struct FrontendMenu {
	/// *Always* contains at least one element.
	presets: VecDeque<LoadOrderPreset>,
	cur_preset: usize,
	full_paths: bool,
	dev_mode: bool,
}

/// Public interface.
impl FrontendMenu {
	#[must_use]
	pub fn new(presets: Option<(VecDeque<LoadOrderPreset>, usize)>, dev_mode: bool) -> Self {
		let (presets, cur_preset) =
			presets.unwrap_or_else(|| (VecDeque::from([LoadOrderPreset::new()]), 0));

		let ret = Self {
			presets,
			cur_preset,
			full_paths: false,
			dev_mode,
		};

		assert!(ret.cur_preset < ret.presets.len());

		ret
	}

	#[must_use]
	pub fn ui(&mut self, ctx: &egui::Context) -> Outcome {
		let mut ret = Outcome::None;

		egui::TopBottomPanel::top("viletech_frontend_menubar").show(ctx, |ui| {
			egui::menu::bar(ui, |ui| {
				ret = self.ui_menu_bar(ui);
			});
		});

		egui::Window::new("Frontend")
			.id("viletech_frontend".into())
			.min_width(480.0)
			.show(ctx, |ui| {
				let mut sel_count = self.load_order().iter().filter(|loe| loe.selected).count();

				egui::SidePanel::right("viletech_frontend_presets")
					.min_width(120.0)
					.show_inside(ui, |ui| {
						ui.heading("Presets");

						egui::ScrollArea::new([false, true]).show(ui, |ui| {
							ui.vertical(|ui| {
								egui::Grid::new("viletech_frontend_presets_grid")
									.num_columns(self.load_order().len())
									.striped(true)
									.show(ui, |ui| {
										self.ui_presets(ui);
									});
							});
						});
					});

				egui::CentralPanel::default().show_inside(ui, |ui| {
					ui.heading("Load Order");

					egui::menu::bar(ui, |ui| {
						if ui.button("\u{2B}\u{1F4C4}").clicked() {
							if let Some(files) = rfd::FileDialog::new()
								.set_directory(
									std::env::current_dir()
										.expect("failed to get program's working directory"),
								)
								.pick_files()
							{
								for file in files {
									self.load_order_mut().push_front(LoadOrderEntry {
										selected: false,
										kind: LoadOrderEntryKind::Item {
											path: file,
											enabled: true,
											exists: true,
										},
									})
								}
							}
						}

						if ui.button("\u{2B}\u{1F4C1}").clicked() {
							if let Some(dirs) = rfd::FileDialog::new()
								.set_directory(
									std::env::current_dir()
										.expect("failed to get program's working directory"),
								)
								.pick_folders()
							{
								for dir in dirs {
									self.load_order_mut().push_front(LoadOrderEntry {
										selected: false,
										kind: LoadOrderEntryKind::Item {
											path: dir,
											enabled: true,
											exists: true,
										},
									})
								}
							}
						}

						ui.add_enabled_ui(sel_count > 0, |ui| {
							if ui.button("\u{2796}").clicked() {
								self.remove_selected();
								sel_count = 0;
							}

							if ui.button("\u{2B06}").clicked() {
								// TODO: shift all highlighted load order items up once.
							}

							if ui.button("\u{2B07}").clicked() {
								// TODO: shift all highlighted load order items down once.
							}

							// TODO: to-top and to-bottom buttons.
						});

						ui.separator();

						ui.checkbox(&mut self.full_paths, "Show Full File Paths");
					});

					egui::ScrollArea::new([false, true]).show(ui, |ui| {
						ui.vertical(|ui| {
							egui::Grid::new("viletech_frontend_loadord_grid")
								.num_columns(self.load_order().len())
								.striped(true)
								.show(ui, |ui| {
									self.ui_load_order(ui);
								});
						});
					});
				});
			});

		ret
	}

	#[must_use]
	pub fn to_mount(&mut self) -> Vec<&Path> {
		let mut ret = Vec::<&Path>::default();

		for entry in self.load_order().iter() {
			entry.get_paths(&mut ret);
		}

		ret
	}

	pub fn validate(&mut self) {
		for entry in self.load_order_mut().iter_mut() {
			if let LoadOrderEntryKind::Item { path, exists, .. } = &mut entry.kind {
				*exists = path.exists();
			}
		}
	}

	#[must_use]
	fn load_order(&self) -> &LoadOrderPreset {
		&self.presets[self.cur_preset]
	}

	#[must_use]
	pub fn dev_mode(&self) -> bool {
		self.dev_mode
	}

	/// Returns the current array of load order presets as well as the
	/// index of the currently-selected one, to be serialized.
	#[must_use]
	pub fn consume(&mut self) -> (VecDeque<LoadOrderPreset>, usize) {
		let presets = std::mem::take(&mut self.presets);
		(presets, self.cur_preset)
	}

	// Internal UI drawing helpers /////////////////////////////////////////////

	fn ui_menu_bar(&mut self, ui: &mut egui::Ui) -> Outcome {
		let mut ret = Outcome::None;

		let any_nonexistent = self.any_nonexistent_items();
		let load_order_empty = self.load_order().is_empty();

		let btn_start = ui.add_enabled_ui(!any_nonexistent && !load_order_empty, |ui| {
			let btn_start = ui.button("Start Game");

			if btn_start.clicked() {
				ret = Outcome::StartGame;
			}

			btn_start
		});

		ui.separator();

		let btn_ed = ui.add_enabled_ui(!any_nonexistent && !load_order_empty, |ui| {
			let btn = ui.button("Editor");

			if btn.clicked() {
				ret = Outcome::StartEditor;
			}

			btn
		});

		if load_order_empty {
			const T: &str = "A game can not be started without an IWAD.";
			btn_start.response.on_hover_text(T);
			btn_ed.response.on_hover_text(T);
		} else if any_nonexistent {
			const T: &str = "One or more load order items have been deleted or moved.";
			btn_start.response.on_hover_text(T);
			btn_ed.response.on_hover_text(T);
		}

		ui.separator();

		// TODO: user information management (e.g. preferences) goes here.

		// TODO: tooltip.
		ui.checkbox(&mut self.dev_mode, "Developer Mode");

		ui.separator();

		if ui.button("Exit").clicked() {
			ret = Outcome::Exit;
		}

		ui.separator();

		egui::widgets::global_dark_light_mode_buttons(ui);

		ret
	}

	fn ui_load_order(&mut self, ui: &mut egui::Ui) {
		let show_full_paths = self.full_paths;
		let load_order = self.load_order_mut();
		let mut to_select = load_order.len();

		for (i, loe) in load_order.iter_mut().enumerate() {
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
				LoadOrderEntryKind::Item { path, enabled, .. } => {
					let name = if show_full_paths {
						path.to_string_lossy()
					} else if let Some(n) = path.file_name() {
						n.to_string_lossy()
					} else {
						Cow::Borrowed("<unnamed file>")
					};

					ui.checkbox(enabled, "");

					let name_label = egui::Label::new(name.as_ref()).sense(egui::Sense::click());

					if ui.add(name_label).clicked() {
						to_select = i;
					}
				}
			};

			ui.end_row();
		}

		if to_select != load_order.len() {
			if ui.input(|inps| inps.modifiers.ctrl) {
				load_order[to_select].selected = !load_order[to_select].selected;
			} else {
				self.clear_selection();
				self.load_order_mut()[to_select].selected = true;
			}
		}
	}

	fn ui_presets(&mut self, ui: &mut egui::Ui) {
		for (_, preset) in self.presets.iter_mut().enumerate() {
			ui.label(&preset.name);
			ui.end_row();
		}
	}

	// Internal non-UI helpers /////////////////////////////////////////////////

	fn remove_selected(&mut self) {
		let mut i = 0;
		let load_order = self.load_order_mut();

		while i < load_order.len() {
			load_order[i].remove_selected();

			if load_order[i].selected {
				load_order.remove(i);
			} else {
				i += 1;
			}
		}
	}

	fn clear_selection(&mut self) {
		for entry in &mut self.load_order_mut().iter_mut() {
			entry.selected = false;
		}
	}

	#[must_use]
	fn load_order_mut(&mut self) -> &mut LoadOrderPreset {
		&mut self.presets[self.cur_preset]
	}

	#[must_use]
	fn any_nonexistent_items(&self) -> bool {
		self.load_order().iter().any(|entry| {
			if let LoadOrderEntryKind::Item { exists, .. } = entry.kind {
				return !exists;
			}

			false
		})
	}
}

/// What the caller should do after having drawn a frontend frame.
/// See [`FrontendMenu::ui`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
	None,
	Exit,
	StartGame,
	StartEditor,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LoadOrderPreset {
	name: String,
	entries: VecDeque<LoadOrderEntry>,
}

impl LoadOrderPreset {
	#[must_use]
	pub fn new() -> Self {
		LoadOrderPreset {
			name: "Default".to_string(), // TODO: Localize this.
			entries: VecDeque::default(),
		}
	}
}

impl std::ops::Deref for LoadOrderPreset {
	type Target = VecDeque<LoadOrderEntry>;

	fn deref(&self) -> &Self::Target {
		&self.entries
	}
}

impl std::ops::DerefMut for LoadOrderPreset {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.entries
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadOrderEntry {
	selected: bool,
	#[serde(flatten)]
	kind: LoadOrderEntryKind,
}

impl LoadOrderEntry {
	fn get_paths<'a>(&'a self, paths: &mut Vec<&'a Path>) {
		match &self.kind {
			LoadOrderEntryKind::Item { path, enabled, .. } => {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadOrderEntryKind {
	Item {
		path: PathBuf,
		enabled: bool,
		/// Gets set by [`FrontendMenu::validate`].
		#[serde(skip)]
		exists: bool,
	},
	Group {
		name: String,
		children: VecDeque<LoadOrderEntry>,
	},
}
