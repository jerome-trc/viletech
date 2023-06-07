//! VileTech's node builder; a forked [ZDBSP](https://github.com/ZDoom/zdbsp).

mod bsp;
mod classify;
mod events;
mod polyobj;
mod segs;

use std::sync::atomic::{self, AtomicUsize};

use bitvec::vec::BitVec;
use rayon::prelude::*;

use crate::{
	data::dobj::{BspNodeChild, Level, LevelFormat, Thing},
	math::{point_to_angle, Fixed32, MinMaxBox, UAngle},
};

use self::events::EventTree;

/// Data structures needed for node-building, exposed so that its allocations
/// can be re-used between levels. Each call to [`Self::build`] is idempotent.
#[derive(Debug)]
pub struct Context {
	gl: bool,
	vmap: VertexMap,
	verts: Vec<Vertex>,
	segs: Vec<Seg>,
	nodes: Vec<BspNode>,
	planes: Vec<FxDisp>,
	plane_checked: BitVec,
	events: EventTree,
}

impl Context {
	#[must_use]
	pub fn new(gl: bool) -> Self {
		Self {
			gl,
			vmap: VertexMap::default(),
			verts: vec![],
			segs: vec![],
			nodes: vec![],
			planes: vec![],
			plane_checked: BitVec::default(),
			events: EventTree::default(),
		}
	}

	pub fn build(&mut self, level: &mut Level) {
		assert!(
			level.nodes.is_empty(),
			"Tried to build BSP nodes for a level which already had some."
		);
		assert!(
			level.subsectors.is_empty(),
			"Tried to build BSP nodes for a level which already had subsectors."
		);
		assert!(
			level.segs.is_empty(),
			"Tried to build BSP nodes for a level which already had segs."
		);

		self.reset();

		let cleaned = CleaningInfo {
			old_verts_len: 0,
			old_sectors: level.prune_unused_sectors(),
		};

		self.vmap.prepare(level.bounds.clone());
		let poly = polyspots(level);

		let mut builder = NodeBuilder {
			level,
			ctx: self,
			cleaned,
			poly,
			initial_verts_len: usize::MAX,
			hack_seg: usize::MAX,
			hack_mate: usize::MAX,
		};

		builder.find_used_verts();
		builder.create_segs_from_sides();
		builder.find_poly_containers();
		builder.group_seg_planes();
	}

	fn reset(&mut self) {
		self.verts.clear();
		self.segs.clear();
		self.nodes.clear();
		self.planes.clear();
		self.plane_checked.clear();
		self.events.clear();

		self.vmap.grid.clear();
		self.vmap.min_x = Fixed32::ZERO;
		self.vmap.min_z = Fixed32::ZERO;
		self.vmap.max_x = Fixed32::ZERO;
		self.vmap.max_z = Fixed32::ZERO;
		self.vmap.blocks_tall = usize::MAX;
		self.vmap.blocks_wide = usize::MAX;
	}
}

#[derive(Debug)]
pub(self) struct NodeBuilder<'lvl> {
	ctx: &'lvl mut Context,
	level: &'lvl mut Level,
	poly: PolySpots,
	cleaned: CleaningInfo,
	/// (GZ) The number of vertices that are connected to linedefs.
	initial_verts_len: usize,
	hack_seg: usize,
	hack_mate: usize,
}

impl std::ops::Deref for NodeBuilder<'_> {
	type Target = Context;

	fn deref(&self) -> &Self::Target {
		&self.ctx
	}
}

impl std::ops::DerefMut for NodeBuilder<'_> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.ctx
	}
}

/// Information derived from pruning extraneous level geometry,
/// which happens to have other applications in these operations.
#[derive(Debug)]
struct CleaningInfo {
	old_verts_len: usize,
	old_sectors: Vec<usize>,
}

/// (GZ) Like a blockmap, but for vertices instead of lines.
#[derive(Debug, Default)]
struct VertexMap {
	min_x: Fixed32,
	min_z: Fixed32,
	max_x: Fixed32,
	max_z: Fixed32,
	blocks_wide: usize,
	blocks_tall: usize,
	// TODO: Test if using a DashMap allows performance increases.
	grid: Vec<Vec<usize>>,
}

impl VertexMap {
	const BLOCK_SHIFT: u32 = 8 + 16;
	const BLOCK_SIZE: i32 = 1 << (Self::BLOCK_SHIFT as i32);

