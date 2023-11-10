//! Developer GUI state and functions.

use bevy_egui::egui::{self, TextStyle};
use regex::Regex;

use super::{dobj::datum_type_name, Catalog};

/// State storage for the catalog's developer GUI.
#[derive(Debug)]
pub(super) struct DevGui {
	search_buf: String,
	search: Regex,
}

impl DevGui {
	fn update_search_regex(&mut self) {
		let mut esc = regex::escape(&self.search_buf);
		esc.insert_str(0, "(?i)"); // Case insensitivity
		self.search = Regex::new(&esc).unwrap();
	}
}

impl Default for DevGui {
	fn default() -> Self {
		Self {
			search_buf: String::new(),
			search: Regex::new("").unwrap(),
		}
	}
}

impl Catalog {
	pub(super) fn ui_impl(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
		ui.heading("Game Data");

		ui.horizontal(|ui| {
			ui.label("Search");

			if ui.text_edit_singleline(&mut self.gui.search_buf).changed() {
				self.gui.update_search_regex();
			}

			if ui.button("Clear").clicked() {
				self.gui.search_buf.clear();
				self.gui.update_search_regex();
			}
		});

		egui::ScrollArea::vertical()
			.auto_shrink([false; 2])
			.show_rows(
				ui,
				ui.text_style_height(&TextStyle::Body),
				self.dobjs.len(),
				|ui, row_range| {
					for (_, store) in self.dobjs.iter().skip(row_range.start) {
						let id = store.id();

						if !self.gui.search.is_match(id) {
							continue;
						}

						let resp = ui.label(id);

						let resp = if resp.hovered() {
							resp.highlight()
						} else {
							resp
						};

						resp.on_hover_ui_at_pointer(|ui| {
							egui::Area::new("viletech_datum_tt").show(ctx, |_| {
								let type_name = datum_type_name(store.datum_typeid());
								ui.label(type_name);
							});
						});
					}
				},
			);
	}
}
