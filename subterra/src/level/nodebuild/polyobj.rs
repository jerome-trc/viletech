//! Functions for resolving the bounding boxes of polyobjects.

use rayon::prelude::*;

use crate::{math::{Fixed32, self, UANGLE_180, UAngle}, data::dobj::LineDef};

use super::{NodeBuilder, FixedXZ, BBox, pt_side_rel, PointSideRel};

/// Step 3: polyobject bounding box resolution.
impl NodeBuilder<'_> {
	/// (GZ) Find "loops" of segs surrounding polyobject's origin.
	///
	/// Note that a polyobject's origin is not solely defined by the polyobject's
	/// anchor, but also by the polyobject itself. For the split avoidance to
	/// work properly, you must have a convex, complete loop of segs surrounding
	/// the polyobject origin. All the maps in hexen.wad have complete loops of
	/// segs around their polyobjects, but they are not all convex: The doors at
	/// the start of MAP01 and some of the pillars in MAP02 that surround the
	/// entrance to MAP06 are not convex. `Self::heuristic` uses some special
	/// weighting to make these cases work properly.
	pub(super) fn find_poly_containers(&mut self) {
		let mut loopnum = 1;

		for i in 0..self.poly.starts.len() {
			let mut bbox = BBox::default();

			if !self.get_poly_extents(self.poly.starts[i].polynum, &mut bbox) {
				continue;
			}

			let anchor = self
				.poly
				.anchors
				.par_iter()
				.find_first(|ps| ps.polynum == self.poly.starts[i].polynum);

			let Some(anchor) = anchor else { continue; };

			let mid = FixedXZ {
				x: *bbox.left() + (*bbox.right() - *bbox.left()) / 2,
				z: *bbox.bottom() + (*bbox.top() - *bbox.bottom()) / 2,
			};

			let center = FixedXZ {
				x: mid.x - anchor.x + self.poly.starts[i].x,
				z: mid.z - anchor.z + self.poly.starts[i].z,
			};

			let mut closest_dist = Fixed32::MAX;
			let mut closest_seg = 0;

			for (ii, seg) in self.segs.iter().enumerate() {
				let v1 = &self.verts[seg.v1];
				let v2 = &self.verts[seg.v2];
				let d_z = v2.z - v1.z;

				if d_z == 0 {
					continue; // (GZ) Horizontal, so skip it.
				}

				if (v1.z < center.z && v2.z < center.z) || (v1.z > center.z && v2.z > center.z) {
					continue; // (GZ) Not crossed.
				}

				let d_x = v2.x - v1.x;

				if pt_side_rel(center, v1.to_point(), FixedXZ::new(d_x, d_z)) != PointSideRel::BehindLine {
					let t = math::fxdiv::<30>(center.z - v1.z, d_z);
					let sx = v1.x + math::fxmul::<30>(d_x, t);
					let dist = sx - self.poly.starts[i].x;

					if dist < closest_dist && dist >= 0 {
						closest_dist = dist;
						closest_seg = ii;
					}
				}
			}

			if closest_dist != Fixed32::MAX {
				loopnum = self.mark_loop(closest_seg, loopnum);
			}
		}
	}

	/// (GZ) Find the bounding box for a specific polyobject.
	#[must_use]
	fn get_poly_extents(&self, polynum: u32, bbox: &mut BBox) -> bool {
		*bbox.left() = Fixed32::MAX;
		*bbox.bottom() = Fixed32::MAX;
		*bbox.right() = Fixed32::MIN;
		*bbox.top() = Fixed32::MIN;

		// (GZ) Try to find a polyobj marked with a start line.
		let start_line_pobj = self.segs.par_iter().position_first(|seg| {
			let linedef = &self.level.linedefs[seg.linedef];
			linedef.special == LineDef::POBJ_LINE_START && linedef.args[0] == (polynum as i32)
		});

		if let Some(mut slpndx) = start_line_pobj {
			let slp = &self.segs[slpndx];
			let mut vndx = slp.v1;
			// (GZ) To prevent endless loops.
			let mut seg_count = self.segs.len();

			let start = FixedXZ {
				x: self.verts[vndx].x,
				z: self.verts[vndx].z,
			};

			loop {
				self.add_seg_to_bbox(bbox, slp);
				vndx = slp.v2;
				slpndx = self.verts[vndx].segs;

				seg_count -= 1;

				if seg_count == 0 {
					break;
				}

				if slpndx >= usize::MAX {
					break;
				}

				if !(self.verts[vndx].x != start.x || self.verts[vndx].z != start.z) {
					break;
				}
			}

			return true;
		}

		let mut found = false;

		for seg in &self.segs {
			// (GZ) Try to find a polyobj marked with explicit lines.
			let linedef = &self.level.linedefs[seg.linedef];

			if linedef.special == LineDef::POBJ_LINE_EXPLICIT && linedef.args[0] == (polynum as i32)
			{
				self.add_seg_to_bbox(bbox, seg);
				found = true;
			}
		}

		found
	}

	#[must_use]
	fn mark_loop(&mut self, first_seg: usize, loop_num: usize) -> usize {
		if self.segs[first_seg].loop_num != 0 {
			return loop_num; // (GZ) Already marked.
		}

		let sec = self.segs[first_seg].sector_front;
		let mut seg = first_seg;

		loop {
			self.segs[seg].loop_num = loop_num;
			let mut best_seg = usize::MAX;
			let mut try_seg = self.verts[self.segs[seg].v2].segs;
			let mut best_ang = UAngle::MAX;
			let ang1 = self.segs[seg].angle;

			while try_seg != usize::MAX {
				if self.segs[try_seg].sector_front == sec {
					let ang2 = self.segs[seg].angle + UANGLE_180;
					let angdiff = ang2 - ang1;

					if angdiff < best_ang && angdiff > 0 {
						best_ang = angdiff;
						best_seg = try_seg;
					}
				}

				try_seg = self.segs[try_seg].next_for_vert;
			}

			seg = best_seg;

			if seg == usize::MAX || self.segs[seg].loop_num != 0 {
				break;
			}
		}

		loop_num + 1
	}
}
