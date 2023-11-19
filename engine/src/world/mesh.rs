//! Functions for building [`Mesh`]es from BSP trees.
//!
//! All code below is adapted from Cristi Cobzarenco's rust-doom.
//! For licensing information, see the repository's ATTRIB.md file.

use std::cmp::Ordering;

use bevy::prelude::*;
use data::level::{read::prelude::*, RawLevel};
use smallvec::SmallVec;

use super::FSCALE;

/// The pair of convex polygons derived from one [`SSectorRaw`].
///
/// Both have identical geometry, but one is at the height of the subsector's
/// floor level, and the other is at the height of the subsector's ceiling level.
#[derive(Debug, Clone, PartialEq)]
pub struct SubSectorPoly {
	subsector: u32,
	verts: Vec<Vec3>,
	indices: Vec<usize>,
	top_verts: u32,
	top_indices: u32,
}

impl SubSectorPoly {
	/// To which subsector does this polygon belong?
	#[must_use]
	pub fn subsector(&self) -> usize {
		self.subsector as usize
	}

	/// Returns a slice of vertices and indices, respectively.
	#[must_use]
	pub fn floor(&self) -> (&[Vec3], &[usize]) {
		(
			&self.verts[..(self.top_verts as usize)],
			&self.indices[..(self.top_indices as usize)],
		)
	}

	/// Returns a slice of vertices and indices, respectively.
	#[must_use]
	pub fn ceiling(&self) -> (&[Vec3], &[usize]) {
		(
			&self.verts[(self.top_verts as usize)..],
			&self.indices[(self.top_indices as usize)..],
		)
	}
}

pub fn subsectors_to_polygons<F: FnMut(SubSectorPoly)>(raw: RawLevel, mut callback: F) {
	let mut bsp_lines = vec![];
	recur(raw, &mut callback, &mut bsp_lines, raw.nodes.len() - 1);
}

/*

TODO:
- Faster vertex sort?
- Using SIMD to find node line intersections?
- Converting subsectors to polygons in parallel?
- Stack-safe node tree recursion?

*/

fn recur<F: FnMut(SubSectorPoly)>(
	raw: RawLevel,
	callback: &mut F,
	bsp_lines: &mut Vec<Disp>,
	node_ix: usize,
) {
	let node = &raw.nodes[node_ix];

	let seg_start_raw = node.seg_start();
	let seg_delta_raw = node.seg_delta();

	let seg_start = glam::vec2(
		-((seg_start_raw[1] as f32) * FSCALE),
		-((seg_start_raw[0] as f32) * FSCALE),
	);

	let seg_end = seg_start
		+ glam::vec2(
			-((i16::from_le(seg_delta_raw[1]) as f32) * FSCALE),
			-((i16::from_le(seg_delta_raw[0]) as f32) * FSCALE),
		);

	bsp_lines.push(Disp::new(seg_start, seg_end));

	match node.child_l() {
		BspNodeChild::SubNode(subnode_ix) => {
			recur(raw, callback, bsp_lines, subnode_ix);
		}
		BspNodeChild::SubSector(subsect_idx) => {
			if let Some(poly) = subsector_to_poly(raw, bsp_lines, subsect_idx) {
				callback(poly);
			}
		}
	}

	bsp_lines.pop();
	bsp_lines.push(Disp::new(seg_start, seg_end).inverted_halfspaces());

	match node.child_r() {
		BspNodeChild::SubNode(subnode_ix) => {
			recur(raw, callback, bsp_lines, subnode_ix);
		}
		BspNodeChild::SubSector(subsect_idx) => {
			if let Some(poly) = subsector_to_poly(raw, bsp_lines, subsect_idx) {
				callback(poly);
			}
		}
	}

	bsp_lines.pop();
}

