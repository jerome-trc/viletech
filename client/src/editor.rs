//! Functions run when entering, updating, and leaving [`AppState::Editor`].

pub(crate) mod contentid;
pub(crate) mod fileview;
pub(crate) mod inspector;
pub(crate) mod leveled;

use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_egui::{
	egui::{self, TextureId},
	EguiContexts,
};
use parking_lot::Mutex;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use viletech::{
	data::gfx::{ColorMapSet, PaletteSet},
	vfs::FileSlot,
	VirtualFs,
};

use crate::AppState;

use self::{fileview::FileViewer, leveled::LevelEditor};

#[derive(SystemParam)]
pub(crate) struct EventReaders<'w, 's> {
	pub(crate) fileview: EventReader<'w, 's, fileview::Event>,
}

#[derive(Resource, Debug)]
pub(crate) struct Editor {
	panel_l: Option<Dialog>,
	panel_r: Option<Dialog>,
	panel_m: Dialog,
	panel_b: Option<Dialog>,

	workbufs: FxHashMap<FileSlot, WorkBuf>,
	palset: Option<PaletteSet<'static>>,
	colormaps: Option<ColorMapSet<'static>>,

	file_viewer: FileViewer,
	level_editor: LevelEditor,
}

/// What content is occupying a panel?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Dialog {
	DecoViz,
	Files,
	Inspector,
	LevelEd,
	Messages,
}

impl Editor {
	#[must_use]
	pub(crate) fn level_editor_open(&self) -> bool {
		if self.panel_m == Dialog::LevelEd {
			return true;
		}

		[self.panel_l, self.panel_r, self.panel_b]
			.iter()
			.any(|opt| opt.is_some_and(|p| p == Dialog::LevelEd))
	}
}

impl std::fmt::Display for Dialog {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DecoViz => write!(f, "\u{1F500} DecoViz"),
			Self::Files => write!(f, "\u{1F5C0} Files"),
			Self::Inspector => write!(f, "\u{1F50E} Inspector"),
			Self::LevelEd => write!(f, "\u{1F5FA} Level Editor"),
			Self::Messages => write!(f, "\u{1F4E7} Message Log"),
		}
	}
}

#[derive(Debug)]
pub(crate) enum WorkBuf {
	Image(TextureId),
	Text(String),
}

pub(crate) fn update(
	mut next_state: ResMut<NextState<AppState>>,
	mut ed: ResMut<Editor>,
	mut egui: EguiContexts,
	mut params: ParamSet<(fileview::SysParam, inspector::SysParam, leveled::SysParam)>,
) {
	let guictx = egui.ctx_mut();

	egui::TopBottomPanel::top("viletech_ed_panel_t").show(guictx, |ui| {
		ui.horizontal_wrapped(|ui| {
			ui.visuals_mut().button_frame = false;

			if ui
				.button("Back")
				.on_hover_text("Return to the frontend.")
				.clicked()
			{
				next_state.set(AppState::Frontend);
				return;
			}

			ui.separator();
			egui::widgets::global_dark_light_mode_switch(ui);
		});
	});

	if let Some(panel_l) = ed.panel_l {
		egui::SidePanel::left("viletech_ed_panel_l")
			.frame(panel_frame(guictx.style().as_ref(), panel_l, false))
			.show(guictx, |ui| {
				egui::menu::bar(ui, |ui| {
					ed.panel_l = Some(dialog_combo(ui, panel_l));
				});

				match panel_l {
					Dialog::DecoViz => {}
					Dialog::Files => fileview::ui(&mut ed, ui, params.p0()),
					Dialog::Inspector => inspector::ui(&mut ed, ui, params.p1()),
					Dialog::LevelEd => leveled::ui(&mut ed, ui, params.p2()),
					Dialog::Messages => {}
				}
			});
	}
	if let Some(panel_r) = ed.panel_r {
		egui::SidePanel::right("viletech_ed_panel_r")
			.frame(panel_frame(guictx.style().as_ref(), panel_r, false))
			.show(guictx, |ui| {
				egui::menu::bar(ui, |ui| {
					ed.panel_r = Some(dialog_combo(ui, panel_r));
				});

				match panel_r {
					Dialog::DecoViz => {}
					Dialog::Files => fileview::ui(&mut ed, ui, params.p0()),
					Dialog::Inspector => inspector::ui(&mut ed, ui, params.p1()),
					Dialog::LevelEd => leveled::ui(&mut ed, ui, params.p2()),
					Dialog::Messages => {}
				}
			});
	}

	if let Some(panel_b) = ed.panel_b {
		egui::TopBottomPanel::bottom("viletech_ed_panel_b")
			.frame(panel_frame(guictx.style().as_ref(), panel_b, false))
			.show(guictx, |ui| {
				egui::menu::bar(ui, |ui| {
					ed.panel_b = Some(dialog_combo(ui, panel_b));
				});

				match panel_b {
					Dialog::DecoViz => {}
					Dialog::Files => fileview::ui(&mut ed, ui, params.p0()),
					Dialog::Inspector => inspector::ui(&mut ed, ui, params.p1()),
					Dialog::LevelEd => leveled::ui(&mut ed, ui, params.p2()),
					Dialog::Messages => {}
				}
			});
	}

	egui::CentralPanel::default()
		.frame(panel_frame(guictx.style().as_ref(), ed.panel_m, true))
		.show(guictx, |ui| {
			egui::menu::bar(ui, |ui| {
				ed.panel_m = dialog_combo(ui, ed.panel_m);
			});

			match ed.panel_m {
				Dialog::DecoViz => {}
				Dialog::Files => fileview::ui(&mut ed, ui, params.p0()),
				Dialog::Inspector => inspector::ui(&mut ed, ui, params.p1()),
				Dialog::LevelEd => leveled::ui(&mut ed, ui, params.p2()),
				Dialog::Messages => {}
			}
		});
}

