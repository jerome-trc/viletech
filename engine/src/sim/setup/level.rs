//! Functions for assembling a level's compact asset representation into ECS form.

use std::{
	cmp::Ordering,
	collections::{hash_map::RandomState, HashMap},
};

use bevy::{
	ecs::system::Insert,
	prelude::*,
	render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use glam::Vec3Swizzles;
use indexmap::IndexMap;
use smallvec::SmallVec;
use triangulate::{formats::IndexedListFormat, ListFormat, Polygon};

use crate::{
	data::asset::{self, BspNodeChild, SegDirection},
	sim::level::VertIndex,
	sim::{
		level::{self, Side, SideIndex, Udmf, Vertex},
		line::{self, Line},
		sector::{self, Sector},
	},
	sparse::SparseSet,
};

pub(crate) fn setup(
	mut ctx: super::Context,
	base: asset::Handle<asset::Level>,
	level: &mut ChildBuilder,
) {
	let mut verts = SparseSet::with_capacity(base.vertices.len(), base.vertices.len());
	let mut sides = SparseSet::with_capacity(base.sidedefs.len(), base.sidedefs.len());

	for (i, vert) in base.vertices.iter().enumerate() {
		verts.insert(VertIndex(i), vert.clone());
	}

	let mut lines = IndexMap::with_capacity(base.linedefs.len());
	let mut sectors = IndexMap::with_capacity(base.sectors.len());

	let mut sectors_by_trigger: HashMap<_, _, RandomState> = HashMap::default();

	for linedef in &base.linedefs {
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

	for sectordef in &base.sectors {
		let sect_id = level.spawn(()).id();

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

	let mesh_parts = walk_nodes(&base, &verts);
	let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

	mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_parts.verts);
	mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_parts.normals);
	mesh.set_indices(Some(Indices::U32(mesh_parts.indices)));

	let mesh = ctx.meshes.add(mesh);

	level.add_command(Insert {
		entity: level.parent_entity(),
		bundle: PbrBundle {
			mesh: mesh.clone(),
			material: ctx.materials.add(StandardMaterial {
				base_color: Color::rgb(1.0, 1.0, 1.0),
				..default()
			}),
			..default()
		},
	});

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

	level.add_command(Insert {
		entity: level.parent_entity(),
		bundle: level::Core {
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
		},
	});
}

// Node walking and subsector-to-polygon conversion ////////////////////////////

/*

All code in this part of this file is adapted from Cristi Cobzarenco's rust-doom.
For licensing information, see the repository's ATTRIB.md file.

TODO:
- Adapt codebase to Bevy's Y-up coordinate system.
- Faster vertex sort?
- Faster triangulation (e.g. Earcut)?
- Using SIMD to find node line intersections?
- Converting subsectors to polygons in parallel?
- Stack-safe node tree recursion?

*/

#[derive(Debug)]
struct LevelMesh {
	verts: Vec<Vec3>,
	indices: Vec<u32>,
	normals: Vec<Vec3>,
}

#[must_use]
fn walk_nodes(
	base: &asset::Handle<asset::Level>,
	verts: &SparseSet<VertIndex, Vertex>,
) -> LevelMesh {
	let mut ret = LevelMesh {
		verts: vec![],
		indices: vec![],
		normals: vec![],
	};

	let mut bsp_lines = vec![];

	recur(base, verts, &mut ret, &mut bsp_lines, base.nodes.len() - 1);

	ret
}

fn recur(
	base: &asset::Handle<asset::Level>,
	verts: &SparseSet<VertIndex, Vertex>,
	mesh: &mut LevelMesh,
	bsp_lines: &mut Vec<Disp>,
	node_idx: usize,
) {
	let node = &base.nodes[node_idx];

	bsp_lines.push(Disp::new(node.seg_start, node.seg_end));

	fn add_poly(mut sspoly: SSectorPoly, mesh: &mut LevelMesh) {
		for idx in sspoly.indices.drain(sspoly.top_indices..) {
			mesh.indices.push((idx + mesh.verts.len()) as u32);
		}

		for vert in sspoly.verts.drain(sspoly.top_verts..) {
			mesh.verts.push(-vert);
			mesh.normals.push(Vec3::NEG_Y);
		}

		for idx in sspoly.indices {
			mesh.indices.push((idx + mesh.verts.len()) as u32);
		}

		for vert in sspoly.verts {
			mesh.verts.push(-vert);
			mesh.normals.push(Vec3::Y);
		}
	}

	match node.child_l {
		BspNodeChild::SubNode(subnode_idx) => {
			recur(base, verts, mesh, bsp_lines, subnode_idx);
		}
		BspNodeChild::SubSector(subsect_idx) => {
			add_poly(subsector_to_poly(base, verts, bsp_lines, subsect_idx), mesh);
		}
	}

	bsp_lines.pop();

	bsp_lines.push(Disp::new(node.seg_start, node.seg_end).inverted_halfspaces());

	match node.child_r {
		BspNodeChild::SubNode(subnode_idx) => {
			recur(base, verts, mesh, bsp_lines, subnode_idx);
		}
		BspNodeChild::SubSector(subsect_idx) => {
			add_poly(subsector_to_poly(base, verts, bsp_lines, subsect_idx), mesh);
		}
	}

	bsp_lines.pop();
}

#[derive(Debug)]
struct SSectorPoly {
	verts: SmallVec<[Vec3; 8]>,
	indices: Vec<usize>,
	top_verts: usize,
	top_indices: usize,
}

#[must_use]
fn subsector_to_poly(
	base: &asset::Handle<asset::Level>,
	map_verts: &SparseSet<VertIndex, Vertex>,
	bsp_lines: &[Disp],
	subsect_idx: usize,
) -> SSectorPoly {
	let mut verts = SmallVec::<[Vec3; 4]>::new();
	let mut indices = Vec::<usize>::new();

	let subsect = &base.subsectors[subsect_idx];
	let seg0 = &base.segs[subsect.seg0 + (subsect.seg_count - 1)];
	let linedef = &base.linedefs[seg0.linedef];

	let sidedef = match seg0.direction {
		SegDirection::Front => &base.sidedefs[linedef.side_right],
		SegDirection::Back => &base.sidedefs[linedef.side_left.unwrap()],
	};

	let sector = &base.sectors[sidedef.sector];

	let mut last_seg_vert = 0;

	for i in subsect.seg0..(subsect.seg0 + subsect.seg_count) {
		let seg_i = &base.segs[i];

		let v_start = &map_verts[VertIndex(seg_i.vert_start)];
		let v_end = &map_verts[VertIndex(seg_i.vert_end)];

		verts.push(glam::vec3(-v_start.z, sector.height_floor, -v_start.x));
		verts.push(glam::vec3(-v_end.z, sector.height_floor, -v_end.x));

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
						glam::vec2(verts[vi].x, verts[vi].z),
						glam::vec2(verts[vi + 1].x, verts[vi + 1].z),
					)
					.signed_distance(point)
				})
				.all(|d| d <= SEG_TOLERANCE);

			if inside_bsp && inside_segs {
				verts.push(glam::vec3(point.x, sector.height_floor, point.y));
			}
		}
	}

	let mut verts = points_to_poly(verts);

	let format = IndexedListFormat::new(&mut indices).into_fan_format();

	// SAFETY: `NodeVert` is `repr(transparent)` over `glam::vec3`.
	unsafe {
		let v = std::mem::transmute::<_, &SmallVec<[TmutVert; 8]>>(&verts);

		if let Err(err) = v.triangulate(format) {
			warn!("Failed to triangulate subsector {subsect_idx}: {err}");
		}
	}

	let v_len = verts.len();

	for i in 0..v_len {
		let vert = verts[i];
		verts.push(glam::vec3(vert.x, sector.height_ceil, vert.z));
	}

	let i_len = indices.len();

	for i in (0..i_len).rev() {
		let vndx = indices[i];
		indices.push(vndx);
	}

	SSectorPoly {
		verts,
		indices,
		top_verts: v_len,
		top_indices: i_len,
	}
}