	fn prepare(&mut self, bounds: MinMaxBox) {
		// Yes, the original ZDBSP really did bash numbers together like this.
		let min_x = (bounds.min.x as i32) as f64;
		let min_z = (bounds.min.y as i32) as f64;

		let max_x: f64 = Fixed32::from_num(bounds.max.x).into();
		let max_z: f64 = Fixed32::from_num(bounds.max.y).into();

		let w = (((max_x - min_x) as i32 - 1) + (Self::BLOCK_SIZE - 1)) / Self::BLOCK_SIZE;
		let t = (((max_z - min_z) as i32 - 1) + (Self::BLOCK_SIZE - 1)) / Self::BLOCK_SIZE;

		self.min_x = Fixed32::from_num(min_x);
		self.min_z = Fixed32::from_num(min_z);
		self.max_x = Fixed32::from_num(max_x);
		self.max_z = Fixed32::from_num(max_z);
		self.blocks_wide = w as usize;
		self.blocks_tall = t as usize;
		self.grid.resize(w as usize * t as usize, vec![]);
	}
}

/// Step 1: vertex map population.
impl NodeBuilder<'_> {
	const VERTEX_EPSILON: Fixed32 = Fixed32::from_bits(6_i32);

	fn find_used_verts(&mut self) {
		let mut map = Vec::with_capacity(self.level.vertices.len());
		map.resize_with(self.level.vertices.len(), || AtomicUsize::new(usize::MAX));

		(0..self.level.linedefs.len()).for_each(|i| {
			let v1 = self.level.linedefs[i].vert_start;
			let v2 = self.level.linedefs[i].vert_end;

			let mut new;

			let map_v1 = map[v1].load(atomic::Ordering::Acquire);
			let map_v2 = map[v2].load(atomic::Ordering::Acquire);

			let map_v1 = if map_v1 == usize::MAX {
				new = Vertex {
					x: self.level.vertices[v1].x_fixed(),
					z: self.level.vertices[v1].z_fixed(),
					segs: usize::MAX,
					segs2: usize::MAX,
				};

				let map_v1 = self.select_or_insert_vert(new);
				map[v1].store(map_v1, atomic::Ordering::Release);
				map_v1
			} else {
				map_v1
			};

			let map_v2 = if map_v2 == usize::MAX {
				new = Vertex {
					x: self.level.vertices[v2].x_fixed(),
					z: self.level.vertices[v2].z_fixed(),
					segs: usize::MAX,
					segs2: usize::MAX,
				};

				let map_v2 = self.select_or_insert_vert(new);
				map[v2].store(map_v2, atomic::Ordering::Release);
				map_v2
			} else {
				map_v2
			};

			self.level.linedefs[i].vert_start = map_v1;
			self.level.linedefs[i].vert_end = map_v2;
		});

		self.initial_verts_len = self.verts.len();
	}

	#[must_use]
	fn select_or_insert_vert(&mut self, vert: Vertex) -> usize {
		let block = &self.vmap.grid[self.get_block(vert.x, vert.z)];

		for &v_ndx in block {
			let v = &self.verts[block[v_ndx]];

			if v.x == vert.x && v.z == vert.z {
				return block[v_ndx];
			}
		}

		self.insert_vert(vert)
	}

	#[must_use]
	fn insert_vert(&mut self, vert: Vertex) -> usize {
		let ret = self.verts.len();

		let min_x = self.vmap.min_x.max(vert.x - Self::VERTEX_EPSILON);
		let max_x = self.vmap.max_x.min(vert.x + Self::VERTEX_EPSILON);
		let min_z = self.vmap.min_z.max(vert.z - Self::VERTEX_EPSILON);
		let max_z = self.vmap.max_z.max(vert.z + Self::VERTEX_EPSILON);

		self.verts.push(vert);

		let blocks = [
			self.get_block(min_x, min_z),
			self.get_block(max_x, min_z),
			self.get_block(min_x, max_z),
			self.get_block(max_x, max_z),
		];

		let bcounts = [
			self.vmap.grid[blocks[0]].len(),
			self.vmap.grid[blocks[1]].len(),
			self.vmap.grid[blocks[2]].len(),
			self.vmap.grid[blocks[3]].len(),
		];

		for i in 0..4 {
			if self.vmap.grid[blocks[i]].len() == bcounts[i] {
				self.vmap.grid[blocks[i]].push(ret);
			}
		}

		ret
	}

	#[must_use]
	fn get_block(&self, x: Fixed32, z: Fixed32) -> usize {
		debug_assert!(
			x >= self.vmap.min_x
				&& z >= self.vmap.min_z
				&& x <= self.vmap.max_x
				&& z <= self.vmap.max_z
		);

		let ret_x = (x - self.vmap.min_x).to_bits() as usize;
		let ret_z = (z - self.vmap.min_z).to_bits() as usize;
		let ret_x = ret_x >> VertexMap::BLOCK_SHIFT;
		let ret_z = ret_z >> VertexMap::BLOCK_SHIFT;

		ret_x + ret_z * self.vmap.blocks_wide
	}
}

