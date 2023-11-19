use std::time::Instant;

use bevy::{
	ecs::system::SystemParam,
	prelude::*,
	render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_egui::egui;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use viletech::{
	gfx::Sky2dMaterial,
	level::{read::prelude::*, RawLevel, RawThings},
	tracing::info,
	vfs::FileSlot,
	VirtualFs,
};

use crate::{common::InputParam, editor::contentid::ContentId};

use super::Editor;

#[derive(Debug, Default)]
pub(crate) struct LevelEditor {
	pub(crate) current: Option<EditedLevel>,
	pub(crate) viewpoint: Viewpoint,

	pub(crate) materials: FxHashMap<FileSlot, Handle<StandardMaterial>>,
}

#[derive(Debug)]
pub(crate) enum EditedLevel {
	Vanilla { _marker: FileSlot },
	_Udmf { _marker: FileSlot },
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Viewpoint {
	#[default]
	TopDown,
	Free,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DrawMode {
	#[default]
	Bsp,
}

#[derive(Component, Debug)]
#[component(storage = "SparseSet")]
pub(crate) struct EdSector(Handle<Mesh>);

#[derive(SystemParam)]
pub(crate) struct SysParam<'w, 's> {
	pub(crate) cmds: Commands<'w, 's>,
	pub(crate) vfs: ResMut<'w, VirtualFs>,
	pub(crate) input: InputParam<'w, 's>,
	pub(crate) time: Res<'w, Time>,
	pub(crate) cameras: Query<'w, 's, &'static mut Transform, With<Camera>>,
	pub(crate) _gizmos: bevy::gizmos::gizmos::Gizmos<'s>,

	pub(crate) meshes: ResMut<'w, Assets<Mesh>>,
	pub(crate) images: ResMut<'w, Assets<Image>>,
	pub(crate) mtrs_std: ResMut<'w, Assets<StandardMaterial>>,
	pub(crate) _mtrs_sky: ResMut<'w, Assets<Sky2dMaterial>>,
}

pub(super) fn ui(ed: &mut Editor, ui: &mut egui::Ui, mut param: SysParam) {
	if ed.level_editor.current.as_ref().is_none() {
		ui.centered_and_justified(|ui| {
			ui.label("No level is currently loaded.");
		});

		return;
	};

	let mut camera = param.cameras.single_mut();

	let (mut yaw, mut pitch, roll) = camera.rotation.to_euler(EulerRot::YXZ);
	let txt_height = ui.text_style_height(&egui::TextStyle::Body);

	let overlay = egui::Window::new("")
		.id("vted_leveled_overlay".into())
		.title_bar(false)
		.anchor(egui::Align2::RIGHT_TOP, [txt_height, txt_height]);

	match ed.level_editor.viewpoint {
		Viewpoint::TopDown => {
			overlay.show(ui.ctx(), |ui| {
				ui.label(format!(
					"X: {:.2} - Y: {:.2} - Z: {:.2}",
					camera.translation.x, camera.translation.y, camera.translation.z
				));
			});
		}
		Viewpoint::Free => {
			overlay.show(ui.ctx(), |ui| {
				ui.label(format!(
					"X: {:.2} - Y: {:.2} - Z: {:.2}",
					camera.translation.x, camera.translation.y, camera.translation.z
				));

				ui.label(format!(
					"Yaw: {:.2} - Pitch: {:.2} - Roll: {:.2}",
					yaw, pitch, roll
				));
			});
		}
	}

	if !ui.rect_contains_pointer(ui.min_rect()) {
		return;
	}

	if param.input.keys.just_released(KeyCode::F) {
		ed.level_editor.viewpoint = match ed.level_editor.viewpoint {
			Viewpoint::TopDown => Viewpoint::Free,
			Viewpoint::Free => Viewpoint::TopDown,
		};
	}

	let dt = param.time.delta_seconds();

	match ed.level_editor.viewpoint {
		Viewpoint::TopDown => {
			if param.input.mouse.pressed(MouseButton::Middle) {
				let mouse_delta = param
					.input
					.mouse_motion
					.read()
					.fold(Vec2::ZERO, |v, mm| v + mm.delta);

				camera.translation.x += dt * mouse_delta.x;
				camera.translation.y -= dt * mouse_delta.y;
			}

			let speed = if param.input.keys.pressed(KeyCode::ShiftLeft) {
				12.0
			} else {
				6.0
			};

			if param.input.keys.pressed(KeyCode::Up) {
				camera.translation.y += dt * speed;
			}

			if param.input.keys.pressed(KeyCode::Down) {
				camera.translation.y -= dt * speed;
			}

			if param.input.keys.pressed(KeyCode::Right) {
				camera.translation.x += dt * speed;
			}

			if param.input.keys.pressed(KeyCode::Left) {
				camera.translation.x -= dt * speed;
			}

			for ev_mwheel in param.input.events.ev_mouse_wheel.read() {
				camera.translation.z -= ev_mwheel.y * 2.0;
			}
		}
		Viewpoint::Free => {
			let mut axis_input = Vec3::ZERO;

			if param.input.keys.pressed(KeyCode::W) {
				axis_input.z += 1.0;
			}

			if param.input.keys.pressed(KeyCode::S) {
				axis_input.z -= 1.0;
			}

			if param.input.keys.pressed(KeyCode::D) {
				axis_input.x += 1.0;
			}

			if param.input.keys.pressed(KeyCode::A) {
				axis_input.x -= 1.0;
			}

			if param.input.keys.pressed(KeyCode::Q) {
				axis_input.y += 1.0;
			}

			if param.input.keys.pressed(KeyCode::E) {
				axis_input.y -= 1.0;
			}

			let mut vel = Vec3::ZERO;

			if axis_input != Vec3::ZERO {
				vel = axis_input.normalize() * 2.0;
			}

			if param.input.keys.pressed(KeyCode::ShiftLeft) {
				vel *= 2.0;
			}

			let f = vel.z * dt * camera.forward();
			let r = vel.x * dt * camera.right();
			let u = vel.y * dt * Vec3::Y;

			camera.translation += f + r + u;

			let mut mouse_delta = Vec2::ZERO;

			for mouse_event in param.input.mouse_motion.read() {
				mouse_delta += mouse_event.delta;
			}

			if mouse_delta != Vec2::ZERO {
				pitch = (pitch - mouse_delta.y * 0.05 * dt)
					.clamp(-std::f32::consts::PI / 2.0, std::f32::consts::PI / 2.0);
				yaw -= mouse_delta.x * 0.1 * dt;

				camera.rotation = Quat::from_euler(EulerRot::ZYX, 0.0, yaw, pitch);
			}
		}
	}
}

pub(super) fn load(ed: &mut Editor, mut param: SysParam, marker_slot: FileSlot) {
	let start_time = Instant::now();
	let prev_mtr_count = param.mtrs_std.len();

	let marker = param.vfs.get_file(marker_slot).unwrap();

	let sky1_opt = marker.vfs().files().par_bridge().find_map_any(|vfile| {
		const SKY_TEXNAMES: &[&str] = &["SKY1", "SKY2", "SKY3", "RSKY1", "RSKY2", "RSKY3"];

		let Some(texname) = SKY_TEXNAMES
			.iter()
			.copied()
			.find(|t| vfile.name().eq_ignore_ascii_case(t))
		else {
			return None;
		};

		let mut guard = vfile.lock();
		let bytes = guard.read().expect("VFS memory read failed");

		let Some(palset) = ed.palset.as_ref() else {
			// TODO: VTEd should ship palettes of its own.
			return None;
		};

		let Some(colormaps) = ed.colormaps.as_ref() else {
			// TODO: VTEd should ship a colormap of its own.
			return None;
		};

		let Ok(img) = viletech::asset::picture_to_image(
			bytes.as_ref(),
			&palset[0],
			&colormaps[0],
			Some(texname.to_string()),
		) else {
			return None;
		};

		Some(img)
	});

	let Some(sky1) = sky1_opt else {
		ed.messages
			.push("Level load failed: sky texture loading is currently limited.".into());

		return;
	};

	let _ = param.images.add(sky1);

	let Some(fref_things) = marker
		.next_sibling()
		.filter(|f| f.name().eq_ignore_ascii_case("THINGS"))
	else {
		ed.messages
			.push("Level load failed: THINGS lump not found.".into());

		return;
	};

	let mut guard = fref_things.lock();
	let bytes = guard.read().expect("VFS memory read failed");

	let Ok(thingdefs) = viletech::level::read::things(bytes.as_ref()) else {
		ed.messages
			.push("Level load failed: error during things reading.".into());
		return;
	};

	let Some(fref_linedefs) = fref_things
		.next_sibling()
		.filter(|f| f.name().eq_ignore_ascii_case("LINEDEFS"))
	else {
		ed.messages
			.push("Level load failed: LINEDEFS lump not found.".into());
		return;
	};

	let mut guard = fref_linedefs.lock();
	let bytes = guard.read().expect("VFS memory read failed");

	let Ok(linedefs) = viletech::level::read::linedefs(bytes.as_ref()) else {
		ed.messages
			.push("Level load failed: error during linedef reading.".into());
		return;
	};

	let Some(fref_sidedefs) = fref_linedefs
		.next_sibling()
		.filter(|f| f.name().eq_ignore_ascii_case("SIDEDEFS"))
	else {
		ed.messages
			.push("Level load failed: SIDEDEFS lump not found.".into());
		return;
	};

	let mut guard = fref_sidedefs.lock();
	let bytes = guard.read().expect("VFS memory read failed");

	let Ok(sidedefs) = viletech::level::read::sidedefs(bytes.as_ref()) else {
		ed.messages
			.push("Level load failed: error during sidedef reading.".into());
		return;
	};

	let Some(fref_vertexes) = fref_sidedefs
		.next_sibling()
		.filter(|f| f.name().eq_ignore_ascii_case("VERTEXES"))
	else {
		ed.messages
			.push("Level load failed: VERTEXES lump not found.".into());
		return;
	};

	let mut guard = fref_vertexes.lock();
	let bytes = guard.read().expect("VFS memory read failed");

	let Ok(vertdefs) = viletech::level::read::vertexes(bytes.as_ref()) else {
		ed.messages
			.push("Level load failed: error during vertex reading.".into());
		return;
	};

	let Some(fref_segs) = fref_vertexes
		.next_sibling()
		.filter(|f| f.name().eq_ignore_ascii_case("SEGS"))
	else {
		ed.messages
			.push("Level load failed: SEGS lump not found.".into());
		return;
	};

	let mut guard = fref_segs.lock();
	let bytes = guard.read().expect("VFS memory read failed");

	let Ok(segdefs) = viletech::level::read::segs(bytes.as_ref()) else {
		ed.messages
			.push("Level load failed: error during segs reading.".into());
		return;
	};

	let Some(fref_ssectors) = fref_segs
		.next_sibling()
		.filter(|f| f.name().eq_ignore_ascii_case("SSECTORS"))
	else {
		ed.messages
			.push("Level load failed: SSECTORS lump not found.".into());
		return;
	};

	let mut guard = fref_ssectors.lock();
	let bytes = guard.read().expect("VFS memory read failed");

	let Ok(ssectordefs) = viletech::level::read::ssectors(bytes.as_ref()) else {
		ed.messages
			.push("Level load failed: error during sub-sector reading.".into());
		return;
	};

	let Some(fref_nodes) = fref_ssectors
		.next_sibling()
		.filter(|f| f.name().eq_ignore_ascii_case("NODES"))
	else {
		ed.messages
			.push("Level load failed: NODES lump not found.".into());
		return;
	};

	let mut guard = fref_nodes.lock();
	let bytes = guard.read().expect("VFS memory read failed");

	let Ok(nodedefs) = viletech::level::read::nodes(bytes.as_ref()) else {
		ed.messages
			.push("Level load failed: error during BSP node reading.".into());
		return;
	};

	let Some(fref_sectors) = fref_nodes
		.next_sibling()
		.filter(|f| f.name().eq_ignore_ascii_case("SECTORS"))
	else {
		ed.messages
			.push("Level load failed: SECTORS lump not found.".into());
		return;
	};

	let mut guard = fref_sectors.lock();
	let bytes = guard.read().expect("VFS memory read failed");

	let Ok(sectordefs) = viletech::level::read::sectors(bytes.as_ref()) else {
		ed.messages
			.push("Level load failed: error during sector reading.".into());
		return;
	};

	ed.level_editor.current = Some(EditedLevel::Vanilla {
		_marker: marker_slot,
	});

	let mut camera = param.cameras.single_mut();

	let (min_raw, _max_raw) = VertexRaw::bounds(vertdefs);

	camera.translation = Vec3::new(
		(min_raw[0] as f32) * viletech::world::FSCALE,
		(min_raw[1] as f32) * viletech::world::FSCALE,
		camera.translation.z,
	);

	let raw = RawLevel {
		linedefs,
		nodes: nodedefs,
		sectors: sectordefs,
		segs: segdefs,
		sidedefs,
		subsectors: ssectordefs,
		things: RawThings::Doom(thingdefs),
		vertices: vertdefs,
	};

	#[derive(Default)]
	struct MeshParts {
		verts: Vec<Vec3>,
		indices: Vec<u32>,
	}

	let mut mesh_parts_map = FxHashMap::default();

	viletech::world::mesh::subsectors_to_polygons(raw, |poly| {
		let subsect = &raw.subsectors[poly.subsector()];
		let seg = &raw.segs[subsect.first_seg() as usize];
		let line = &raw.linedefs[seg.linedef() as usize];

		let side = match seg.direction() {
			SegDirection::Front => &raw.sidedefs[line.right_side() as usize],
			SegDirection::Back => &raw.sidedefs[line.left_side().unwrap() as usize],
		};

		let mesh_parts = mesh_parts_map
			.entry(side.sector() as usize)
			.or_insert(MeshParts::default());

		let (poly_verts, poly_ixs) = poly.floor();

		for i in poly_ixs {
			mesh_parts
				.indices
				.push((*i + mesh_parts.verts.len()) as u32);
		}

		for v in poly_verts {
			mesh_parts.verts.push(*v);
		}
	});

	for (sector_ix, mesh_parts) in mesh_parts_map {
		let sector = &raw.sectors[sector_ix];
		let floor_tex_lmpname = sector.floor_texture().unwrap();

		let tex_slot_opt = param.vfs.files().par_bridge().find_map_last(|vfile| {
			(vfile
				.name()
				.eq_ignore_ascii_case(floor_tex_lmpname.as_str()))
			.then_some(vfile.slot())
		});

		let Some(tex_slot) = tex_slot_opt else {
			continue;
		};

		let material = ed.level_editor.materials.entry(tex_slot).or_insert({
			let vfile = param.vfs.get_file(tex_slot).unwrap();
			let mut guard = vfile.lock();
			let bytes = guard.read().expect("VFS memory read failed");
			let palset = ed.palset.as_ref().unwrap();
			let colormaps = ed.colormaps.as_ref().unwrap();
			let content_id = ed.file_viewer.content_id.get(&tex_slot).unwrap();

			let result = match content_id {
				ContentId::Flat => Ok(viletech::asset::flat_to_image(
					bytes.as_ref(),
					&palset[0],
					&colormaps[0],
					Some(floor_tex_lmpname.to_string()),
				)),
				ContentId::Picture => viletech::asset::picture_to_image(
					bytes.as_ref(),
					&palset[0],
					&colormaps[0],
					Some(floor_tex_lmpname.to_string()),
				),
				_ => unimplemented!(),
			};

			let Ok(img) = result else {
				continue;
			};

			let img_handle = param.images.add(img);

			let mtr = StandardMaterial {
				base_color_texture: Some(img_handle.clone()),
				emissive_texture: Some(img_handle),
				emissive: Color::WHITE,
				..Default::default()
			};

			param.mtrs_std.add(mtr)
		});

		let normals = vec![Vec3::Z; mesh_parts.verts.len()];
		let mut uvs = Vec::with_capacity(mesh_parts.verts.len());

		for v in &mesh_parts.verts {
			// TODO: this needs to correctly align flats, but it doesn't yet.
			uvs.push(Vec2::new(-v.x, -v.y));
		}

		let mesh = Mesh::new(PrimitiveTopology::TriangleList)
			.with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_parts.verts)
			.with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
			.with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
			.with_indices(Some(Indices::U32(mesh_parts.indices)));

		let mesh_handle = param.meshes.add(mesh);

		let mut ecmds = param.cmds.spawn(MaterialMeshBundle {
			mesh: mesh_handle.clone(),
			material: material.clone(),
			transform: Transform {
				translation: Vec3::new(
					(min_raw[0] as f32) * viletech::world::FSCALE,
					(min_raw[1] as f32) * viletech::world::FSCALE,
					0.0,
				),
				..Default::default()
			},
			..Default::default()
		});

		ecmds.insert(EdSector(mesh_handle));
	}

	info!(
		concat!(
			"Loaded level for editing: {}\n",
			"Stats:\n",
			"\tTook {}ms\n",
			"\tNew materials: {}"
		),
		marker.path(),
		start_time.elapsed().as_millis(),
		param.mtrs_std.len() - prev_mtr_count
	);

	if !ed.level_editor_open() {
		ed.panel_m = super::Dialog::LevelEd;
	}
}
