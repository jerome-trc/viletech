use bevy_egui::egui;
use egui_extras::TableBody;
use regex::{Regex, RegexBuilder};
use rustc_hash::{FxHashMap, FxHashSet};
use viletech::{
	vfs::{self, FileRef, FileSlot, FolderKind, FolderRef, FolderSlot},
	VirtualFs,
};

use super::{
	contentid::{ContentId, WadMarkers},
	Editor,
};

#[derive(Debug)]
pub(super) struct FileViewer {
	pub(super) filter_buf: String,
	pub(super) filter_regex: bool,
	pub(super) filter: Result<Regex, regex::Error>,

	pub(super) selected: FxHashSet<vfs::Slot>,
	pub(super) folded: FxHashSet<FolderSlot>,
	pub(super) content_id: FxHashMap<FileSlot, ContentId>,
}

impl FileViewer {
	#[must_use]
	pub fn new(vfs: &VirtualFs) -> Self {
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
}

pub(super) fn ui(ed: &mut Editor, ui: &mut egui::Ui, vfs: &mut VirtualFs) {
	ui.horizontal(|ui| {
		ui.label("Filter:");

		let resp = ui.text_edit_singleline(&mut ed.file_viewer.filter_buf);

		if resp.changed() {
			if !ed.file_viewer.filter_regex {
				let pattern = regex::escape(&ed.file_viewer.filter_buf);
				ed.file_viewer.filter = RegexBuilder::new(&pattern).case_insensitive(true).build();
				debug_assert!(ed.file_viewer.filter.is_ok());
			} else {
				ed.file_viewer.filter = Regex::new(&ed.file_viewer.filter_buf);
			}
		}

		if ui.button(".*").on_hover_text("Regex").clicked() {
			ed.file_viewer.filter_regex = !ed.file_viewer.filter_regex;
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
			ui_folder(ed, vfs.root(), &mut body, 0, row_height);
		});
}

fn ui_folder(
	ed: &mut Editor,
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
			label.push_str(vfolder.name());
			ui.label(&label);
		});

		let _ = row.col(|ui| {
			let label = match vfolder.kind() {
				FolderKind::Directory => "directory",
				FolderKind::Root => "VFS root",
				FolderKind::Wad => "WAD archive",
				FolderKind::Zip => "zip archive",
				FolderKind::ZipDir => "zip directory",
			};

			ui.label(label);
		});

		let _ = row.col(|_| {
			// Folders have no bytes.
		});
	});

	if !folded {
		for subfolder in vfolder.subfolders() {
			ui_folder(ed, subfolder, body, depth + 1, row_height);
		}

		let mut markers = WadMarkers::None;

		for file in vfolder.files() {
			if file.name().eq_ignore_ascii_case("F_START") {
				markers = WadMarkers::Flats;
			} else if file.name().eq_ignore_ascii_case("F_END") {
				markers = WadMarkers::None;
			} // TODO: expand on this system.

			ui_file(ed, file, body, depth + 1, row_height, markers);
		}
	}
}

fn ui_file(
	ed: &mut Editor,
	vfile: FileRef,
	body: &mut TableBody,
	depth: u32,
	row_height: f32,
	markers: WadMarkers,
) {
	if let Ok(rgx) = ed.file_viewer.filter.as_ref() {
		if !rgx.is_match(vfile.name()) {
			return;
		}
	}

	let mut guard = vfile.lock();
	let bytes = guard
		.read()
		.expect("failed to read from VFS in-memory file");

	let id = *ed
		.file_viewer
		.content_id
		.entry(vfile.slot())
		.or_insert(ContentId::deduce(&vfile, &bytes, markers));

	body.row(row_height, |mut row| {
		let (_, _) = row.col(|ui| {
			let mut label = String::new();
			label.push_str(vfile.name());
			ui.add_space((depth as f32) * 16.0);
			ui.label(&label);
		});

		let (_, _) = row.col(|ui| {
			ui.label(&format!("{id}"));
		});

		let (_, _) = row.col(|ui| {
			ui.label(&subdivide_file_len(vfile.size()));
		});
	});
}

#[must_use]
fn subdivide_file_len(len: usize) -> String {
	if len == 0 {
		return "0 B".to_string();
	}

	let mut len = len as f32;
	let mut unit = "B";

	if len > 1024.0 {
		len /= 1024.0;
		unit = "KB";
	} else {
		return format!("{len} {unit}");
	}

	if len > 1024.0 {
		len /= 1024.0;
		unit = "MB";
	}

	if len > 1024.0 {
		len /= 1024.0;
		unit = "GB";
	}

	format!("{len:.2} {unit}")
}
