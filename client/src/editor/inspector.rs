use std::sync::Arc;

use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_egui::egui;
use image::{ImageBuffer, Rgba};
use viletech::{
	data::gfx::{PaletteSet, PictureReader},
	vfs::{self, FileSlot},
	VirtualFs,
};

use super::{contentid::ContentId, Editor, WorkBuf};

#[derive(SystemParam)]
pub(crate) struct SysParam<'w> {
	pub(crate) vfs: ResMut<'w, VirtualFs>,
}

pub(super) fn ui(ed: &mut Editor, ui: &mut egui::Ui, param: SysParam) {
	match ed.file_viewer.selected.len() {
		0 => {}
		1 => {
			let slot = ed.file_viewer.selected.iter().copied().next().unwrap();

			let vfs::Slot::File(islot) = slot else {
				return;
			};

			ui_inspect(ed, ui, param, islot);
		}
		n => {
			ui.centered_and_justified(|ui| {
				ui.label(&format!("{n} files selected"));
			});
		}
	}
}

fn ui_inspect(ed: &mut Editor, ui: &mut egui::Ui, param: SysParam, slot: FileSlot) {
	let content_id = ed.file_viewer.content_id.get(&slot).unwrap();

	if content_id.is_text() {
		ui_inspect_text(ed, ui, param, slot);
		return;
	}

	match content_id {
		ContentId::Flat => ui_inspect_flat(ed, ui, param, slot),
		ContentId::Picture => ui_inspect_picture(ed, ui, param, slot),
		ContentId::PlayPal => ui_inspect_playpal(ed, ui, param, slot),
		_ => {}
	}
}