#[must_use]
fn points_to_poly(mut points: SmallVec<[Vec3; 4]>) -> SmallVec<[Vec3; 8]> {
	// Sort points in polygonal CCW order around their center.
	let center = poly_center(&points);

	points.sort_unstable_by(|a, b| {
		let ac = *a - center;
		let bc = *b - center;

		if ac.x >= 0.0 && bc.x < 0.0 {
			return Ordering::Less;
		}

		if ac.x < 0.0 && bc.x >= 0.0 {
			return Ordering::Greater;
		}

		if ac.x == 0.0 && bc.x == 0.0 {
			if ac.z >= 0.0 || bc.z >= 0.0 {
				return if a.z > b.z {
					Ordering::Less
				} else {
					Ordering::Greater
				};
			}

			return if b.z > a.z {
				Ordering::Less
			} else {
				Ordering::Greater
			};
		}

		if ac.xz().perp_dot(bc.xz()) < 0.0 {
			Ordering::Less
		} else {
			Ordering::Greater
		}
	});

	// Remove duplicates.
	let mut simplified = SmallVec::<[Vec3; 8]>::new();
	simplified.push(points[0]);
	let mut current_point = points[1];
	let mut area = 0.0;

	for i_point in 2..points.len() {
		let next_point = points[i_point];
		let prev_point = simplified[simplified.len() - 1];

		let new = (next_point - current_point).xz();
		let new_area = new.perp_dot((current_point - prev_point).xz()) * 0.5;

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

	simplified.push(points[points.len() - 1]);

	debug_assert!(
		simplified.len() >= 3,
		"Degenerate polygon created during level init."
	);

	while (simplified[0] - simplified[simplified.len() - 1])
		.xz()
		.length()
		< 0.0032
	{
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
	glam::vec3(center.x + f32::EPSILON, center.y, center.z + f32::EPSILON)
}

/// Fake type for impl trait coherence.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
struct TmutVert(Vec3);

impl triangulate::Vertex for TmutVert {
	type Coordinate = f32;

	fn x(&self) -> Self::Coordinate {
		self.0.x
	}

	#[allow(clippy::misnamed_getters)]
	fn y(&self) -> Self::Coordinate {
		self.0.z
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
