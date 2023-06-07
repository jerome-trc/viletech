//! Routines for building segs from sidedefs.

use crate::{math::{UAngle, Fixed32, point_to_angle}};

use super::{NodeBuilder, Seg, BBox};

/// Step 2: building segs from sidedefs.
impl NodeBuilder<'_> {
	pub(super) fn create_segs_from_sides(&mut self) {
		let mut ii;

		for i in 0..self.level.linedefs.len() {
			self.create_seg(i, false);

			if self.level.linedefs[i].side_left.is_some() {
				ii = self.create_seg(i, true);
				self.segs[ii - 1].partner = ii;
				self.segs[ii].partner = ii - 1;
			}
		}
	}

	/// Returns the index of the new seg in `Self::segs`.
	fn create_seg(&mut self, line: usize, back_side: bool) -> usize {
		let mut seg = Seg {
			v1: usize::MAX,
			v2: usize::MAX,
			sidedef: usize::MAX,
			linedef: usize::MAX,
			sector_front: usize::MAX,
			sector_back: usize::MAX,
			next: usize::MAX,
			next_for_vert: usize::MAX,
			next_for_vert2: usize::MAX,
			loop_num: 0,
			partner: usize::MAX,
			stored_seg: usize::MAX,
			angle: UAngle::MAX,
			offset: Fixed32::ZERO,
			plane_num: usize::MAX,
			plane_front: false,
			to_check_next: usize::MAX,
		};

		if back_side {
			seg.v1 = self.level.linedefs[line].vert_start;
			seg.v2 = self.level.linedefs[line].vert_end;
		} else {
			seg.v2 = self.level.linedefs[line].vert_start;
			seg.v1 = self.level.linedefs[line].vert_end;
		}

		seg.linedef = line;

		seg.sidedef = if back_side {
			self.level.linedefs[line].side_right
		} else {
			self.level.linedefs[line].side_left.unwrap_or(usize::MAX)
		};

		let backside = if !back_side {
			self.level.linedefs[line].side_right
		} else {
			self.level.linedefs[line].side_left.unwrap_or(usize::MAX)
		};

		seg.sector_front = self.level.sidedefs[seg.sidedef].sector;
		seg.sector_back = if backside != usize::MAX {
			self.level.sidedefs[backside].sector
		} else {
			usize::MAX
		};

		seg.next_for_vert = self.verts[seg.v1].segs;
		seg.next_for_vert2 = self.verts[seg.v2].segs2;

		seg.angle = point_to_angle(
			self.verts[seg.v2].x - self.verts[seg.v1].x,
			self.verts[seg.v2].z - self.verts[seg.v1].z,
		);

		let ret = self.segs.len();

		self.verts[seg.v1].segs = ret;
		self.verts[seg.v2].segs2 = ret;
		self.segs.push(seg);

		ret
	}

	pub(super) fn add_seg_to_bbox(&self, bbox: &mut BBox, seg: &Seg) {
		let v1 = &self.verts[seg.v1];
		let v2 = &self.verts[seg.v2];

		if v1.x < *bbox.left() {
			*bbox.left() = v1.x;
		}

		if v1.x > *bbox.right() {
			*bbox.right() = v1.x;
		}

		if v1.z < *bbox.bottom() {
			*bbox.bottom() = v1.z;
		}

		if v1.z > *bbox.top() {
			*bbox.top() = v1.z;
		}

		if v2.x < *bbox.left() {
			*bbox.left() = v2.x;
		}

		if v2.x > *bbox.right() {
			*bbox.right() = v2.x;
		}

		if v2.z < *bbox.bottom() {
			*bbox.bottom() = v2.z;
		}

		if v2.z > *bbox.top() {
			*bbox.top() = v2.z;
		}
	}
}
