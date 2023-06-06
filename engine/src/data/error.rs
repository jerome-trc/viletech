//! Things that can go wrong during data management operations.

use image::ImageError;
use util::{EditorNum, Id8};
use vfs::VPathBuf;

use crate::udmf;

/// Things that can go wrong during (non-preparation) datum management operations,
/// like lookup and mutation. Also see [`PrepError`].
#[derive(Debug)]
pub enum DatumError {
	/// A data object ID didn't resolve to anything.
	NotFound(String),
}

impl std::error::Error for DatumError {}

impl std::fmt::Display for DatumError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::NotFound(id) => {
				write!(f, "No data object exists by the ID: {id}")
			}
		}
	}
}

#[derive(Debug)]
pub struct PrepError {
	pub path: VPathBuf,
	pub kind: PrepErrorKind,
}

impl PrepError {
	#[must_use]
	pub fn is_fatal(&self) -> bool {
		match &self.kind {
			PrepErrorKind::Level(err) => err.is_fatal(),
			PrepErrorKind::ColorMap(_)
			| PrepErrorKind::EnDoom(_)
			| PrepErrorKind::Flat
			| PrepErrorKind::Image(_)
			| PrepErrorKind::PNames
			| PrepErrorKind::Sprite
			| PrepErrorKind::TextureX
			| PrepErrorKind::WaveformAudio(_) => false,
			PrepErrorKind::Io(_) | PrepErrorKind::MissingVzsDir | PrepErrorKind::VzsParse(_) => {
				true
			}
		}
	}
}

/// Game loading is a two-step process; data preparation is the second step.
/// This covers the errors that can possibly happen during these operations.
#[derive(Debug)]
pub enum PrepErrorKind {
	/// A [COLORMAP] WAD lump is the wrong size.
	///
	/// [COLORMAP]: https://doomwiki.org/wiki/COLORMAP
	ColorMap(usize),
	/// An [ENDOOM] WAD lump is the wrong size.
	///
	/// [ENDOOM]: https://doomwiki.org/wiki/ENDOOM
	EnDoom(usize),
	/// A file between the `F_START` and `F_END` markers was not 4096 bytes in size.
	///
	/// See <https://doomwiki.org/wiki/WAD#Flats.2C_Sprites.2C_and_Patches>.
	Flat,
	Level(LevelError),
	/// Tried to decode a non-picture format image and failed.
	Image(ImageError),
	Io(std::io::Error),
	/// A mount declared a script root file that was not found in the VFS.
	MissingVzsDir,
	/// A [PNAMES] WAD lump is too short or an incorrect size.
	///
	/// [PNAMES]: https://doomwiki.org/wiki/PNAMES
	PNames,
	/// A file between the `S_START` and `S_END` markers is not in picture format,
	/// or any other recognized image format.
	///
	/// See <https://doomwiki.org/wiki/WAD#Flats.2C_Sprites.2C_and_Patches>.
	Sprite,
	/// A [TEXTURE1 or TEXTURE2] WAD lump is too short or an incorrect size.
	///
	/// [TEXTURE1 or TEXTURE2]: https://doomwiki.org/wiki/TEXTURE1_and_TEXTURE2
	TextureX,
	VzsParse(vzs::parse::Error<'static>),
	/// Failure to decode a FLAC, MP3, Ogg, or WAV file.
	WaveformAudio(kira::sound::FromFileError),
}

impl std::error::Error for PrepError {}

impl std::fmt::Display for PrepError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.kind {
			PrepErrorKind::ColorMap(size) => {
				write!(
					f,
					"`COLORMAP` lump is the wrong size: {p}\r\n\t\
					Expected 8704, but found: {size}",
					p = self.path.display()
				)
			}
			PrepErrorKind::EnDoom(size) => {
				write!(
					f,
					"`ENDOOM` lump is the wrong size: {p}\r\n\t\
					Expected 4000, but found: {size}",
					p = self.path.display()
				)
			}
			PrepErrorKind::Flat => {
				write!(
					f,
					"Lump {} is between `F_START` and `F_END` but is not a flat.",
					self.path.display()
				)
			}
			PrepErrorKind::Io(err) => err.fmt(f),
			PrepErrorKind::Level(err) => {
				write!(f, "Map `{}` is invalid. Reason: {err}", self.path.display())
			}
			PrepErrorKind::Image(err) => {
				write!(
					f,
					"Failed to decode image: {p}\r\n\tDetails: {err}",
					p = self.path.display()
				)
			}
			PrepErrorKind::MissingVzsDir => {
				write!(
					f,
					"VZScript root directory not found at path: {}",
					self.path.display()
				)
			}
			PrepErrorKind::PNames => {
				write!(f, "Malformed PNAMES lump: {}", self.path.display())
			}
			PrepErrorKind::Sprite => {
				write!(
					f,
					"Lump {} is between `S_START` and `S_END` \
					but is not a recognized sprite format.",
					self.path.display()
				)
			}
			PrepErrorKind::TextureX => {
				write!(
					f,
					"Malformed TEXTURE1 or TEXTURE2 lump: {}",
					self.path.display()
				)
			}
			PrepErrorKind::VzsParse(_) => todo!(),
			PrepErrorKind::WaveformAudio(err) => write!(
				f,
				"Failed to load audio file: {p}\r\n\t\
				Details: {err}",
				p = self.path.display()
			),
		}
	}
}