/// Step 4: seg plane grouping.
impl NodeBuilder<'_> {
	/// (GZ) Group colinear segs together so that only one seg per line needs to
	/// be checked by `Self::select_splitter`.
	fn group_seg_planes(&mut self) {
		const BUCKET_BITS: u32 = 12;
		let buckets = [usize::MAX; 1 << BUCKET_BITS];

		self.segs.par_iter_mut().enumerate().for_each(|(i, seg)| {
			seg.next = i + 1;
			seg.to_check_next = usize::MAX;
		});

		self.segs.last_mut().unwrap().next = usize::MAX;

		let plane_num = 0;

		for i in 0..self.segs.len() {
			let seg = &self.segs[i];

			let p1 = FixedXZ::new(self.verts[seg.v1].x, self.verts[seg.v1].z);
			let p2 = FixedXZ::new(self.verts[seg.v2].x, self.verts[seg.v2].z);

			let mut ang = point_to_angle(p2.x - p1.x, p2.z - p1.z);

			if ang >= (1 << 31) {
				ang += 1 << 31;
			}

			ang >>= 31 - BUCKET_BITS;
			let mut check = buckets[ang as usize];

			while check != usize::MAX {
				let checked = &self.segs[check];

				let ckstart = FixedXZ::new(self.verts[checked.v1].x, self.verts[checked.v1].z);
				let ckdelta = FixedXZ::new(
					self.verts[checked.v2].x - ckstart.x,
					self.verts[checked.v2].z - ckstart.z,
				);

				let p1rel = pt_side_rel(p1, ckstart, ckdelta);
				let p2rel = pt_side_rel(p2, ckstart, ckdelta);

				if p1rel == PointSideRel::OnLine && p2rel == PointSideRel::OnLine {
					break;
				}

				check = checked.to_check_next;
			}

			if check != usize::MAX {
				let pn = self.segs[check].plane_num;
				let seg = &mut self.segs[i];
				seg.plane_num = pn;
			} else {

			}
		}

		self.plane_checked.reserve((plane_num + 7) / 8);
	}
}

// Common detail types /////////////////////////////////////////////////////////

/// Two-axis 32-bit fixed-point vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(self) struct FixedXZ {
	pub(self) x: Fixed32,
	pub(self) z: Fixed32,
}

impl FixedXZ {
	#[must_use]
	pub(self) fn new(x: Fixed32, z: Fixed32) -> Self {
		Self { x, z }
	}
}

/// "Fixed-point displacement line".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(self) struct FxDisp {
	pub(self) start: FixedXZ,
	pub(self) disp: FixedXZ,
}

/// The node builder's understanding of [`Vertex`](crate::sim::level::Vertex).
#[derive(Debug, Clone, PartialEq)]
#[repr(align(8))] // (GZ)
pub(self) struct Vertex {
	pub(self) x: Fixed32,
	pub(self) z: Fixed32,
	/// Segs that use this vertex as a start.
	pub(self) segs: usize,
	/// Segs that use this vertex as an end.
	pub(self) segs2: usize,
}

impl Vertex {
	#[must_use]
	pub(self) fn to_point(&self) -> FixedXZ {
		FixedXZ::new(self.x, self.z)
	}
}

#[derive(Debug)]
pub(self) struct Seg {
	v1: usize,
	v2: usize,
	sidedef: usize,
	linedef: usize,
	sector_front: usize,
	sector_back: usize,
	next: usize,
	next_for_vert: usize,
	next_for_vert2: usize,
	/// (GZ) For split avoidance. 0 means that splitting is O.K.
	loop_num: usize,
	/// (GZ) Index of the seg on the back side.
	partner: usize,
	/// (GZ) Index from the GL_SEGS lump.
	stored_seg: usize,
	angle: UAngle,
	offset: Fixed32,
	plane_num: usize,
	plane_front: bool,
	to_check_next: usize,
}

