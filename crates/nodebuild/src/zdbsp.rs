
use util::fixed::I16F16;

use super::read::BspNodeChild;

/// Two-axis 32-bit fixed-point vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FixedXY {
	x: I16F16,
	y: I16F16,
}

/// 32-bit fixed-point bounding box.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct BBox([I16F16; 4]);

impl BBox {
	#[must_use]
	fn bottom(&mut self) -> &mut I16F16 {
		&mut self.0[1]
	}

	#[must_use]
	fn left(&mut self) -> &mut I16F16 {
		&mut self.0[2]
	}

	#[must_use]
	fn right(&mut self) -> &mut I16F16 {
		&mut self.0[3]
	}

	#[must_use]
	fn top(&mut self) -> &mut I16F16 {
		&mut self.0[0]
	}
}

#[derive(Debug)]
struct BspNode {
	x: I16F16,
	y: I16F16,
	dx: I16F16,
	dy: I16F16,
	bboxes: [BBox; 2],
	child_r: BspNodeChild,
	child_l: BspNodeChild,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VertexIx(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SideIx(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LineIx(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SectorIx(u32);

/// "Point-side relationship".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
enum PointSideRel {
	AheadOfLine,
	OnLine,
	BehindLine,
}

impl PointSideRel {
	#[must_use]
	fn resolve(point: FixedXY, start: FixedXY, delta: FixedXY) -> Self {
		let d_dx = f64::from(delta.x);
		let d_dy = f64::from(delta.y);
		let d_x = f64::from(point.x);
		let d_y = f64::from(point.y);
		let d_x1 = f64::from(start.x);
		let d_y1 = f64::from(start.x);

		// (GZ) For most cases, a simple dot product is enough.
		let s_num = (d_y1 - d_y) * d_dx - (d_x1 - d_x) * d_dy;

		// i.e. 4 << 32.
		if s_num.abs() < 17179869184.0 {
			// (GZ) Either the point is very near the line, or the segment defining
			// the line is very short: do a more expensive test to determine just how
			// far the point is from the line.
			let l = d_dx * d_dx + d_dy * d_dy;
			let dist = s_num * s_num / l;

			if dist < (SIDE_EPSILON * SIDE_EPSILON) {
				return Self::OnLine;
			}
		}

		if s_num > 0.0 {
			Self::AheadOfLine
		} else {
			Self::BehindLine
		}
	}
}

/// (GZ) Points within this distance of a line will be considered on the line.
const SIDE_EPSILON: f64 = 6.5536;
