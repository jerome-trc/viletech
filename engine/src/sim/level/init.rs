//! Functions for assembling a level's compact asset representation into ECS form.

use std::{
	cmp::Ordering,
	collections::{hash_map::RandomState, HashMap},
};

use bevy::{
	ecs::system::EntityCommands,
	prelude::*,
	render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use glam::Vec3Swizzles;
use indexmap::IndexMap;
use smallvec::SmallVec;
use triangulate::{formats::IndexedListFormat, ListFormat, Polygon};

use crate::{
	data::asset::{self, BspNodeChild, LevelFormat, UdmfNamespace},
	sim::level,
	sim::ActiveMarker,
	sparse::SparseSet,
	BaseGame,
};

use super::{
	line::{self, Line},
	sector::{self, Sector},
	Side, SideIndex, Udmf, VertIndex, Vertex,
};

pub fn init(
	mut cmds: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	base: asset::Handle<asset::Level>,
	active: bool,
) {
	let level_id = if active {
		cmds.spawn(ActiveMarker)
	} else {
		cmds.spawn(())
	}
	.id();

	let mut verts = SparseSet::with_capacity(base.vertices.len(), base.vertices.len());
	let mut sides = SparseSet::with_capacity(base.sidedefs.len(), base.sidedefs.len());

	for (i, vert) in base.vertices.iter().enumerate() {
		verts.insert(VertIndex(i), vert.clone());
	}

	let mut lines = IndexMap::with_capacity(base.linedefs.len());
	let mut sectors = IndexMap::with_capacity(base.sectors.len());

	let mut sectors_by_trigger: HashMap<_, _, RandomState> = HashMap::default();

	for linedef in &base.linedefs {
		let line_id = cmds.spawn(()).id();

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

	for sectordef in &base.sectors {
		let sect_id = cmds.spawn(()).id();
		sectors.insert(
			Sector(sect_id),
			(sector::Core { lines: vec![] }, sectordef.special),
		);
		let trigger = line::Trigger(sectordef.trigger);

		let sect_grp = sectors_by_trigger.entry(trigger).or_insert(vec![]);
		sect_grp.push(Sector(sect_id));
	}

	for (i, sidedef) in base.sidedefs.iter().enumerate() {
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

	let (mesh_verts, indices) = walk_nodes(&base, &verts);

	let mesh_verts_len = mesh_verts.len();
	let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

	mesh.set_indices(Some(Indices::U32(indices)));

	mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_verts);

	mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, {
		let mut normals = vec![];
		normals.resize(mesh_verts_len, Vec3::Z);
		normals
	});

	let mesh = meshes.add(mesh);

	cmds.get_entity(level_id).unwrap().insert(PbrBundle {
		mesh: mesh.clone(),
		material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
		..default()
	});

	for (line_id, (line, _special)) in lines {
		let mut ent = cmds.get_entity(line_id.0).unwrap();
		ent.insert(line);
		// TODO: Add line special bundles here.
		cmds.get_entity(level_id).unwrap().add_child(line_id.0);
	}

	for (sect_id, (sect, _special)) in sectors {
		let mut ent = cmds.get_entity(sect_id.0).unwrap();
		ent.insert(sect);
		// TODO: Add sector special bundles here.
		cmds.get_entity(level_id).unwrap().add_child(sect_id.0);
	}

	cmds.get_entity(level_id).unwrap().insert(level::Core {
		base: Some(base.clone()),
		flags: level::Flags::empty(),
		ticks_elapsed: 0,
		geom: level::Geometry {
			mesh,
			verts,
			sides,
			triggers: sectors_by_trigger,
			num_sectors: base.sectors.len(),
		},
	});
}

// Node walking and subsector-to-polygon conversion ////////////////////////////

/*

All code in this part of this file is courtesy of Cristi Cobzarenco's rust-doom.
For licensing information, see the repository's ATTRIB.md file.

TODO:
- Adapt codebase to Bevy's Y-up coordinate system.
- Faster vertex sort?
- Faster triangulation (e.g. Earcut)?
- Using SIMD to find node line intersections?
- Converting subsectors to polygons in parallel?
- Stack-safe node tree recursion?

*/

#[must_use]
fn walk_nodes(
	base: &asset::Handle<asset::Level>,
	verts: &SparseSet<VertIndex, Vertex>,
) -> (Vec<Vec3>, Vec<u32>) {
	let mut mesh_verts = vec![];
	let mut indices = vec![];
	let mut bsp_lines = vec![];

	recur(
		base,
		verts,
		&mut bsp_lines,
		&mut mesh_verts,
		&mut indices,
		base.nodes.len() - 1,
	);

	(mesh_verts, indices)
}

fn recur(
	base: &asset::Handle<asset::Level>,
	verts: &SparseSet<VertIndex, Vertex>,
	bsp_lines: &mut Vec<Disp>,
	mesh_verts: &mut Vec<Vec3>,
	indices: &mut Vec<u32>,
	node_idx: usize,
) {
	let node = &base.nodes[node_idx];

	bsp_lines.push(Disp::new(node.seg_start, node.seg_end));

	match node.child_l {
		BspNodeChild::SubNode(subnode_idx) => {
			recur(base, verts, bsp_lines, mesh_verts, indices, subnode_idx);
		}
		BspNodeChild::SubSector(subsect_idx) => {
			let (poly_verts, poly_indices) = subsector_to_poly(base, verts, bsp_lines, subsect_idx);

			for idx in poly_indices {
				indices.push((idx + mesh_verts.len()) as u32);
			}

			for vert in poly_verts {
				mesh_verts.push(vert);
			}
		}
	}

	bsp_lines.pop();

	bsp_lines.push(Disp::new(node.seg_start, node.seg_end).inverted_halfspaces());

	match node.child_r {
		BspNodeChild::SubNode(subnode_idx) => {
			recur(base, verts, bsp_lines, mesh_verts, indices, subnode_idx);
		}
		BspNodeChild::SubSector(subsect_idx) => {
			let (poly_verts, poly_indices) = subsector_to_poly(base, verts, bsp_lines, subsect_idx);

			for idx in poly_indices {
				indices.push((idx + mesh_verts.len()) as u32);
			}

			for vert in poly_verts {
				mesh_verts.push(vert);
			}
		}
	}

	bsp_lines.pop();
}

#[must_use]
fn subsector_to_poly(
	base: &asset::Handle<asset::Level>,
	map_verts: &SparseSet<VertIndex, Vertex>,
	bsp_lines: &[Disp],
	subsect_idx: usize,
) -> (SmallVec<[Vec3; 4]>, Vec<usize>) {
	let mut verts = SmallVec::<[Vec3; 4]>::new();
	let mut indices = Vec::<usize>::new();

	let subsect = &base.subsectors[subsect_idx];
	let seg0 = &base.segs[subsect.seg0];
	let linedef = &base.linedefs[seg0.linedef];
	let side = &base.sidedefs[linedef.side_right];
	let _sector = &base.sectors[side.sector];

	let mut last_seg_vert = 0;

	for i in subsect.seg0..(subsect.seg0 + subsect.seg_count) {
		let s = &base.segs[i];

		let v_start = &map_verts[VertIndex(s.vert_start)];
		let v_end = &map_verts[VertIndex(s.vert_end)];

		verts.push(glam::vec3(-v_start.y, -v_start.x, v_start.z));
		verts.push(glam::vec3(-v_end.y, -v_end.x, v_end.z));

		last_seg_vert += 2;
	}

	for node_i in 0..(bsp_lines.len() - 1) {
		for node_ii in (node_i + 1)..bsp_lines.len() {
			let l1 = bsp_lines[node_i];
			let l2 = bsp_lines[node_ii];

			let Some(point) = l1.intersect_pt(l2) else { continue; };

			const BSP_TOLERANCE: f32 = 1e-3;
			const SEG_TOLERANCE: f32 = 0.1;

			let inside_bsp = bsp_lines
				.iter()
				.map(|line| line.signed_distance(point))
				.all(|d| d >= -BSP_TOLERANCE);
			let inside_segs = (0..last_seg_vert)
				.step_by(2)
				.map(|vi| {
					Disp::new(
						glam::vec2(verts[vi].x, verts[vi].y),
						glam::vec2(verts[vi + 1].x, verts[vi + 1].y),
					)
					.signed_distance(point)
				})
				.all(|d| d <= SEG_TOLERANCE);

			if inside_bsp && inside_segs {
				verts.push(glam::vec3(point.x, point.y, verts[0].z));
			}
		}
	}

	let verts = points_to_poly(verts);

	let format = IndexedListFormat::new(&mut indices).into_fan_format();

	// SAFETY: `NodeVert` is `repr(transparent)` over `glam::vec3`.
	unsafe {
		let v = std::mem::transmute::<_, &SmallVec<[NodeVert; 4]>>(&verts);

		if let Err(err) = v.triangulate(format) {
			warn!("Failed to triangulate subsector {subsect_idx}: {err}");
		}
	}

	(verts, indices)
}

#[must_use]
fn points_to_poly(mut points: SmallVec<[Vec3; 4]>) -> SmallVec<[Vec3; 4]> {
	// Sort points in polygonal CCW order around their center.
	let center = poly_center(&points);

	points.sort_unstable_by(|a, b| {
		let ac = (*a - center).xy();
		let bc = (*b - center).xy();

		if ac[0] >= 0.0 && bc[0] < 0.0 {
			return Ordering::Less;
		}
		if ac[0] < 0.0 && bc[0] >= 0.0 {
			return Ordering::Greater;
		}
		if ac[0] == 0.0 && bc[0] == 0.0 {
			if ac[1] >= 0.0 || bc[1] >= 0.0 {
				return if a[1] > b[1] {
					Ordering::Less
				} else {
					Ordering::Greater
				};
			}
			return if b[1] > a[1] {
				Ordering::Less
			} else {
				Ordering::Greater
			};
		}

		if ac.perp_dot(bc) < 0.0 {
			Ordering::Less
		} else {
			Ordering::Greater
		}
	});

	// Remove duplicates.
	let mut simplified = SmallVec::<[Vec3; 4]>::new();
	simplified.push((*points)[0]);
	let mut current_point = (*points)[1];
	let mut area = 0.0;

	for i_point in 2..points.len() {
		let next_point = (*points)[i_point];
		let prev_point = simplified[simplified.len() - 1];

		let new = (next_point - current_point).xy();
		let new_area = new.perp_dot((current_point - prev_point).xy()) * 0.5;

		if new_area >= 0.0 {
			if area + new_area > 1.024e-5 {
				area = 0.0;
				simplified.push(current_point);
			} else {
				area += new_area;
			}
		}

		current_point = next_point;
	}

	simplified.push((*points)[points.len() - 1]);

	debug_assert!(simplified.len() >= 3);

	while (simplified[0] - simplified[simplified.len() - 1]).length() < 0.0032 {
		simplified.pop();
	}

	let center = poly_center(&simplified);

	/// All polygons are "fattened" by this amount to fill in thin gaps between them.
	const POLY_BIAS: f32 = 0.64 * 3e-4;

	for point in &mut simplified {
		*point += (*point - center).normalize_or_zero() * POLY_BIAS;
	}

	simplified
}

#[must_use]
fn poly_center(verts: &[Vec3]) -> Vec3 {
	let sum = verts
		.iter()
		.cloned()
		.reduce(|base, arg| base + arg)
		.unwrap();

	let center = sum / verts.len() as f32;

	// Move the center slightly so that the angles are not all equal
	// if the polygon is a perfect quadrilateral.
	glam::vec3(center.x + f32::EPSILON, center.y + f32::EPSILON, center.z)
}

/// Fake type for impl trait coherence.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
struct NodeVert(Vec3);

impl triangulate::Vertex for NodeVert {
	type Coordinate = f32;

	fn x(&self) -> Self::Coordinate {
		self.0.x
	}

	fn y(&self) -> Self::Coordinate {
		self.0.y
	}
}

/// "Displacement line".
#[derive(Debug, Clone, Copy, PartialEq)]
struct Disp {
	origin: Vec2,
	displace: Vec2,
}

impl Disp {
	#[must_use]
	fn new(start: Vec2, end: Vec2) -> Self {
		let displace = end - start;
		let length = displace.length();

		if length.abs() >= 1e-16 {
			Self {
				origin: start,
				displace: displace / length,
			}
		} else {
			Self {
				origin: start,
				displace: Vec2::ZERO,
			}
		}
	}

	#[must_use]
	fn inverted_halfspaces(self) -> Self {
		Disp {
			origin: self.origin,
			displace: -self.displace,
		}
	}

	#[must_use]
	fn signed_distance(self, to: Vec2) -> f32 {
		to.perp_dot(self.displace) + self.displace.perp_dot(self.origin)
	}

	#[must_use]
	fn intersect_offs(self, other: Self) -> Option<f32> {
		let denom = self.displace.perp_dot(other.displace);

		if denom.abs() < 1e-16 {
			None
		} else {
			Some((other.origin - self.origin).perp_dot(other.displace) / denom)
		}
	}

	#[must_use]
	fn intersect_pt(self, other: Self) -> Option<Vec2> {
		self.intersect_offs(other)
			.map(|offs| self.origin + self.displace * offs)
	}
}

// Line specials ///////////////////////////////////////////////////////////////

fn _line_special_bundle(mut cmds: EntityCommands, format: LevelFormat, num: u16) {
	match format {
		LevelFormat::Doom => match num {
			0 => {}
			1 => {
				cmds.insert(line::Door {
					stay_time: 35 * 4,
					stay_timer: 0,
					one_off: false,
					monster_usable: true,
					remote: false,
					speed: line::Door::SPEED_NORMAL,
					lock: None,
				});
			}
			other => unimplemented!("Doom line special {other} is unimplemented."),
		},
		LevelFormat::Hexen => todo!(),
		LevelFormat::Udmf(namespace) => match namespace {
			UdmfNamespace::Doom => todo!(),
			other => unimplemented!("UDMF namespace `{other:#?}` is not yet supported."),
		},
	}
}

// Sector specials /////////////////////////////////////////////////////////////

fn _sector_special_bundle(cmds: EntityCommands, game: BaseGame, format: LevelFormat, num: u16) {
	match game {
		BaseGame::Doom => match format {
			LevelFormat::Doom => _sector_special_bundle_boom(cmds, num),
			LevelFormat::Udmf(UdmfNamespace::ZDoom) => _sector_special_bundle_zdoom(cmds, num),
			_ => unimplemented!("Unsupported configuration: {game:#?}/{format:#?}"),
		},
		BaseGame::Hexen => {
			_sector_special_bundle_zdoom(cmds, num);
		}
		BaseGame::Heretic => {
			_sector_special_bundle_heretic(cmds, num);
		}
		BaseGame::Strife => {
			_sector_special_bundle_strife(cmds, num);
		}
		BaseGame::ChexQuest => {
			// TODO: Not sure yet.
		}
	}
}

fn _sector_special_bundle_boom(mut cmds: EntityCommands, num: u16) {
	if (num & 96) != 0 {
		cmds.insert(sector::Damaging {
			damage: 20,
			interval: 35,
			leak_chance: 6,
		});
	} else if (num & 64) != 0 {
		cmds.insert(sector::Damaging {
			damage: 10,
			interval: 35,
			leak_chance: 0,
		});
	} else if (num & 32) != 0 {
		cmds.insert(sector::Damaging {
			damage: 5,
			interval: 35,
			leak_chance: 0,
		});
	}

	if (num & 128) != 0 {
		cmds.insert(sector::Secret);
	}

	if (num & 256) != 0 {
		unimplemented!("Boom friction effects are unimplemented.");
	}

	if (num & 512) != 0 {
		unimplemented!("Boom conveyor effects are unimplemented.");
	}

	match num {
		9 => {
			cmds.insert(sector::Secret);
		}
		10 => {
			cmds.insert(sector::CloseAfter { ticks: 35 * 30 });
		}
		11 => {
			cmds.insert(sector::Ending { threshold: 11 });

			cmds.insert(sector::Damaging {
				damage: 20,
				interval: 35,
				leak_chance: 6, // Q: Suit leak on ending damage floors?
			});
		}
		14 => {
			cmds.insert(sector::OpenAfter { ticks: 35 * 300 });
		}
		16 => {
			cmds.insert(sector::Damaging {
				damage: 20,
				interval: 35,
				leak_chance: 16,
			});
		}
		other => unimplemented!("Boom sector special {other} is unimplemented."),
	}
}

fn _sector_special_bundle_heretic(mut _cmds: EntityCommands, _num: u16) {
	unimplemented!("Heretic sector specials are unimplemented.")
}

fn _sector_special_bundle_strife(mut _cmds: EntityCommands, _num: u16) {
	unimplemented!("Strife sector specials are unimplemented.")
}

fn _sector_special_bundle_zdoom(mut cmds: EntityCommands, num: u16) {
	match num {
		115 => {
			// Instant death.
			cmds.insert(sector::Damaging {
				damage: 999,
				interval: 1,
				leak_chance: u8::MAX,
			});
		}
		196 => {
			cmds.insert(sector::Healing {
				interval: 32,
				amount: 1,
			});
		}
		other => unimplemented!("ZDoom sector special {other} is unimplemented."),
	}
}
