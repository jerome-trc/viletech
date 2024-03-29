use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_egui::egui;
use egui_extras::TableBody;
use viletech::{
	regex::{self, Regex, RegexBuilder},
	rustc_hash::{FxHashMap, FxHashSet},
	util::string::subdivide_file_len,
	vfs::{self, FileRef, FileSlot, FolderKind, FolderRef, FolderSlot},
	VirtualFs,
};

use crate::editor;

use super::{
	contentid::{ContentId, WadMarkers},
	Editor,
};

#[derive(Debug)]
pub(crate) struct FileViewer {
	pub(super) filter_buf: String,
	pub(super) filter_regex: bool,
	pub(super) filter: Result<Regex, regex::Error>,

	pub(super) selected: FxHashSet<vfs::Slot>,
	pub(super) folded: FxHashSet<FolderSlot>,
	pub(super) content_id: FxHashMap<FileSlot, ContentId>,
}

impl FileViewer {
	#[must_use]
	pub(crate) fn new(vfs: &VirtualFs) -> Self {
		let mut ret = Self {
			filter_buf: String::new(),
			filter_regex: false,
			filter: Regex::new(""),

			selected: FxHashSet::default(),
			folded: FxHashSet::default(),
			content_id: FxHashMap::default(),
		};

		debug_assert!(ret.filter.is_ok());

		for folder in vfs.folders() {
			if folder.parent().is_some_and(|s| s != vfs.root().slot()) {
				ret.folded.insert(folder.slot());
			}
		}

		ret
	}

	fn update_filter(&mut self) {
		if !self.filter_regex {
			let pattern = regex::escape(&self.filter_buf);
			self.filter = RegexBuilder::new(&pattern).case_insensitive(true).build();
			debug_assert!(self.filter.is_ok());
		} else {
			self.filter = Regex::new(&self.filter_buf);
		}
	}
}

#[derive(SystemParam)]
pub(crate) struct SysParam<'w> {
	pub(crate) vfs: ResMut<'w, VirtualFs>,
	pub(crate) ewriter: EventWriter<'w, editor::Event>,
}

pub(super) fn ui(ed: &mut Editor, ui: &mut egui::Ui, mut param: SysParam) {
	ui.horizontal(|ui| {
		ui.label("Filter:");

		let resp = ui.text_edit_singleline(&mut ed.file_viewer.filter_buf);

		if resp.changed() {
			ed.file_viewer.update_filter();
		}

		let clear_btn = egui::Button::new("\u{2716}");

		if ui
			.add_enabled(!ed.file_viewer.filter_buf.is_empty(), clear_btn)
			.on_hover_text("Clear")
			.clicked()
		{
			ed.file_viewer.filter_buf.clear();
			ed.file_viewer.update_filter();
		}

		if ui.button(".*").on_hover_text("Regex").clicked() {
			ed.file_viewer.filter_regex = !ed.file_viewer.filter_regex;
			ed.file_viewer.update_filter();
		}
	});

	ui.separator();

	let row_height = ui.text_style_height(&egui::TextStyle::Body) * 1.2;

	let table = egui_extras::TableBuilder::new(ui)
		.striped(true)
		.resizable(true)
		.cell_layout(egui::Layout::left_to_right(egui::Align::Center))
		.column(egui_extras::Column::auto())
		.column(egui_extras::Column::auto())
		.column(egui_extras::Column::remainder())
		.min_scrolled_height(0.0);

	table
		.header(20.0, |mut header| {
			let _ = header.col(|ui| {
				ui.strong("Name");
			});
			let _ = header.col(|ui| {
				ui.strong("Type");
			});
			let _ = header.col(|ui| {
				ui.strong("Size");
			});
		})
		.body(|mut body| {
			// TODO: row culling.
			ui_folder(
				ed,
				&mut param.ewriter,
				param.vfs.root(),
				&mut body,
				0,
				row_height,
			);
		});
}

fn ui_folder(
	ed: &mut Editor,
	ewriter: &mut EventWriter<editor::Event>,
	vfolder: FolderRef,
	body: &mut TableBody,
	depth: u32,
	row_height: f32,
) {
	let folded = ed.file_viewer.folded.contains(&vfolder.slot());

	body.row(row_height, |mut row| {
		let _ = row.col(|ui| {
			let btn_icon = if folded { "\u{23F5}" } else { "\u{23F7}" };
			let btn = egui::Button::new(btn_icon).frame(false);

			ui.add_space((depth as f32) * 8.0);

			if ui.add(btn).clicked() {
				if folded {
					let was_present = ed.file_viewer.folded.remove(&vfolder.slot());
					debug_assert!(was_present);
				} else {
					let was_absent = ed.file_viewer.folded.insert(vfolder.slot());
					debug_assert!(was_absent);
				}
			}

			let mut label = String::new();
			label.push_str(vfolder.name().as_str());
			ui.label(&label);
		});

		let _ = row.col(|ui| {
			let label = match vfolder.kind() {
				FolderKind::Directory => "directory",
				FolderKind::Wad => "WAD archive",
				FolderKind::ZipDir => "zip directory",
				FolderKind::Zip => "zip archive",
				FolderKind::Root => "",
			};

			ui.label(label);
		});

		let _ = row.col(|_| {
			// Folders have no bytes.
		});
	});

	if !folded {
		for subfolder in vfolder.subfolders() {
			ui_folder(ed, ewriter, subfolder, body, depth + 1, row_height);
		}

		let mut markers = WadMarkers::None;

		for file in vfolder.files() {
			if file.name().eq_ignore_ascii_case("F_START")
				|| file.name().eq_ignore_ascii_case("FF_START")
			{
				markers = WadMarkers::Flats;
			} else if file.name().eq_ignore_ascii_case("F_END")
				|| file.name().eq_ignore_ascii_case("FF_END")
			{
				markers = WadMarkers::None;
			} // TODO: expand on this system.

			ui_file(ed, ewriter, file, body, depth + 1, row_height, markers);
		}
	}
}

