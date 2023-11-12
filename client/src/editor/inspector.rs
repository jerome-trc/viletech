use std::sync::Arc;

use bevy_egui::egui;
use viletech::{
	data::gfx::PaletteSet,
	vfs::{self, FileSlot},
};

use super::{contentid::ContentId, Editor, EditorCommon, WorkBuf};

pub(super) fn ui(ed: &mut Editor, ui: &mut egui::Ui, core: &mut EditorCommon) {
	match ed.file_viewer.selected.len() {
		0 => {}
		1 => {
			let slot = ed.file_viewer.selected.iter().copied().next().unwrap();

			let vfs::Slot::File(islot) = slot else {
				return;
			};

			ui_inspect(ed, ui, core, islot);
		}
		n => {
			ui.centered_and_justified(|ui| {
				ui.label(&format!("{n} files selected"));
			});
		}
	}
}

fn ui_inspect(ed: &mut Editor, ui: &mut egui::Ui, core: &mut EditorCommon, slot: FileSlot) {
	let content_id = ed.file_viewer.content_id.get(&slot).unwrap();

	if content_id.is_text() {
		ui_inspect_text(ed, ui, core, slot);
		return;
	}

	match content_id {
		ContentId::Flat => ui_inspect_flat(ed, ui, core, slot),
		ContentId::PlayPal => ui_inspect_playpal(ed, ui, core, slot),
		_ => {}
	}
}

fn ui_inspect_flat(ed: &mut Editor, ui: &mut egui::Ui, core: &mut EditorCommon, slot: FileSlot) {
	let vfile = core.vfs.get_file(slot).unwrap();
	let uri = format!("vfs:/{}", vfile.path());

	let wbuf = ed.workbufs.entry(slot).or_insert({
		let Some(palset) = ed.palset.as_ref() else {
			// TODO: VTEd should ship palettes of its own.
			return;
		};

		let Some(colormap) = ed.colormap.as_ref() else {
			// TODO: VTEd should ship a colormap of its own.
			return;
		};

		let mut guard = vfile.lock();
		let bytes = guard.read().expect("VFS memory read failed");

		if bytes.len() < 4096 {
			// TODO: report an error.
			return;
		}

		let palette = &palset.0[0];
		let texname = uri.clone();
		let texman_arc = ui.ctx().tex_manager();
		let mut texman = texman_arc.write();
		let mut color_img = egui::ColorImage::new([64, 64], egui::Color32::TEMPORARY_COLOR);

		for y in 0..64 {
			for x in 0..64 {
				let i = (y * 64) + x;
				let map_entry = bytes[i];
				let pal_entry = colormap.0[0][map_entry as usize];
				let pixel = palette.0[pal_entry as usize];
				let r = (pixel.0[0] * 255.0) as u8;
				let g = (pixel.0[1] * 255.0) as u8;
				let b = (pixel.0[2] * 255.0) as u8;
				color_img.pixels[i] = egui::Color32::from_rgb(r, g, b);
			}
		}

		let texid = texman.alloc(
			texname,
			egui::ImageData::Color(Arc::new(color_img)),
			egui::TextureOptions::NEAREST,
		);

		WorkBuf::Image(texid)
	});

	let WorkBuf::Image(texid) = wbuf else {
		unreachable!()
	};

	let imgsrc = egui::ImageSource::Texture(egui::load::SizedTexture {
		id: *texid,
		size: egui::Vec2::new(64.0, 64.0),
	});

	ui.centered_and_justified(|ui| {
		// TODO: upscaling, even with `NEAREST` filtering, uses linear filtering.
		// Is this intentional behavior or a bug?
		ui.add(egui::Image::new(imgsrc).texture_options(egui::TextureOptions::NEAREST));
	});
}

/// TODO: this is dreadfully inefficient, but it can't be improved until VTEd's
/// architecture crystallizes further.
fn ui_inspect_playpal(_: &mut Editor, ui: &mut egui::Ui, core: &mut EditorCommon, slot: FileSlot) {
	let vfile = core.vfs.get_file(slot).unwrap();
	let mut guard = vfile.lock();
	let bytes = guard.read().expect("VFS memory read failed");

	let Ok(palset) = PaletteSet::new(bytes.as_ref()) else {
		// TODO: report an error.
		return;
	};

	ui.horizontal_wrapped(|ui| {
		for color in palset.0[0].0.iter() {
			let (rect, _resp) =
				ui.allocate_at_least(egui::Vec2::new(32.0, 32.0), egui::Sense::hover());

			let r = (color.0[0] * 255.0) as u8;
			let g = (color.0[1] * 255.0) as u8;
			let b = (color.0[2] * 255.0) as u8;

			ui.painter().rect(
				rect,
				1.0,
				egui::Color32::from_rgb(r, g, b),
				egui::Stroke::new(0.0, egui::Color32::TRANSPARENT),
			);
		}
	});
}

fn ui_inspect_text(ed: &mut Editor, ui: &mut egui::Ui, core: &mut EditorCommon, slot: FileSlot) {
	// TODO:
	// - Save and revert functionality.
	// - egui's TextEdit widget isn't good enough here. It will gladly eat up
	// multiple gigabytes of RAM to hold the content of a UDMF TEXTMAP file containing
	// a few megabytes.
	let wbuf = ed.workbufs.entry(slot).or_insert(WorkBuf::Text({
		let vfile = core.vfs.get_file(slot).unwrap();
		let mut guard = vfile.lock();
		let bytes = guard.read().expect("VFS memory read failed");
		String::from_utf8_lossy(bytes.as_ref()).into_owned()
	}));

	let WorkBuf::Text(string) = wbuf else {
		unreachable!()
	};

	ui.centered_and_justified(|ui| {
		if string.len() > 1024 * 1024 {
			ui.label("Editing files over 1 MB large is not yet supported.");
			return;
		}

		ui.text_edit_multiline(string);
	});
}
