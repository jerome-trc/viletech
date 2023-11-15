//! Functions for assembling a level's compact datum representation into ECS form.

use std::{cmp::Ordering, collections::HashMap};

use bevy::{
	ecs::system::Insert,
	prelude::*,
	render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use glam::Vec3Swizzles;
use indexmap::IndexMap;
use parking_lot::Mutex;
use smallvec::SmallVec;
use triangulate::{formats::IndexedListFormat, ListFormat, Polygon};
use util::sparseset::SparseSet;

use crate::{
	catalog::dobj,
	gfx::TerrainMaterial,
	level::repr::{BspNodeChild, LevelDef, SegDirection, Vertex},
	sim::level::VertIndex,
	sim::{
		level::{self, Side, SideIndex, Udmf},
		line::{self, Line},
		sector::{self, Sector},
	},
};

pub(crate) fn setup(
	mut ctx: super::Context,
	base: dobj::Handle<LevelDef>,
	level: &mut ChildBuilder,
) {
	let level = Mutex::new(level);

	let mut verts = SparseSet::with_capacity(base.geom.vertdefs.len(), base.geom.vertdefs.len());

	for (i, vert) in base.geom.vertdefs.iter().enumerate() {
		verts.insert(VertIndex(i), *vert);
	}

	let (mesh, simstate) = rayon::join(
		|| build_mesh(&base, &verts),
		|| {
			let mut level = level.lock();
			spawn_children(&base, &mut level)
		},
	);

	let mesh = ctx.meshes.add(mesh);
	let level = level.into_inner();

	level.add_command(Insert {
		entity: level.parent_entity(),
		bundle: MaterialMeshBundle {
			mesh: mesh.clone(),
			material: Handle::<TerrainMaterial>::Weak(AssetId::<TerrainMaterial>::Uuid {
				uuid: AssetId::<TerrainMaterial>::DEFAULT_UUID,
			}),
			..default()
		},
	});

	level.add_command(Insert {
		entity: level.parent_entity(),
		bundle: level::Core {
			base: Some(base.clone()),
			flags: level::Flags::empty(),
			ticks_elapsed: 0,
			geom: level::Geometry {
				mesh,
				verts,
				sides: simstate.sides,
				triggers: simstate.triggers,
				num_sectors: base.geom.sectordefs.len(),
			},
		},
	});
}

struct SimState {
	sides: SparseSet<SideIndex, Side>,
	triggers: HashMap<line::Trigger, Vec<Sector>>,
}

#[must_use]
fn spawn_children(base: &dobj::Handle<LevelDef>, level: &mut ChildBuilder) -> SimState {
	let mut lines = IndexMap::with_capacity(base.geom.linedefs.len());
	let mut sectors = IndexMap::with_capacity(base.geom.sectordefs.len());
	let mut sides = SparseSet::with_capacity(base.geom.sidedefs.len(), base.geom.sidedefs.len());

	let mut sectors_by_trigger = HashMap::new();

	for linedef in &base.geom.linedefs {
		let line_id = level.spawn(()).id();

		lines.insert(
			Line(line_id),
			(
				line::Core {
					udmf_id: linedef.udmf_id,
					vert_start: VertIndex(linedef.vert_start),
					vert_end: VertIndex(linedef.vert_end),
					flags: linedef.flags,
					side_right: SideIndex(linedef.side_right),
					side_left: linedef.side_left.map(SideIndex),
				},
				linedef.special,
			),
		);
	}

	for sectordef in &base.geom.sectordefs {
		let sect_id = level.spawn(()).id();

		sectors.insert(
			Sector(sect_id),
			(sector::Core { lines: vec![] }, sectordef.special),
		);

		let trigger = line::Trigger(sectordef.trigger);

		let sect_grp = sectors_by_trigger.entry(trigger).or_insert(vec![]);
		sect_grp.push(Sector(sect_id));
	}

	for (i, sidedef) in base.geom.sidedefs.iter().enumerate() {
		sides.insert(
			SideIndex(i),
			Side {
				offset: sidedef.offset,
				sector: *sectors.get_index(sidedef.sector).unwrap().0,
				udmf: Udmf::default(),
			},
		);
	}

	for (line_id, (line, _)) in &lines {
		let side_r = &sides[line.side_right];
		let (sect, _) = sectors.get_mut(&side_r.sector).unwrap();
		sect.lines.push(*line_id);

		if let Some(side_l_idx) = line.side_left {
			let side_l = &sides[side_l_idx];
			let (sect, _) = sectors.get_mut(&side_l.sector).unwrap();
			sect.lines.push(*line_id);
		}
	}

	for (line_id, (line, _special)) in lines {
		// TODO: Add line special bundles here.
		level.add_command(Insert {
			entity: line_id.0,
			bundle: line,
		});
	}

	for (sect_id, (sect, _special)) in sectors {
		// TODO: Add sector special bundles here.
		level.add_command(Insert {
			entity: sect_id.0,
			bundle: sect,
		});
	}

	SimState {
		sides,
		triggers: sectors_by_trigger,
	}
}