#[must_use]
fn subsector_to_poly(
	raw: RawLevel,
	bsp_lines: &[Disp],
	subsect_ix: usize,
) -> Option<SubSectorPoly> {
	let mut mverts = vec![];

	let subsect = &raw.subsectors[subsect_ix];
	let seg0_ix = subsect.first_seg() as usize;
	let seg_count = subsect.seg_count() as usize;
	let seg0 = &raw.segs[seg0_ix + (seg_count - 1)];
	let linedef = &raw.linedefs[seg0.linedef() as usize];

	let sidedef = match seg0.direction() {
		SegDirection::Front => &raw.sidedefs[linedef.right_side() as usize],
		SegDirection::Back => &raw.sidedefs[linedef.left_side().unwrap() as usize],
	};

	let sector = &raw.sectors[sidedef.sector() as usize];

	let mut last_seg_vert = 0;

	for i in subsect.segs() {
		let seg_i = &raw.segs[i];

		let v_start_raw = &raw.vertices[seg_i.start_vertex() as usize];
		let v_end_raw = &raw.vertices[seg_i.end_vertex() as usize];

		let v_start = super::Vertex::from(*v_start_raw);
		let v_end = super::Vertex::from(*v_end_raw);

		mverts.push(glam::vec3(
			-v_start.y,
			-v_start.x,
			(sector.floor_height() as f32) * FSCALE,
		));

		mverts.push(glam::vec3(
			-v_end.y,
			-v_end.x,
			(sector.floor_height() as f32) * FSCALE,
		));

		last_seg_vert += 2;
	}

	for node_i in 0..(bsp_lines.len() - 1) {
		for node_ii in (node_i + 1)..bsp_lines.len() {
			let l1 = bsp_lines[node_i];
			let l2 = bsp_lines[node_ii];

			let Some(point) = l1.intersect_pt(l2) else {
				continue;
			};

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
						glam::vec2(mverts[vi].x, mverts[vi].y),
						glam::vec2(mverts[vi + 1].x, mverts[vi + 1].y),
					)
					.signed_distance(point)
				})
				.all(|d| d <= SEG_TOLERANCE);

			if inside_bsp && inside_segs {
				mverts.push(glam::vec3(
					point.x,
					point.y,
					(sector.floor_height() as f32) * FSCALE,
				));
			}
		}
	}

	let Some(mut verts) = points_to_poly(mverts).filter(|v| v.len() >= 3) else {
		return None;
	};

	let mut verts2d = SmallVec::<[f32; 8]>::new();

	for v in verts.iter().copied() {
		verts2d.push(v.x);
		verts2d.push(v.y);
	}

	let Ok(mut indices) = earcutr::earcut(&verts2d, &[], 2) else {
		return None;
	};

	let v_len = verts.len();

	for i in 0..v_len {
		let vert = verts[i];

		verts.push(glam::vec3(
			vert.x,
			vert.y,
			(sector.ceiling_height() as f32) * FSCALE,
		));
	}

	let i_len = indices.len();

	for i in (0..i_len).rev() {
		let vndx = indices[i];
		indices.push(vndx);
	}

	Some(SubSectorPoly {
		subsector: subsect_ix as u32,
		verts,
		indices,
		top_verts: v_len as u32,
		top_indices: i_len as u32,
	})
}

// Triangulation ///////////////////////////////////////////////////////////////

#[must_use]
fn points_to_poly(mut points: Vec<Vec3>) -> Option<Vec<Vec3>> {
	debug_assert!(
		points.len() >= 3,
		"`points_to_poly` received less than 3 points."
	);

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
			if ac.y >= 0.0 || bc.y >= 0.0 {
				return if a.y > b.y {
					Ordering::Less
				} else {
					Ordering::Greater
				};
			}

			return if b.y > a.y {
				Ordering::Less
			} else {
				Ordering::Greater
			};
		}

		if ac.xy().perp_dot(bc.xy()) < 0.0 {
			Ordering::Less
		} else {
			Ordering::Greater
		}
	});

	// Remove duplicates.
	let mut simplified = vec![];
	simplified.push(points[0]);
	let mut current_point = points[1];
	let mut area = 0.0;

	for next_point in points.iter().skip(2).copied() {
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

	simplified.push(points[points.len() - 1]);

	if simplified.len() < 3 {
		return None;
	}

	while (simplified[0] - simplified[simplified.len() - 1])
		.xy()
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

	Some(simplified)
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

// Disp ////////////////////////////////////////////////////////////////////////

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