fn ui_file(
	ed: &mut Editor,
	ewriter: &mut EventWriter<editor::Event>,
	vfile: FileRef,
	body: &mut TableBody,
	depth: u32,
	row_height: f32,
	markers: WadMarkers,
) {
	if let Ok(rgx) = ed.file_viewer.filter.as_ref() {
		if !rgx.is_match(vfile.name().as_str()) {
			return;
		}
	}

	let slot = vfs::Slot::File(vfile.slot());
	let mut guard = vfile.lock();
	let bytes = guard
		.read()
		.expect("failed to read from VFS in-memory file");

	let content_id = *ed
		.file_viewer
		.content_id
		.entry(vfile.slot())
		.or_insert(ContentId::deduce(&vfile, &bytes, markers));

	let mut row_rect = egui::Rect::NOTHING;

	body.row(row_height, |mut row| {
		let mut ctrl_held = false;

		let (_, resp0) = row.col(|ui| {
			let mut label = String::new();
			label.push_str(vfile.name().as_str());
			ui.add_space((depth as f32) * 16.0);
			ui.label(&label);
			ctrl_held = ui.input(|inps| inps.modifiers.ctrl);
		});

		let (_, resp1) = row.col(|ui| {
			ui.label(&format!("{content_id}"));
		});

		let (_, resp2) = row.col(|ui| {
			ui.label(&subdivide_file_len(vfile.size()));
		});

		let resp = resp0
			.interact(egui::Sense::click())
			.union(resp1.interact(egui::Sense::click()))
			.union(resp2.interact(egui::Sense::click()));

		if resp.clicked() {
			if ctrl_held {
				if ed.file_viewer.selected.contains(&slot) {
					ed.file_viewer.selected.remove(&slot);
				} else {
					ed.file_viewer.selected.insert(slot);
				}
			} else {
				ed.file_viewer.selected.clear();
				ed.file_viewer.selected.insert(slot);

				if !ed.currently_inspecting(vfile.slot()) {
					ewriter.send(editor::Event::Inspect {
						file: vfile.slot(),
						transient: true,
					});
				}
			}
		} else if resp.double_clicked() {
			ed.file_viewer.selected.clear();
			ed.file_viewer.selected.insert(slot);

			if !ed.currently_inspecting(vfile.slot()) {
				ewriter.send(editor::Event::Inspect {
					file: vfile.slot(),
					transient: true,
				});
			}
		} else {
			resp.context_menu(|ui| {
				context_menu(ed, ewriter, ui, &vfile, content_id);
			});
		}

		row_rect.set_top(resp0.rect.top());
		row_rect.set_bottom(resp0.rect.bottom());
		row_rect.set_left(resp0.rect.left());
		row_rect.set_right(resp2.rect.right());
	});

	if ed.file_viewer.selected.contains(&slot) {
		body.ui_mut().painter().rect_filled(
			row_rect,
			egui::Rounding::ZERO,
			egui::Color32::from_rgba_unmultiplied(127, 224, 255, 7),
		);
	}
}

fn context_menu(
	ed: &mut Editor,
	ewriter: &mut EventWriter<editor::Event>,
	ui: &mut egui::Ui,
	vfile: &FileRef,
	content_id: ContentId,
) {
	const LEVELEDIT_BTN_TXT: &str = "\u{1F5FA} Edit";
	const INSPECT_BTN_TXT: &str = "\u{1F50E} Inspect";

	match content_id {
		ContentId::MapMarker => {
			if ui.button(LEVELEDIT_BTN_TXT).clicked() {
				ewriter.send(editor::Event::EditLevel {
					marker: vfile.slot(),
				});
			}
		}
		ContentId::Picture | ContentId::Flat => {
			if ui.button(INSPECT_BTN_TXT).clicked() {
				ed.file_viewer.selected.clear();

				ed.file_viewer
					.selected
					.insert(vfs::Slot::File(vfile.slot()));

				ewriter.send(editor::Event::Inspect {
					file: vfile.slot(),
					transient: false,
				});
			}
		}
		_ => {}
	}
}
