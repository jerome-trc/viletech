//! Functions run when entering, updating, and leaving [`AppState::Editor`].

pub(crate) mod contentid;
pub(crate) mod fileview;
pub(crate) mod inspector;
pub(crate) mod leveled;

use std::borrow::Cow;

use bevy::{
	prelude::*,
	window::{PrimaryWindow, WindowMode},
};
use bevy_egui::{egui, EguiContexts};
use parking_lot::Mutex;
use rayon::prelude::*;
use viletech::{
	data::gfx::{ColorMapSet, PaletteSet},
	vfs::FileSlot,
	VirtualFs,
};

use crate::{common::NewWindow, AppState};

use self::{fileview::FileViewer, inspector::Inspector, leveled::LevelEditor};

#[derive(Event, Debug, Clone)]
pub(crate) enum Event {
	CloseInspector { index: usize },
	EditLevel { marker: FileSlot },
	Inspect { file: FileSlot, transient: bool },
}

#[derive(Resource, Debug)]
pub(crate) struct Editor {
	panel_l: Option<Dialog>,
	panel_r: Option<Dialog>,
	panel_m: Dialog,
	panel_b: Option<Dialog>,

	palset: Option<PaletteSet<'static>>,
	colormaps: Option<ColorMapSet<'static>>,

	file_viewer: FileViewer,
	inspectors: Vec<Inspector>,
	/// If `self.inspectors` is not empty, this is always `Some`.
	cur_inspector: Option<usize>,
	level_editor: LevelEditor,
	messages: Vec<Cow<'static, str>>,
}

/// What content is occupying a panel?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Dialog {
	DecoViz,
	Files,
	Inspector,
	LevelEd,
}

impl Editor {
	#[must_use]
	#[allow(unused)]
	pub(crate) fn current_inspector(&self) -> &Inspector {
		&self.inspectors[self.cur_inspector.unwrap()]
	}

	#[must_use]
	pub(crate) fn current_inspector_mut(&mut self) -> &mut Inspector {
		&mut self.inspectors[self.cur_inspector.unwrap()]
	}

	#[must_use]
	pub(crate) fn currently_inspecting(&self, slot: FileSlot) -> bool {
		self.inspectors.iter().any(|insp| insp.file == slot)
	}

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
		}
	}
}

pub(crate) fn update(
	mut next_state: ResMut<NextState<AppState>>,
	mut ed: ResMut<Editor>,
	mut egui: EguiContexts,
	mut params: ParamSet<(fileview::SysParam, inspector::SysParam, leveled::SysParam)>,
	windows: Query<Entity, With<Window>>,
	mut new_windows: EventWriter<NewWindow>,
) {
	for window in &windows {
		let guictx = egui.ctx_for_window_mut(window);

		egui::TopBottomPanel::top("viletech_ed_panel_t").show(guictx, |ui| {
			ui.horizontal_wrapped(|ui| {
				ui.visuals_mut().button_frame = false;

				egui::widgets::global_dark_light_mode_switch(ui);

				ui.separator();

				if ui
					.button("Back")
					.on_hover_text("Return to the frontend.")
					.clicked()
				{
					next_state.set(AppState::Frontend);
					return;
				}

				ui.separator();

				if ui.button("New Window").clicked() {
					let mut param = params.p2();

					let ecmds = param.cmds.spawn(Window {
						title: "VileTech Client".to_string(),
						mode: WindowMode::Windowed,
						..Default::default()
					});

					new_windows.send(NewWindow(ecmds.id()));
				}
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
				}
			});

		if !ed.messages.is_empty() {
			let txt_height = 2.0
				* guictx.fonts(|f| f.row_height(&egui::TextStyle::Body.resolve(&guictx.style())));

			egui::Window::new("viletech_ed_messages")
				.title_bar(false)
				.auto_sized()
				.anchor(egui::Align2::RIGHT_BOTTOM, [txt_height, txt_height])
				.show(guictx, |ui| {
					ed.messages.retain(|msg| {
						let inner_resp = ui.horizontal(|ui| {
							ui.label(msg.as_ref());
							ui.separator();
							ui.button("\u{2716}").on_hover_text("Dismiss").clicked()
						});

						!inner_resp.inner
					});
				});
		}
	}
}

pub(crate) fn post_update(
	mut ed: ResMut<Editor>,
	mut egui: EguiContexts,
	mut params: ParamSet<(
		fileview::SysParam,
		inspector::SysParam,
		leveled::SysParam,
		EventReader<Event>,
	)>,
) {
	let mut ereader = params.p3();
	let events_fv: Vec<_> = ereader.read().cloned().collect();

	for e_fv in events_fv {
		match e_fv {
			Event::EditLevel { marker } => {
				leveled::load(&mut ed, params.p2(), marker);
			}
			Event::Inspect { file, transient } => {
				inspector::open(&mut ed, &mut egui, params.p1(), file, transient);
			}
			Event::CloseInspector { index } => {
				inspector::close(&mut ed, &mut egui, &mut params.p1(), index);
			}
		}
	}
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
		panel_b: None,

		palset: palset.into_inner(),
		colormaps: colormap.into_inner(),

		file_viewer: FileViewer::new(&vfs),
		inspectors: vec![],
		cur_inspector: None,
		level_editor: LevelEditor::default(),
		messages: Vec::new(),
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
	mut vfs: ResMut<VirtualFs>,
	cameras: Query<Entity, Or<(With<Camera2d>, With<Camera3d>)>>,
	windows: Query<Entity, (With<Window>, Without<PrimaryWindow>)>,
) {
	cmds.remove_resource::<Editor>();

	for camera in &cameras {
		cmds.entity(camera).despawn();
	}

	for window in &windows {
		cmds.entity(window).despawn();
	}

	if let Err(err) = vfs.retain(|mntinfo| mntinfo.mount_point.as_str().ends_with("viletech")) {
		error!("Mass unmount error during editor exit: {err}");
	}
}

// Helpers /////////////////////////////////////////////////////////////////////

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

#[must_use]
fn dialog_combo(ui: &mut egui::Ui, mut dialog: Dialog) -> Dialog {
	egui::ComboBox::new("viletech_ed_dialog_combo", "")
		.selected_text(format!("{}", dialog))
		.show_ui(ui, |ui| {
			ui.selectable_value(&mut dialog, Dialog::Files, "\u{1F5C0} Files");
			ui.selectable_value(&mut dialog, Dialog::Inspector, "\u{1F50E} Inspector");
			ui.selectable_value(&mut dialog, Dialog::DecoViz, "\u{1F500} DecoViz");
			ui.selectable_value(&mut dialog, Dialog::LevelEd, "\u{1F5FA} Level Editor");
		});

	dialog
}
