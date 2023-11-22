use std::time::Instant;

use bevy::{
	prelude::*,
	render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use viletech::{
	level::{read::prelude::*, RawLevel, RawThings},
	rayon::{self, prelude::*},
	rustc_hash::FxHashMap,
	types::FxDashMap,
	util::{string::ZString, SmallString},
	vfs::FileSlot,
};

use crate::editor::{
	self,
	contentid::ContentId,
	leveled::{EdSector, EditedLevel},
	Editor,
};

use super::SysParam;

pub(crate) fn load(ed: &mut Editor, mut param: SysParam, marker_slot: FileSlot) {
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

	let mut camera = param.cameras.single_mut();

	let ([min_raw_x, min_raw_y], [max_raw_x, max_raw_y]) = VertexRaw::bounds(vertdefs);

	camera.translation = Vec3::new(
		(((max_raw_x + min_raw_x) / 2) as f32) * viletech::world::FSCALE,
		(((max_raw_y + min_raw_y) / 2) as f32) * viletech::world::FSCALE,
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

	let (textures, mesh_parts_map) = rayon::join(
		|| {
			let textures = FxDashMap::default();

			ed.file_viewer
				.content_id
				.iter()
				.par_bridge()
				.for_each(|(slot, content_id)| {
					if matches!(content_id, ContentId::Picture | ContentId::Flat) {
						let vfile = param.vfs.get_file(*slot).unwrap();
						let smallstr = SmallString::from(vfile.name().as_str());
						let _ = textures.insert(ZString(smallstr), *slot);
					}
				});

			textures.into_read_only()
		},
		|| {
			let mut mesh_parts_map = FxHashMap::default();

			viletech::world::mesh::triangulate(raw, |ss_poly| {
				let subsect = &raw.subsectors[ss_poly.subsector];
				let seg = &raw.segs[subsect.first_seg() as usize];
				let line = &raw.linedefs[seg.linedef() as usize];

				let side = match seg.direction() {
					SegDirection::Front => &raw.sidedefs[line.right_side() as usize],
					SegDirection::Back => &raw.sidedefs[line.left_side().unwrap() as usize],
				};

				let sectordef = &raw.sectors[side.sector() as usize];

				let mesh_parts = mesh_parts_map
					.entry(side.sector() as usize)
					.or_insert(MeshParts::default());

				for i in ss_poly.indices {
					let idx = (i + mesh_parts.verts.len()) as u32;
					mesh_parts.indices.push(idx);
				}

				for v in ss_poly.verts {
					mesh_parts.verts.push(Vec3::new(
						v.x,
						v.y,
						(sectordef.floor_height() as f32) * viletech::world::FSCALE,
					));
				}
			});

			mesh_parts_map
		},
	);

	let mut lcmds = param.cmds.spawn(viletech::world::level_bundle_base());

	lcmds.with_children(|cbuilder| {
		for (sector_ix, mesh_parts) in mesh_parts_map {
			let sector = &raw.sectors[sector_ix];
			let floor_tex_fname =
				ZString(SmallString::from(sector.floor_texture().unwrap().as_str()));

			let Some(tex_slot) = textures.get(&floor_tex_fname).copied() else {
				ed.messages.push(
					format!("Sector {sector_ix} references unknown texture: {floor_tex_fname}")
						.into(),
				);

				continue;
			};

			let material = match ed.level_editor.materials.entry(tex_slot) {
				std::collections::hash_map::Entry::Occupied(occ) => occ.get().clone(),
				std::collections::hash_map::Entry::Vacant(vac) => {
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
							Some(floor_tex_fname.to_string()),
						)),
						ContentId::Picture => viletech::asset::picture_to_image(
							bytes.as_ref(),
							&palset[0],
							&colormaps[0],
							Some(floor_tex_fname.to_string()),
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

					vac.insert(param.mtrs_std.add(mtr)).clone()
				}
			};

			let normals = vec![Vec3::Z; mesh_parts.verts.len()];
			let mut uvs = Vec::with_capacity(mesh_parts.verts.len());

			for v in &mesh_parts.verts {
				uvs.push(Vec2::new(-v.x / 64.0, -v.y / 64.0));
			}

			let mesh = Mesh::new(PrimitiveTopology::TriangleList)
				.with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_parts.verts)
				.with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
				.with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
				.with_indices(Some(Indices::U32(mesh_parts.indices)));

			let mesh_handle = param.meshes.add(mesh);

			let mut ecmds = cbuilder.spawn(MaterialMeshBundle {
				mesh: mesh_handle.clone(),
				material,
				transform: Transform {
					translation: Vec3::new(
						(min_raw_x as f32) * viletech::world::FSCALE,
						(min_raw_y as f32) * viletech::world::FSCALE,
						0.0,
					),
					..Default::default()
				},
				..Default::default()
			});

			ecmds.insert(EdSector(mesh_handle));
		}
	});

	info!(
		concat!(
			"Loaded level for editing: {}\n",
			"Stats:\n",
			"\tTook {}ms\n",
			"\tMaterials: {}"
		),
		marker.path(),
		start_time.elapsed().as_millis(),
		param.mtrs_std.len() - prev_mtr_count
	);

	ed.level_editor.current = Some(EditedLevel::Vanilla {
		entity: lcmds.id(),
		_marker: marker_slot,
	});

	if !ed.level_editor_open() {
		ed.panel_m = editor::Dialog::LevelEd;
	}
}

pub(crate) fn unload(ed: &mut Editor, mut param: SysParam) {
	match ed.level_editor.current {
		Some(EditedLevel::Vanilla { entity, .. }) => {
			param.cmds.entity(entity).despawn_recursive();
		}
		Some(EditedLevel::_Udmf { entity, .. }) => {
			param.cmds.entity(entity).despawn_recursive();
		}
		None => {}
	}
}
