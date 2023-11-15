//! Code used for reading, storing, manipulating, and writing Doom levels.

pub mod read;
pub mod udmf;
pub mod zdbsp;

use util::Id8;

use crate::EditorNum;

/// Exists only to bundle multiple raw level data types to simplify other interfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawLevel<'r> {
	pub linedefs: &'r [read::LineDefRaw],
	pub nodes: &'r [read::NodeRaw],
	pub sectors: &'r [read::SectorRaw],
	pub segs: &'r [read::SegRaw],
	pub sidedefs: &'r [read::SideDefRaw],
	pub subsectors: &'r [read::SSectorRaw],
	pub things: RawThings<'r>,
	pub vertices: &'r [read::VertexRaw],
}

/// See [`RawLevel`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawThings<'r> {
	Doom(&'r [read::ThingRaw]),
	Ext(&'r [read::ThingExtRaw]),
}

/// Certain important ["editor numbers"](https://zdoom.org/wiki/Editor_number).
pub mod ednums {
	use crate::EditorNum;

	pub const HEXEN_ANCHOR: EditorNum = 3000;
	pub const HEXEN_SPAWN: EditorNum = 3001;
	pub const HEXEN_SPAWNCRUSH: EditorNum = 3002;

	pub const DOOM_ANCHOR: EditorNum = 9300;
	pub const DOOM_SPAWN: EditorNum = 9301;
	pub const DOOM_SPAWNCRUSH: EditorNum = 9302;
	pub const DOOM_SPAWNHURT: EditorNum = 9303;
}

/// Possible failure modes of trying to process files into a [LevelDef].
#[derive(Debug)]
pub enum Error {
	/// A line tried to reference a non-existent side.
	InvalidLinedefSide {
		linedef: usize,
		left: bool,
		sidedef: usize,
		sides_len: usize,
	},
	/// A seg tried to reference a non-existent linedef.
	InvalidSegLinedef {
		seg: usize,
		linedef: usize,
		lines_len: usize,
	},
	/// A BSP node tried to reference a non-existent child node.
	InvalidSubnode {
		node: usize,
		left: bool,
		subnode: usize,
		nodes_len: usize,
	},
	/// A BSP node tried to reference a non-existent subsector.
	InvalidNodeSubsector {
		node: usize,
		left: bool,
		ssector: usize,
		ssectors_len: usize,
	},
	/// A sidedef tried to reference a non-existent sector.
	InvalidSidedefSector {
		sidedef: usize,
		sector: usize,
		sectors_len: usize,
	},
	/// A subsector tried to reference a non-existent seg.
	InvalidSubsectorSeg {
		subsector: usize,
		seg: usize,
		segs_len: usize,
	},
	/// For example, a file's byte length is not divisible
	/// by the size of its individual structures.
	///
	/// The containde slice will be something like `VERTEXES` or `THINGS`.
	MalformedFile(&'static str),
	/// No thingdef was defined as a player 1 starting location.
	NoPlayer1Start,
	TextmapParse(udmf::Error),
	UnknownEdNum {
		thingdef: usize,
		ed_num: EditorNum,
	},
	Udmf(udmf::Error),
	/// A sector tried to reference a non-existent texture.
	UnknownFlat {
		sector: usize,
		ceiling: bool,
		name: Id8,
	},
	/// Non-fatal; the line is treated as though it has no special.
	UnknownLineSpecial(i16),
	/// Non-fatal; the sector is treated as though it has no special.
	UnknownSectorSpecial(i16),
	UnknownSideTex {
		sidedef: usize,
		which: SideTexture,
		name: Id8,
	},
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::InvalidLinedefSide {
				linedef,
				left,
				sidedef,
				sides_len,
			} => {
				let l_or_r = if *left { "left" } else { "right" };

				write!(
					f,
					"linedef {linedef} references {l_or_r} side {sidedef}, \
					but only {sides_len} sidedefs exist"
				)
			}
			Self::InvalidNodeSubsector {
				node,
				left,
				ssector,
				ssectors_len,
			} => {
				let l_or_r = if *left { "left" } else { "right" };

				write!(
					f,
					"BSP node {node} references {l_or_r} sub-sector {ssector}, \
				but only {ssectors_len} sub-sectors exist"
				)
			}
			Self::InvalidSegLinedef {
				seg,
				linedef,
				lines_len,
			} => {
				write!(
					f,
					"seg {seg} references linedef {linedef}, \
				but only {lines_len} linedefs exist"
				)
			}
			Self::InvalidSidedefSector {
				sidedef,
				sector,
				sectors_len,
			} => {
				write!(
					f,
					"sidedef {sidedef} references sector {sector}, \
					but only {sectors_len} sectors exist"
				)
			}
			Self::InvalidSubnode {
				node,
				left,
				subnode,
				nodes_len,
			} => {
				let l_or_r = if *left { "left" } else { "right" };

				write!(
					f,
					"BSP node {node} references {l_or_r} sub-node {subnode}, \
				but only {nodes_len} nodes exist"
				)
			}
			Self::InvalidSubsectorSeg {
				subsector,
				seg,
				segs_len,
			} => {
				write!(
					f,
					"sub-sector {subsector} references seg {seg}, but only {segs_len} exist"
				)
			}
			Self::MalformedFile(name) => {
				write!(f, "`{name}` has malformed contents")
			}
			Self::NoPlayer1Start => {
				write!(f, "no thingdef was defined as a player 1 starting location")
			}
			Self::TextmapParse(err) => {
				write!(f, "error while parsing `TEXTMAP`: {err}")
			}
			Self::Udmf(err) => err.fmt(f),
			Self::UnknownEdNum { thingdef, ed_num } => {
				write!(f, "thing {thingdef} has unknown editor number: {ed_num}")
			}
			Self::UnknownFlat {
				sector,
				ceiling,
				name,
			} => {
				let c_or_f = if *ceiling { "ceiling" } else { "floor" };

				write!(
					f,
					"eector {sector} references non-existent {c_or_f} texture `{name}`"
				)
			}
			Self::UnknownLineSpecial(short) => {
				write!(f, "unknown line special: {short}")
			}
			Self::UnknownSectorSpecial(short) => {
				write!(f, "unknown sector special: {short}")
			}
			Self::UnknownSideTex {
				sidedef,
				which,
				name,
			} => {
				write!(
					f,
					"eidedef {sidedef} references non-existent {which} texture `{name}`"
				)
			}
		}
	}
}

/// See [`Error::UnknownSideTex`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideTexture {
	Bottom,
	Middle,
	Top,
}

impl std::fmt::Display for SideTexture {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Bottom => write!(f, "lower"),
			Self::Middle => write!(f, "middle"),
			Self::Top => write!(f, "upper"),
		}
	}
}