/// 32-bit fixed-point bounding box.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(self) struct BBox([Fixed32; 4]);

impl BBox {
	#[must_use]
	fn bottom(&mut self) -> &mut Fixed32 {
		&mut self.0[1]
	}

	#[must_use]
	fn left(&mut self) -> &mut Fixed32 {
		&mut self.0[2]
	}

	#[must_use]
	fn right(&mut self) -> &mut Fixed32 {
		&mut self.0[3]
	}

	#[must_use]
	fn top(&mut self) -> &mut Fixed32 {
		&mut self.0[0]
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(self) struct PolyStart {
	pub(self) polynum: u32,
	pub(self) x: Fixed32,
	pub(self) z: Fixed32,
}

#[derive(Debug)]
pub(self) struct PolySpots {
	pub(self) starts: Vec<PolyStart>,
	pub(self) anchors: Vec<PolyStart>,
}

#[must_use]
fn polyspots(level: &Level) -> PolySpots {
	assert_eq!(
		level.format,
		LevelFormat::Extended,
		"Can only calculate polyobject spots for extended-format levels."
	);

	let is_hexen = level.things.par_iter().any(|thing| thing.ed_num == 3000);

	let (spot1_num, spot2_num, anchor_num) = if is_hexen {
		(
			Thing::HEXEN_SPAWN,
			Thing::HEXEN_SPAWNCRUSH,
			Thing::HEXEN_ANCHOR,
		)
	} else {
		(
			Thing::DOOM_SPAWN,
			Thing::DOOM_SPAWNCRUSH,
			Thing::DOOM_ANCHOR,
		)
	};

	let mut starts = vec![];
	let mut anchors = vec![];

	for thing in &level.things {
		if thing.ed_num == spot1_num
			|| thing.ed_num == spot2_num
			|| thing.ed_num == Thing::DOOM_SPAWNHURT
			|| thing.ed_num == anchor_num
		{
			let v = PolyStart {
				polynum: thing.angle,
				x: Fixed32::from_num(thing.pos.x),
				z: Fixed32::from_num(thing.pos.y),
			};

			if thing.ed_num == anchor_num {
				anchors.push(v);
			} else {
				starts.push(v);
			}
		}
	}

	PolySpots { starts, anchors }
}

#[derive(Debug)]
pub(self) struct BspNode {
	x: Fixed32,
	z: Fixed32,
	dx: Fixed32,
	dz: Fixed32,
	bboxes: [BBox; 2],
	child_r: BspNodeChild,
	child_l: BspNodeChild,
}

/// "Point-side relationship".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(self) enum PointSideRel {
	AheadOfLine,
	OnLine,
	BehindLine,
}

/// (GZ) Points within this distance of a line will be considered on the line.
pub(self) const SIDE_EPSILON: f64 = 6.5536;

#[must_use]
pub(self) fn pt_side_rel(point: FixedXZ, start: FixedXZ, delta: FixedXZ) -> PointSideRel {
	let d_dx = delta.x.to_num::<f64>();
	let d_dy = delta.z.to_num::<f64>();
	let d_x = point.x.to_num::<f64>();
	let d_y = point.z.to_num::<f64>();
	let d_x1 = start.x.to_num::<f64>();
	let d_y1 = start.x.to_num::<f64>();

	// (GZ) For most cases, a simple dot product is enough.
	let s_num = (d_y1 - d_y) * d_dx - (d_x1 - d_x) * d_dy;

	if s_num.abs() < 17179869184.0
	/* 4 << 32 */
	{
		// (GZ) Either the point is very near the line, or the segment defining
		// the line is very short: do a more expensive test to determine just how
		// far the point is from the line.
		let l = d_dx * d_dx + d_dy * d_dy;
		let dist = s_num * s_num / l;

		if dist < (SIDE_EPSILON * SIDE_EPSILON) {
			return PointSideRel::OnLine;
		}
	}

	if s_num > 0.0 {
		PointSideRel::AheadOfLine
	} else {
		PointSideRel::BehindLine
	}
}