fn ui_inspect_flat(ed: &mut Editor, ui: &mut egui::Ui, param: SysParam, slot: FileSlot) {
	let wbuf = ed.workbufs.entry(slot).or_insert({
		let Some(palset) = ed.palset.as_ref() else {
			// TODO: VTEd should ship palettes of its own.
			ui.centered_and_justified(|ui| {
				ui.label("No palette available; cannot display this graphic.");
			});

			return;
		};

		let Some(colormaps) = ed.colormaps.as_ref() else {
			// TODO: VTEd should ship a colormap of its own.
			ui.centered_and_justified(|ui| {
				ui.label("No colormap available; cannot display this graphic.");
			});

			return;
		};

		let vfile = param.vfs.get_file(slot).unwrap();
		let mut guard = vfile.lock();
		let bytes = guard.read().expect("VFS memory read failed");

		if bytes.len() < 4096 {
			ui.centered_and_justified(|ui| {
				ui.label("This flat is invalid (should be 4096 bytes).");
			});

			return;
		}

		let uri = format!("vfs:/{}", vfile.path());
		let texman_arc = ui.ctx().tex_manager();
		let mut texman = texman_arc.write();
		let mut color_img = egui::ColorImage::new([64, 64], egui::Color32::TEMPORARY_COLOR);

		let palette = &palset[0];
		let colormap = &colormaps[0];

		for y in 0..64 {
			for x in 0..64 {
				let i = (y * 64) + x;
				let map_entry = bytes[i];
				let pal_entry = colormap[map_entry as usize];
				let pixel = palette[pal_entry as usize];
				color_img.pixels[i] = egui::Color32::from_rgb(pixel.r, pixel.g, pixel.b);
			}
		}

		let texid = texman.alloc(
			uri,
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

fn ui_inspect_picture(ed: &mut Editor, ui: &mut egui::Ui, param: SysParam, slot: FileSlot) {
	let wbuf = ed.workbufs.entry(slot).or_insert({
		let Some(palset) = ed.palset.as_ref() else {
			// TODO: VTEd should ship palettes of its own.
			ui.centered_and_justified(|ui| {
				ui.label("No palette available; cannot display this graphic.");
			});

			return;
		};

		let Some(colormaps) = ed.colormaps.as_ref() else {
			// TODO: VTEd should ship a colormap of its own.
			ui.centered_and_justified(|ui| {
				ui.label("No colormap available; cannot display this graphic.");
			});

			return;
		};

		let vfile = param.vfs.get_file(slot).unwrap();
		let mut guard = vfile.lock();
		let bytes = guard.read().expect("VFS memory read failed");

		let palette = &palset[0];
		let colormap = &colormaps[0];

		let pic_reader = match PictureReader::new(bytes.as_ref(), palette, colormap) {
			Ok(pr) => pr,
			Err(err) => {
				ui.centered_and_justified(|ui| {
					ui.label(&format!("This graphic is invalid: {err}"));
				});

				return;
			}
		};

		let dims = [pic_reader.width() as usize, pic_reader.height() as usize];
		let mut img_buf = ImageBuffer::new(dims[0] as u32, dims[1] as u32);

		pic_reader.read(|row, col, pixel| {
			img_buf.put_pixel(row, col, Rgba([pixel.r, pixel.g, pixel.b, 255]))
		});

		let mut color_img = egui::ColorImage::new(dims, egui::Color32::TEMPORARY_COLOR);

		for (i, pixel) in img_buf.pixels().enumerate() {
			color_img.pixels[i] =
				egui::Color32::from_rgba_unmultiplied(pixel[0], pixel[1], pixel[2], pixel[3]);
		}

		let uri = format!("vfs:/{}", vfile.path());
		let texman_arc = ui.ctx().tex_manager();
		let mut texman = texman_arc.write();

		let texid = texman.alloc(
			uri,
			egui::ImageData::Color(Arc::new(color_img)),
			egui::TextureOptions::NEAREST,
		);

		WorkBuf::Image(texid)
	});

	let WorkBuf::Image(texid) = wbuf else {
		unreachable!()
	};

	let texman_arc = ui.ctx().tex_manager();
	let texman = texman_arc.read();
	let tex_meta = texman.meta(*texid).unwrap();

	let imgsrc = egui::ImageSource::Texture(egui::load::SizedTexture {
		id: *texid,
		size: egui::Vec2::new(tex_meta.size[0] as f32, tex_meta.size[1] as f32),
	});

	ui.centered_and_justified(|ui| {
		// TODO: upscaling, even with `NEAREST` filtering, uses linear filtering.
		// Is this intentional behavior or a bug?
		ui.add(egui::Image::new(imgsrc).texture_options(egui::TextureOptions::NEAREST));
	});
}

fn ui_inspect_playpal(_: &mut Editor, ui: &mut egui::Ui, param: SysParam, slot: FileSlot) {
	let vfile = param.vfs.get_file(slot).unwrap();
	let mut guard = vfile.lock();
	let bytes = guard.read().expect("VFS memory read failed");

	let Ok(palset) = PaletteSet::new(bytes.as_ref()) else {
		// TODO: report an error.
		return;
	};

	ui.horizontal_wrapped(|ui| {
		for color in palset[0].0.iter() {
			let (rect, _resp) =
				ui.allocate_at_least(egui::Vec2::new(32.0, 32.0), egui::Sense::hover());

			ui.painter().rect(
				rect,
				1.0,
				egui::Color32::from_rgb(color.r, color.g, color.b),
				egui::Stroke::new(0.0, egui::Color32::TRANSPARENT),
			);
		}
	});
}

fn ui_inspect_text(ed: &mut Editor, ui: &mut egui::Ui, param: SysParam, slot: FileSlot) {
	// TODO:
	// - Save and revert functionality.
	// - egui's TextEdit widget isn't good enough here. It will gladly eat up
	// multiple gigabytes of RAM to hold the content of a UDMF TEXTMAP file containing
	// a few megabytes.
	let wbuf = ed.workbufs.entry(slot).or_insert(WorkBuf::Text({
		let vfile = param.vfs.get_file(slot).unwrap();
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