/// Things that can go wrong when trying to process files into a [Level] datum.
///
/// [Level]: super::dobj::Level
#[derive(Debug)]
pub enum LevelError {
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
	MalformedFile(VPathBuf),
	/// No thingdef was defined as a player 1 starting location.
	NoPlayer1Start,
	TextmapParse(udmf::Error),
	/// A VFS entry was deduced to be a level component,
	/// but is empty or a directory.
	UnreadableFile(VPathBuf),
	UnknownEdNum {
		thingdef: usize,
		ed_num: EditorNum,
	},
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

/// See [`LevelError::UnknownSideTex`].
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

impl LevelError {
	#[must_use]
	pub fn is_fatal(&self) -> bool {
		false
	}
}

impl std::fmt::Display for LevelError {
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
					"Linedef {linedef} references {l_or_r} side {sidedef}, \
					but only {sides_len} sidedefs exist."
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
				but only {ssectors_len} sub-sectors exist."
				)
			}
			Self::InvalidSegLinedef {
				seg,
				linedef,
				lines_len,
			} => {
				write!(
					f,
					"Seg {seg} references linedef {linedef}, \
				but only {lines_len} linedefs exist."
				)
			}
			Self::InvalidSidedefSector {
				sidedef,
				sector,
				sectors_len,
			} => {
				write!(
					f,
					"Sidedef {sidedef} references sector {sector}, \
					but only {sectors_len} sectors exist."
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
				but only {nodes_len} nodes exist."
				)
			}
			Self::InvalidSubsectorSeg {
				subsector,
				seg,
				segs_len,
			} => {
				write!(
					f,
					"Sub-sector {subsector} references seg {seg}, but only {segs_len} exist."
				)
			}
			Self::MalformedFile(file) => {
				write!(f, "`{}` has malformed contents.", file.display())
			}
			Self::NoPlayer1Start => {
				write!(
					f,
					"No thingdef was defined as a player 1 starting location."
				)
			}
			Self::TextmapParse(err) => {
				write!(f, "Error while parsing `TEXTMAP`: {err}")
			}
			Self::UnreadableFile(file) => {
				write!(f, "`{}` is empty or a directory.", file.display())
			}
			Self::UnknownEdNum { thingdef, ed_num } => {
				write!(f, "Thing {thingdef} has unknown editor number: {ed_num}")
			}
			Self::UnknownFlat {
				sector,
				ceiling,
				name,
			} => {
				let c_or_f = if *ceiling { "ceiling" } else { "floor" };

				write!(
					f,
					"Sector {sector} references non-existent {c_or_f} texture {name}."
				)
			}
			Self::UnknownLineSpecial(short) => {
				write!(f, "Unknown line special: {short}")
			}
			Self::UnknownSectorSpecial(short) => {
				write!(f, "Unknown sector special: {short}")
			}
			Self::UnknownSideTex {
				sidedef,
				which,
				name,
			} => {
				write!(
					f,
					"Sidedef {sidedef} references non-existent {which} texture {name}."
				)
			}
		}
	}
}
