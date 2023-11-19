use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_egui::{
	egui::{self, TextureId},
	EguiContexts,
};
use viletech::{
	data::gfx::{ColorMapSet, PaletteSet},
	vfs::FileSlot,
	VirtualFs,
};

use crate::editor;

use super::{contentid::ContentId, Editor};

#[derive(Debug)]
pub(crate) struct Inspector {
	pub(crate) file: FileSlot,
	pub(crate) transient: bool,
	pub(crate) inspected: Inspected,
}

#[derive(Debug)]
pub(crate) enum Inspected {
	Colormap(ColorMapSet<'static>),
	Image(Handle<Image>, TextureId),
	Marker,
	PlayPal(PaletteSet<'static>, usize),
	Text(String),
	Unsupported,
}

#[derive(SystemParam)]
pub(crate) struct SysParam<'w> {
	pub(crate) ewriter: EventWriter<'w, editor::Event>,
	pub(crate) vfs: ResMut<'w, VirtualFs>,
	pub(crate) images: ResMut<'w, Assets<Image>>,
}

pub(super) fn ui(ed: &mut Editor, ui: &mut egui::Ui, mut param: SysParam) {
	egui::menu::bar(ui, |ui| {
		for (i, inspector) in ed.inspectors.iter().enumerate() {
			egui::Frame::group(&ui.ctx().style()).show(ui, |ui| {
				ui.horizontal(|ui| {
					let vfile = param.vfs.get_file(inspector.file).unwrap();
					let mut text = egui::RichText::new(vfile.path().as_str());

					if inspector.transient {
						text = text.italics();
					}

					if ui.button(text).clicked() {
						ed.cur_inspector = Some(i);
					}

					if ui.button("\u{2716}").on_hover_text("Close").clicked() {
						param
							.ewriter
							.send(editor::Event::CloseInspector { index: i });
					}
				});
			});
		}
	});

	let Some(i) = ed.cur_inspector else {
		match ed.file_viewer.selected.len() {
			0 => {
				ui.centered_and_justified(|ui| {
					ui.label("Select a file to inspect its content");
				});
			}
			1 => {}
			n => {
				ui.centered_and_justified(|ui| {
					ui.label(&format!("{n} files selected"));
				});
			}
		}

		return;
	};

	let slot = ed.inspectors[i].file;
	let content_id = ed.file_viewer.content_id.get(&slot).unwrap();

	if let Inspected::Marker = &ed.inspectors[i].inspected {
		return;
	}

	ui.separator();

	if content_id.is_text() {
		ui_inspect_text(ed, ui, param);
		return;
	}

	match content_id {
		ContentId::Colormap => ui_inspect_colormap(ed, ui, param),
		ContentId::Flat => ui_inspect_flat(ed, ui, param),
		ContentId::Picture => ui_inspect_picture(ed, ui, param),
		ContentId::PlayPal => ui_inspect_playpal(ed, ui, param),
		_ => {
			ui.centered_and_justified(|ui| {
				ui.label("Inspecting this kind of file is not yet supported");
			});
		}
	}
}

fn ui_inspect_colormap(_: &mut Editor, _: &mut egui::Ui, _: SysParam) {
	// TODO
}

fn ui_inspect_flat(ed: &mut Editor, ui: &mut egui::Ui, _: SysParam) {
	let Inspected::Image(_img_h, tex_id) = &mut ed.inspectors[ed.cur_inspector.unwrap()].inspected
	else {
		unreachable!()
	};

	let imgsrc = egui::ImageSource::Texture(egui::load::SizedTexture {
		id: *tex_id,
		size: egui::Vec2::new(64.0, 64.0),
	});

	ui.centered_and_justified(|ui| {
		// TODO: upscaling, even with `NEAREST` filtering, uses linear filtering.
		// Is this intentional behavior or a bug?
		ui.add(egui::Image::new(imgsrc).texture_options(egui::TextureOptions::NEAREST));
	});
}

fn ui_inspect_picture(ed: &mut Editor, ui: &mut egui::Ui, param: SysParam) {
	let Inspected::Image(img_h, tex_id) = &mut ed.inspectors[ed.cur_inspector.unwrap()].inspected
	else {
		unreachable!()
	};

	let img_ref = param.images.get(img_h.clone()).unwrap();

	let imgsrc = egui::ImageSource::Texture(egui::load::SizedTexture {
		id: *tex_id,
		size: egui::Vec2::new(img_ref.width() as f32, img_ref.height() as f32),
	});

	ui.centered_and_justified(|ui| {
		// TODO: upscaling, even with `NEAREST` filtering, uses linear filtering.
		// Is this intentional behavior or a bug?
		ui.add(egui::Image::new(imgsrc).texture_options(egui::TextureOptions::NEAREST));
	});
}

fn ui_inspect_playpal(ed: &mut Editor, ui: &mut egui::Ui, mut param: SysParam) {
	let Inspected::PlayPal(palset, index) = &mut ed.inspectors[ed.cur_inspector.unwrap()].inspected
	else {
		unreachable!()
	};

	menu_bar(&mut param, ui, |ui| {
		ui.add_enabled_ui(*index > 0, |ui| {
			if ui
				.button("\u{23EE}")
				.on_hover_text("First Palette")
				.clicked()
			{
				*index = 0;
			}

			if ui
				.button("\u{2B05}")
				.on_hover_text("Previous Palette")
				.clicked()
			{
				*index -= 1;
			}
		});

		ui.label(&format!("{}/14", *index + 1));

		ui.add_enabled_ui(*index < 13, |ui| {
			if ui
				.button("\u{27A1}")
				.on_hover_text("Next Palette")
				.clicked()
			{
				*index += 1;
			}

			if ui
				.button("\u{23ED}")
				.on_hover_text("Last Palette")
				.clicked()
			{
				*index = 13;
			}
		});
	});

	ui.horizontal_wrapped(|ui| {
		for (i, color) in palset[*index].0.iter().enumerate() {
			let (rect, _resp) =
				ui.allocate_at_least(egui::Vec2::new(32.0, 32.0), egui::Sense::hover());

			ui.painter().rect(
				rect,
				1.0,
				egui::Color32::from_rgb(color.r, color.g, color.b),
				egui::Stroke::new(0.0, egui::Color32::TRANSPARENT),
			);

			if ui
				.ctx()
				.pointer_hover_pos()
				.is_some_and(|p| rect.contains(p))
			{
				egui::show_tooltip_at_pointer(
					ui.ctx(),
					egui::Id::new("viletech_ed_colormap_tt"),
					|ui| {
						ui.label(&format!("{i}: R {}, G {}, B {}", color.r, color.g, color.b));
					},
				);
			}
		}
	});
}

fn ui_inspect_text(ed: &mut Editor, ui: &mut egui::Ui, _: SysParam) {
	// TODO: egui's TextEdit widget isn't good enough here. It will gladly eat up
	// multiple gigabytes of RAM to hold the content of a UDMF TEXTMAP file containing
	// a few megabytes.

	let Inspected::Text(string) = &mut ed.current_inspector_mut().inspected else {
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

pub(super) fn open(
	ed: &mut Editor,
	egui: &mut EguiContexts,
	mut param: SysParam,
	file: FileSlot,
	transient: bool,
) {
	if let Some(pos) = ed.inspectors.iter().position(|insp| insp.transient) {
		close(ed, egui, &mut param, pos);
	}

	let content_id = ed.file_viewer.content_id.get(&file).copied().unwrap();

	let vfile = param.vfs.get_file(file).unwrap();
	let mut guard = vfile.lock();
	let bytes = guard.read().expect("VFS memory read failed");

	if content_id.is_text() {
		ed.inspectors.push(Inspector {
			file,
			transient,
			inspected: Inspected::Text(String::from_utf8_lossy(bytes.as_ref()).into_owned()),
		});

		return;
	}

	let inspected = match content_id {
		ContentId::Flat => {
			let Some(palset) = ed.palset.as_ref() else {
				// TODO: VTEd should ship palettes of its own.
				ed.messages
					.push("No palette available: cannot display this graphic.".into());
				return;
			};

			let Some(colormaps) = ed.colormaps.as_ref() else {
				// TODO: VTEd should ship a colormap of its own.
				ed.messages
					.push("No colormap available; cannot inspect this graphic.".into());
				return;
			};

			let palette = &palset[0];
			let colormap = &colormaps[0];

			let image = viletech::asset::flat_to_image(bytes.as_ref(), palette, colormap, None);
			let img_h = param.images.add(image);
			let tex_id = egui.add_image(img_h.clone());
			Inspected::Image(img_h, tex_id)
		}
		ContentId::Picture => {
			let Some(palset) = ed.palset.as_ref() else {
				// TODO: VTEd should ship palettes of its own.
				ed.messages
					.push("No palette available: cannot display this graphic.".into());
				return;
			};

			let Some(colormaps) = ed.colormaps.as_ref() else {
				// TODO: VTEd should ship a colormap of its own.
				ed.messages
					.push("No colormap available; cannot inspect this graphic.".into());
				return;
			};

			let palette = &palset[0];
			let colormap = &colormaps[0];

			let image =
				match viletech::asset::picture_to_image(bytes.as_ref(), palette, colormap, None) {
					Ok(i) => i,
					Err(err) => {
						ed.messages
							.push(format!("This graphic is invalid: {err}").into());
						return;
					}
				};

			let img_h = param.images.add(image);
			let tex_id = egui.add_image(img_h.clone());
			Inspected::Image(img_h, tex_id)
		}
		ContentId::Colormap => {
			let colormap = match ColorMapSet::new(bytes.as_ref()) {
				Ok(cm) => cm,
				Err(err) => {
					ed.messages
						.push(format!("Invalid COLORMAP lump: {err}").into());
					return;
				}
			};

			Inspected::Colormap(ColorMapSet::Owned(Box::new(colormap.into_owned())))
		}
		ContentId::PlayPal => {
			let palset = match PaletteSet::new(bytes.as_ref()) {
				Ok(p) => p,
				Err(err) => {
					ed.messages
						.push(format!("Invalid PLAYPAL lump: {err}").into());
					return;
				}
			};

			Inspected::PlayPal(PaletteSet::Owned(Box::new(palset.into_owned())), 0)
		}
		ContentId::Marker => Inspected::Marker,
		_ => Inspected::Unsupported,
	};

	ed.inspectors.push(Inspector {
		file,
		transient,
		inspected,
	});

	ed.cur_inspector = Some(ed.inspectors.len() - 1);
}

pub(super) fn close(ed: &mut Editor, egui: &mut EguiContexts, _: &mut SysParam, index: usize) {
	let inspector = ed.inspectors.remove(index);
	let new_index = index.saturating_sub(1);

	ed.cur_inspector = if ed.inspectors.len() > new_index {
		Some(new_index)
	} else {
		None
	};

	match inspector.inspected {
		Inspected::Image(img_h, tex_id) => {
			let removed = egui.remove_image(&img_h).unwrap();
			debug_assert_eq!(removed, tex_id);
		}
		Inspected::Marker
		| Inspected::Colormap(_)
		| Inspected::PlayPal(_, _)
		| Inspected::Text(_)
		| Inspected::Unsupported => {}
	}
}

// Details /////////////////////////////////////////////////////////////////////

fn menu_bar<F, R>(
	_: &mut SysParam,
	ui: &mut egui::Ui,
	mut add_contents: F,
) -> egui::InnerResponse<R>
where
	F: FnMut(&mut egui::Ui) -> R,
{
	let ret = egui::menu::bar(ui, |ui| {
		if ui.button("Save").clicked() {
			unimplemented!()
		}

		if ui.button("Revert").clicked() {
			unimplemented!()
		}

		add_contents(ui)
	});

	ui.separator();

	ret
}