pub(crate) fn post_update(
	mut ed: ResMut<Editor>,
	mut params: ParamSet<(
		fileview::SysParam,
		inspector::SysParam,
		leveled::SysParam,
		EventReaders,
	)>,
) {
	let mut ereaders = params.p3();
	let events_fv: Vec<_> = ereaders.fileview.read().cloned().collect();

	for e_fv in events_fv {
		match e_fv {
			fileview::Event::EditLevel(islot) => {
				leveled::load(&mut ed, params.p2(), islot);
			}
		}
	}
}

#[must_use]
fn dialog_combo(ui: &mut egui::Ui, mut dialog: Dialog) -> Dialog {
	egui::ComboBox::new("viletech_ed_dialog_combo", "")
		.selected_text(format!("{}", dialog))
		.show_ui(ui, |ui| {
			ui.selectable_value(&mut dialog, Dialog::Files, "\u{1F5C0} Files");
			ui.selectable_value(&mut dialog, Dialog::Inspector, "\u{1F50E} Inspector");
			ui.selectable_value(&mut dialog, Dialog::Messages, "\u{1F4E7} Message Log");
			ui.selectable_value(&mut dialog, Dialog::DecoViz, "\u{1F500} DecoViz");
			ui.selectable_value(&mut dialog, Dialog::LevelEd, "\u{1F5FA} Level Editor");
		});

	dialog
}

// Transitions /////////////////////////////////////////////////////////////////

pub(crate) fn on_enter(mut cmds: Commands, vfs: Res<VirtualFs>) {
	let palset = Mutex::new(None);
	let colormap = Mutex::new(None);

	vfs.files().par_bridge().for_each(|vfile| {
		if vfile.name().eq_ignore_ascii_case("PLAYPAL") {
			let mut guard = vfile.lock();
			let bytes = guard.read().expect("VFS memory read failed");
			*palset.lock() = PaletteSet::new(bytes.as_ref()).ok();
		} else if vfile.name().eq_ignore_ascii_case("COLORMAP") {
			let mut guard = vfile.lock();
			let bytes = guard.read().expect("VFS memory read failed");
			*colormap.lock() = ColorMapSet::new(bytes.as_ref()).ok();
		}
	});

	cmds.insert_resource(Editor {
		panel_l: Some(Dialog::Files),
		panel_r: None,
		panel_m: Dialog::Inspector,
		panel_b: Some(Dialog::Messages),

		workbufs: FxHashMap::default(),
		palset: palset.into_inner(),
		colormaps: colormap.into_inner(),

		file_viewer: FileViewer::new(&vfs),
		level_editor: LevelEditor::default(),
	});

	cmds.spawn(Camera3dBundle {
		transform: Transform {
			translation: Vec3::new(0.0, 0.0, 50.0),
			rotation: Quat::from_euler(EulerRot::YXZ, 0.0, 0.0, 0.0),
			..Default::default()
		},
		..Default::default()
	});
}

#[allow(clippy::type_complexity)]
pub(crate) fn on_exit(
	mut cmds: Commands,
	cameras: Query<Entity, Or<(With<Camera2d>, With<Camera3d>)>>,
) {
	cmds.remove_resource::<Editor>();

	for camera in &cameras {
		cmds.entity(camera).despawn();
	}
}

// Helpers /////////////////////////////////////////////////////////////////////

#[must_use]
fn panel_frame(style: &egui::Style, dialog: Dialog, central: bool) -> egui::Frame {
	let frame = if central {
		egui::Frame::central_panel(style)
	} else {
		egui::Frame::side_top_panel(style)
	};

	if dialog == Dialog::LevelEd {
		frame.multiply_with_opacity(0.0)
	} else {
		frame
	}
}
