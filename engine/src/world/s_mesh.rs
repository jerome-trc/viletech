//! A routine for triangulating a Doom level using its sectors.
//!
//! All code below is based on Jazz Mickle's algorithm for triangulating Doom levels.
//! See <https://medium.com/@jmickle_/build-a-model-of-a-doom-level-7283addf009f>.
//!
//! This code *almost* works, but not quite; it breaks down upon encountering
//! complex polygons (i.e. those where a vertex connects more than two lines).
//! Feel free to try to make it work if you're interested in this kind of thing.

use bevy::utils::petgraph::{graphmap::UnGraphMap, visit::DfsPostOrder};
use data::level::RawLevel;
use geo::{triangulate_earcut::RawTriangulation, TriangulateEarcut, Winding, Within};
use glam::Vec2;
use rayon::prelude::*;
use rustc_hash::FxHashSet;
use slotmap::SlotMap;

use crate::types::FxDashMap;

/// Note that this is one triangle amongst potentially many making up one sector.
#[derive(Debug, Clone, PartialEq)]
pub struct SectorPoly {
	pub sector: usize,
	pub verts: Vec<Vec2>,
	pub indices: Vec<usize>,
}

/// Note that this is one triangle amongst potentially many making up one sector.
#[derive(Debug, Clone, PartialEq)]
pub struct SectorTri {
	pub sector: usize,
	pub inner: RawTriangulation<f32>,
}

pub fn polygonize_par<F>(raw: RawLevel, callback: F)
where
	F: Send + Sync + Fn(SectorTri),
{
	let sgraphs = FxDashMap::<usize, SectorGraph>::default();

	// Generate one adjacency graph per sector.

	raw.linedefs.par_iter().for_each(|linedef| {
		let i_r = linedef.right_side() as usize;
		let side_r = &raw.sidedefs[i_r];
		let sector_r = side_r.sector() as usize;
		let mut sgraph = sgraphs
			.entry(sector_r)
			.or_insert(SectorGraph::new(sector_r));
		sgraph.add_verts(
			raw,
			linedef.start_vertex(),
			linedef.end_vertex(),
			LineSide::Right,
		);

		if let Some(i_l) = linedef.left_side().map(|i| i as usize) {
			let side_l = &raw.sidedefs[i_l];
			let sector_l = side_l.sector() as usize;

			if sector_l == sector_r {
				sgraph.add_verts(
					raw,
					linedef.start_vertex(),
					linedef.end_vertex(),
					LineSide::Left,
				);
			} else {
				drop(sgraph);

				let mut sgraph = sgraphs
					.entry(sector_l)
					.or_insert(SectorGraph::new(sector_l));

				sgraph.add_verts(
					raw,
					linedef.start_vertex(),
					linedef.end_vertex(),
					LineSide::Left,
				);
			}
		}
	});

	// Reduce a sector to polygons by tracing lines.
	// Some will be "shells" and some will be "holes".

	sgraphs.par_iter().for_each(|sgraph| {
		let mut polys = SlotMap::new();

		let mut unvisited = FxHashSet::default();

		for node in sgraph.graph.nodes() {
			unvisited.insert(node);
		}

		loop {
			let mut coords = vec![];

			let Some(base) = unvisited.iter().copied().next() else {
				break;
			};

			let mut traverser = DfsPostOrder::new(&sgraph.graph, base);

			while let Some(traversed) = traverser.next(&sgraph.graph) {
				coords.push(geo::Coord {
					x: (traversed.x as f32) * super::FSCALE,
					y: (traversed.y as f32) * super::FSCALE,
				});

				unvisited.remove(&traversed);
			}

			// It is legal for a Doom level to have a linedef inside a sector that
			// does not connect to any other lines.

			if coords.len() < 3 {
				continue;
			}

			let mut line_str = geo::LineString::new(coords);
			line_str.make_ccw_winding();
			polys.insert(geo::Polygon::new(line_str, vec![]));
		}

		let mut relations = vec![];

		for (i, poly) in &polys {
			for (o, other) in &polys {
				if i == o {
					continue;
				}

				if other.exterior().is_within(poly) {
					relations.push((i, o));
				} else if poly.exterior().is_within(other) {
					relations.push((o, i));
				}
			}
		}

		for (shell_slot, hole_slot) in relations {
			if let Some(hole) = polys.remove(hole_slot) {
				let innards = hole.into_inner();
				debug_assert!(innards.1.is_empty());
				polys[shell_slot].interiors_push(innards.0);
			}
		}

		for (_slot, poly) in polys {
			let tri = poly.earcut_triangles_raw();

			callback(SectorTri {
				inner: tri,
				sector: sgraph.sector,
			});
		}
	});
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct SectorGraphNode {
	x: i16,
	y: i16,
}

struct SectorGraph {
	sector: usize,
	graph: UnGraphMap<SectorGraphNode, LineSide>,
}

impl SectorGraph {
	#[must_use]
	fn new(sector: usize) -> Self {
		Self {
			sector,
			graph: UnGraphMap::default(),
		}
	}

	fn add_verts(&mut self, raw: RawLevel, v_start_ix: u16, v_end_ix: u16, side: LineSide) {
		let v_start = raw.vertices[v_start_ix as usize];
		let v_end = raw.vertices[v_end_ix as usize];

		let ix_start = self.graph.add_node(SectorGraphNode {
			x: v_start.position()[0],
			y: v_start.position()[1],
		});

		let ix_end = self.graph.add_node(SectorGraphNode {
			x: v_end.position()[0],
			y: v_end.position()[1],
		});

		self.graph.add_edge(ix_start, ix_end, side);
	}
}

impl std::ops::Deref for SectorGraph {
	type Target = UnGraphMap<SectorGraphNode, LineSide>;

	fn deref(&self) -> &Self::Target {
		&self.graph
	}
}

impl std::ops::DerefMut for SectorGraph {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.graph
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineSide {
	Left,
	Right,
}
