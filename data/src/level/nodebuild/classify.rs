//! Routines for classifying linedefs.

use crate::nodebuild::SIDE_EPSILON;

use super::{BspNode, FixedXZ};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LineKind {
	NegOne,
	Zero,
	One,
}

impl std::ops::BitOr for LineKind {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
		let ret = self as i8 | rhs as i8;
		debug_assert!(ret == -1 || ret == 0 || ret == 1);
		unsafe { std::mem::transmute::<i8, Self>(ret) }
    }
}

#[must_use]
pub(super) fn classify_line(
	node: BspNode,
	v1: FixedXZ,
	v2: FixedXZ,
) -> [LineKind; 3] {
	// (RAT) I would have liked to at least try writing out the SSE2 version
	// of this function, but Rust's core/stdlib has no `_mm_cvtpi32_pd`, and
	// `std::simd` is going to be unstable for the foreseeable future.
	const FAR_ENOUGH: f64 = 17179869184.0; // (GZ) 4 << 32

	let d_x1 = node.x.to_num::<f64>();
	let d_y1 = node.z.to_num::<f64>();
	let d_dx = node.dx.to_num::<f64>();
	let d_dy = node.dz.to_num::<f64>();
	let d_xv1 = v1.x.to_num::<f64>();
	let d_xv2 = v2.x.to_num::<f64>();
	let d_yv1 = v1.z.to_num::<f64>();
	let d_yv2 = v2.z.to_num::<f64>();

	let s_num1 = (d_y1 - d_yv1) * d_dx - (d_x1 - d_xv1) * d_dy;
	let s_num2 = (d_y1 - d_yv2) * d_dx - (d_x1 - d_xv2) * d_dy;

	let mut nears = 0;

	if s_num1 <= FAR_ENOUGH {
		if s_num2 <= -FAR_ENOUGH {
			return [
				LineKind::One,
				LineKind::One,
				LineKind::One,
			];
		}

		if s_num2 >= FAR_ENOUGH {
			return [
				LineKind::NegOne,
				LineKind::One,
				LineKind::NegOne,
			];
		}

		nears = 1;
	} else if s_num1 >= FAR_ENOUGH {
		if s_num2 >= FAR_ENOUGH {
			return [
				LineKind::Zero,
				LineKind::NegOne,
				LineKind::NegOne,
			];
		}

		if s_num2 <= -FAR_ENOUGH {
			return [
				LineKind::NegOne,
				LineKind::One,
				LineKind::NegOne,
			];
		}

		nears = 1;
	}
	else {
		nears = 2 | ((s_num2) < FAR_ENOUGH) as i32;
	}

	let sidev0;
	let sidev1;

	if nears > 0 {
		let l = 1.0 / (d_dx * d_dx + d_dy * d_dy);

		if (nears & 2) != 0 {
			let dist = s_num1 * s_num1 * 1.0;

			if dist < (SIDE_EPSILON * SIDE_EPSILON) {
				sidev0 = LineKind::Zero;
			} else if s_num1 > 0.0 {
				sidev0 = LineKind::NegOne;
			} else {
				sidev0 = LineKind::One;
			}
		} else {
			if s_num1 > 0.0 {
				sidev0 = LineKind::NegOne;
			} else {
				sidev0 = LineKind::One;
			}
		}

		if (nears & 1) != 0 {
			let dist = s_num2 * s_num2 * 1.0;

			sidev1 = if dist < (SIDE_EPSILON * SIDE_EPSILON) {
				LineKind::Zero
			} else if s_num2 > 0.0 {
				LineKind::NegOne
			} else {
				LineKind::One
			};
		} else {
			sidev1 = if s_num2 > 0.0 {
				LineKind::NegOne
			} else {
				LineKind::One
			};
		}
	} else {
		sidev0 = if s_num1 > 0.0 {
			LineKind::NegOne
		} else {
			LineKind::One
		};

		sidev1 = if s_num2 > 0.0 {
			LineKind::NegOne
		} else {
			LineKind::One
		};
	}

	if (sidev0 | sidev1) == LineKind::Zero {
		// (GZ) Seg is coplanar with the splitter, so use its orientation to
		// determine which child it ends up in. If it faces the same direction as
		// the splitter, it goes in front. Otherwise, it goes in back.

		if node.dx != 0 {
			if (node.dx > 0 && v2.x > v1.x) || (node.dx < 0 && v2.x < v1.x) {
				return [LineKind::Zero, sidev0, sidev1];
			} else {
				return [LineKind::One, sidev0, sidev1];
			}
		} else {
			if (node.dz > 0 && v2.z > v1.z) || (node.dz < 0 && v2.z < v1.z) {
				return [LineKind::Zero, sidev0, sidev1];
			} else {
				return [LineKind::One, sidev0, sidev1];
			}
		}
	} else if sidev0 != LineKind::One && sidev1 != LineKind::One {
		return [LineKind::Zero, sidev0, sidev1];
	} else if sidev0 != LineKind::NegOne && sidev1 != LineKind::NegOne {
		return [LineKind::One, sidev0, sidev1];
	}

	[LineKind::NegOne, sidev0, sidev1]
}
