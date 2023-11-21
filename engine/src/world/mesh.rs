//! Functions for building [`Mesh`]es from BSP trees.
//!
//! All code below is adapted from Cristi Cobzarenco's rust-doom.
//! For licensing information, see the repository's ATTRIB.md file.

use std::cmp::Ordering;

use bevy::prelude::*;
use data::level::{read::prelude::*, RawLevel};

use super::FSCALE;

/// A triangulated convex polygon derived from a [sub-sector](SSectorRaw).
///
/// See [`triangulate`].
#[derive(Debug, Clone, PartialEq)]
pub struct SubSectorPoly {
	pub subsector: usize,
	pub verts: Vec<Vec2>,
	pub indices: Vec<usize>,
}

/// Walk the binary space partition (BSP) tree of `raw` to
///
/// This assumes that [`RawLevel::nodes`], [`RawLevel::segs`], and
/// [`RawLevel::subsectors`] are all non-empty with coherent content.
pub fn triangulate<F: FnMut(SubSectorPoly)>(raw: RawLevel, mut callback: F) {
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
	let mut points = vec![];
	let subsect = &raw.subsectors[subsect_ix];
	let mut last_seg_vert = 0;

	for i in subsect.segs() {
		let seg_i = &raw.segs[i];

		let v_start_raw = &raw.vertices[seg_i.start_vertex() as usize];
		let v_end_raw = &raw.vertices[seg_i.end_vertex() as usize];

		points.push(Vec2 {
			x: -((v_start_raw.position()[1] as f32) * FSCALE),
			y: -((v_start_raw.position()[0] as f32) * FSCALE),
		});

		points.push(Vec2 {
			x: -((v_end_raw.position()[1] as f32) * FSCALE),
			y: -((v_end_raw.position()[0] as f32) * FSCALE),
		});

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
			const SEG_TOLERANCE: f32 = 0.05;

			let inside_bsp = bsp_lines
				.iter()
				.map(|line| line.signed_distance(point))
				.all(|d| d >= -BSP_TOLERANCE);
			let inside_segs = (0..last_seg_vert)
				.step_by(2)
				.map(|vi| {
					Disp::new(
						glam::vec2(points[vi].x, points[vi].y),
						glam::vec2(points[vi + 1].x, points[vi + 1].y),
					)
					.signed_distance(point)
				})
				.all(|d| d <= SEG_TOLERANCE);

			if inside_bsp && inside_segs {
				points.push(Vec2 {
					x: point.x,
					y: point.y,
				});
			}
		}
	}

	let Some(verts) = points_to_poly(points).filter(|v| v.len() >= 3) else {
		return None;
	};

	let verts_flat = bytemuck::cast_vec::<_, f32>(verts);

	let Ok(indices) = earcutr::earcut(&verts_flat, &[], 2) else {
		return None;
	};

	Some(SubSectorPoly {
		subsector: subsect_ix,
		verts: bytemuck::cast_vec(verts_flat),
		indices,
	})
}

// Triangulation ///////////////////////////////////////////////////////////////

#[must_use]
fn points_to_poly(mut points: Vec<Vec2>) -> Option<Vec<Vec2>> {
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

		if ac.perp_dot(bc) < 0.0 {
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

		let new = next_point - current_point;
		let new_area = new.perp_dot(current_point - prev_point) * 0.5;

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

	while (simplified[0] - simplified[simplified.len() - 1]).length() < 0.0032 {
		simplified.pop();
	}

	let center = poly_center(&simplified);

	/// All polygons are "fattened" by this amount to fill in thin gaps between them.
	const POLY_BIAS: f32 = 0.64 * 3e-4;

	for point in &mut simplified {
		*point += (*point - center).normalize_or_zero() * POLY_BIAS;
		*point *= -1.0;
	}

	Some(simplified)
}

#[must_use]
fn poly_center(verts: &[Vec2]) -> Vec2 {
	let sum = verts
		.iter()
		.cloned()
		.reduce(|base, arg| base + arg)
		.unwrap();

	let center = sum / verts.len() as f32;

	// Move the center slightly so that the angles are not all equal
	// if the polygon is a perfect quadrilateral.
	Vec2 {
		x: center.x + f32::EPSILON,
		y: center.y + f32::EPSILON,
	}
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
