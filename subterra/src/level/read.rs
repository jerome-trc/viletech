//! Functions for deserializing vanilla ["map lumps"] into levels.
//!
//! ["map lumps"]: https://doomwiki.org/wiki/Lump#Standard_lumps

use std::ops::Range;

use util::{read_id8, Id8};

use super::Error;

// TODO: Serde support for raw structs with correct endianness.

pub mod prelude {
	pub use super::{
		BspNodeChild, LineDefRaw, NodeRaw, SSectorRaw, SectorRaw, SegDirection, SegRaw, SideDefRaw,
		ThingExtRaw, ThingFlags, ThingRaw, VertexRaw,
	};
}

// LINEDEFS ////////////////////////////////////////////////////////////////////

/// See <https://doomwiki.org/wiki/Linedef>. Acquired via [`linedefs`].
/// These are cast directly from the bytes of a WAD's lump;
/// attached methods automatically convert from Little Endian.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::Zeroable, bytemuck::Pod)]
pub struct LineDefRaw {
	v_start: u16,
	v_end: u16,
	flags: u16,
	special: u16,
	trigger: u16,
	right: u16,
	left: u16,
}

bitflags::bitflags! {
	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
	pub struct LineFlags: u32 {
		/// Line blocks things (i.e. player, missiles, and monsters).
		const IMPASSIBLE = 1 << 0;
		/// Line blocks monsters.
		const BLOCK_MONS = 1 << 1;
		/// Line's two sides can have the "transparent texture".
		const TWO_SIDED = 1 << 2;
		/// Upper texture is pasted onto wall from the top down instead of bottom-up.
		const UPPER_UNPEGGED = 1 << 3;
		/// Lower and middle textures are drawn from the bottom up instead of top-down.
		const LOWER_UNPEGGED = 1 << 4;
		/// If set, drawn as 1S on the map.
		const SECRET = 1 << 5;
		/// If set, blocks sound propagation.
		const BLOCK_SOUND = 1 << 6;
		/// If set, line is never drawn on the automap,
		/// even if the computer area map power-up is acquired.
		const UNMAPPED = 1 << 7;
		/// If set, line always appears on the automap,
		/// even if no player has seen it yet.
		const PRE_MAPPED = 1 << 8;
		/// If set, linedef passes use action.
		const PASS_USE = 1 << 9;
		/// Strife translucency.
		const TRANSLUCENT = 1 << 10;
		/// Strife railing.
		const JUMPOVER = 1 << 11;
		/// Strife floater-blocker.
		const BLOCK_FLOATERS = 1 << 12;
		/// Player can cross.
		const ALLOW_PLAYER_CROSS = 1 << 13;
		/// Player can use.
		const ALLOW_PLAYER_USE = 1 << 14;
		/// Monsters can cross.
		const ALLOW_MONS_CROSS = 1 << 15;
		/// Monsters can use.
		const ALLOW_MONS_USE = 1 << 16;
		/// Projectile can activate.
		const IMPACT = 1 << 17;
		const ALLOW_PLAYER_PUSH = 1 << 18;
		const ALLOW_MONS_PUSH = 1 << 19;
		const ALLOW_PROJ_CROSS = 1 << 20;
		const REPEAT_SPECIAL = 1 << 21;
	}
}

impl LineDefRaw {
	/// A possible value for [`Self::special`].
	pub const POBJ_LINE_START: u16 = 1;
	/// A possible value for [`Self::special`].
	pub const POBJ_LINE_EXPLICIT: u16 = 5;

	/// To be used as an index into a slice of [`VertexRaw`].
	#[must_use]
	pub fn start_vertex(&self) -> u16 {
		u16::from_le(self.v_start)
	}

	/// To be used as an index into a slice of [`VertexRaw`].
	#[must_use]
	pub fn end_vertex(&self) -> u16 {
		u16::from_le(self.v_end)
	}

	#[must_use]
	pub fn flags(&self) -> LineFlags {
		LineFlags::from_bits_truncate(u16::from_le(self.flags) as u32)
	}

	#[must_use]
	pub fn special(&self) -> u16 {
		u16::from_le(self.special)
	}

	#[must_use]
	pub fn trigger(&self) -> u16 {
		u16::from_le(self.trigger)
	}

	/// a.k.a. the linedef's "front". To be used as an index into a slice of [`SideDefRaw`].
	#[must_use]
	pub fn right_side(&self) -> u16 {
		u16::from_le(self.right)
	}

	/// a.k.a. the linedef's "back". To be used as an index into a slice of [`SideDefRaw`].
	/// Returns `None` if the LE bytes of this value match the bit pattern `0xFFFF`.
	#[must_use]
	pub fn left_side(&self) -> Option<u16> {
		let s = u16::from_le(self.left);
		(s != 0xFFFF).then_some(s)
	}
}

/// Casts a slice of raw bytes to line definitions (without allocating).
/// Returns [`Error::MalformedFile`] if the length of `lump` is not divisible by 14.
pub fn linedefs(lump: &[u8]) -> Result<&[LineDefRaw], Error> {
	if (lump.len() % std::mem::size_of::<LineDefRaw>()) != 0 {
		return Err(Error::MalformedFile("LINEDEFS"));
	}

	Ok(bytemuck::cast_slice(lump))
}

/// See [`linedefs`].
pub fn linedefs_mut(lump: &mut [u8]) -> Result<&mut [LineDefRaw], Error> {
	if (lump.len() % std::mem::size_of::<LineDefRaw>()) != 0 {
		return Err(Error::MalformedFile("LINEDEFS"));
	}

	Ok(bytemuck::cast_slice_mut(lump))
}

/// Like [`linedefs`], but any bytes at the end of slice which do not fit into
/// another [`LineDefRaw`] are truncated.
#[must_use]
pub fn linedefs_lossy(lump: &[u8]) -> &[LineDefRaw] {
	let sz = std::mem::size_of::<LineDefRaw>();
	let count = lump.len() / sz;
	let subslice = &lump[..(count * sz)];
	bytemuck::cast_slice(subslice)
}

/// Like [`linedefs_mut`], but any bytes at the end of slice which do not fit into
/// another [`LineDefRaw`] are truncated.
#[must_use]
pub fn linedefs_lossy_mut(lump: &mut [u8]) -> &mut [LineDefRaw] {
	let sz = std::mem::size_of::<LineDefRaw>();
	let count = lump.len() / sz;
	let subslice = &mut lump[..(count * sz)];
	bytemuck::cast_slice_mut(subslice)
}

// NODES ///////////////////////////////////////////////////////////////////////

/// See <https://doomwiki.org/wiki/Node>. Acquired via [`nodes`].
/// These are cast directly from the bytes of a WAD's lump;
/// attached methods automatically convert from Little Endian.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::Zeroable, bytemuck::Pod)]
pub struct NodeRaw {
	x: i16,
	y: i16,
	delta_x: i16,
	delta_y: i16,
	/// Top, bottom, left, right.
	aabb_r: [i16; 4],
	aabb_l: [i16; 4],
	child_r: i16,
	child_l: i16,
}

impl NodeRaw {
	#[must_use]
	pub fn seg_start(&self) -> [i16; 2] {
		[i16::from_le(self.x), i16::from_le(self.y)]
	}

	#[must_use]
	pub fn seg_delta(&self) -> [i16; 2] {
		[i16::from_le(self.delta_x), i16::from_le(self.delta_y)]
	}

	#[must_use]
	pub fn seg_end(&self) -> [i16; 2] {
		[
			i16::from_le(self.x) + i16::from_le(self.delta_x),
			i16::from_le(self.y) + i16::from_le(self.delta_y),
		]
	}

	#[must_use]
	pub fn child_r(&self) -> BspNodeChild {
		let child = i16::from_le(self.child_r);

		if child.is_negative() {
			BspNodeChild::SubSector((child & 0x7FFF) as usize)
		} else {
			BspNodeChild::SubNode(child as usize)
		}
	}

	#[must_use]
	pub fn child_l(&self) -> BspNodeChild {
		let child = i16::from_le(self.child_l);

		if child.is_negative() {
			BspNodeChild::SubSector((child & 0x7FFF) as usize)
		} else {
			BspNodeChild::SubNode(child as usize)
		}
	}
}

/// See [`NodeRaw`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BspNodeChild {
	SubSector(usize),
	SubNode(usize),
}

/// Casts a slice of raw bytes to node definitions (without allocating).
/// Returns [`Error::MalformedFile`] if the length of `lump` is not divisible by 28.
pub fn nodes(lump: &[u8]) -> Result<&[NodeRaw], Error> {
	if (lump.len() % std::mem::size_of::<NodeRaw>()) != 0 {
		return Err(Error::MalformedFile("NODES"));
	}

	Ok(bytemuck::cast_slice(lump))
}

/// See [`nodes`].
pub fn nodes_mut(lump: &mut [u8]) -> Result<&mut [NodeRaw], Error> {
	if (lump.len() % std::mem::size_of::<NodeRaw>()) != 0 {
		return Err(Error::MalformedFile("NODES"));
	}

	Ok(bytemuck::cast_slice_mut(lump))
}

/// Like [`nodes`], but any bytes at the end of slice which do not fit into
/// another [`NodeRaw`] are truncated.
#[must_use]
pub fn nodes_lossy(lump: &[u8]) -> &[NodeRaw] {
	let sz = std::mem::size_of::<NodeRaw>();
	let count = lump.len() / sz;
	let subslice = &lump[..(count * sz)];
	bytemuck::cast_slice(subslice)
}

/// Like [`nodes_mut`], but any bytes at the end of slice which do not fit into
/// another [`NodeRaw`] are truncated.
#[must_use]
pub fn nodes_lossy_mut(lump: &mut [u8]) -> &mut [NodeRaw] {
	let sz = std::mem::size_of::<NodeRaw>();
	let count = lump.len() / sz;
	let subslice = &mut lump[..(count * sz)];
	bytemuck::cast_slice_mut(subslice)
}

// SECTORS /////////////////////////////////////////////////////////////////////

/// See <https://doomwiki.org/wiki/Sector>. Acquired via [`sectors`].
/// These are cast directly from the bytes of a WAD's lump;
/// attached methods automatically convert from Little Endian.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::Zeroable, bytemuck::Pod)]
pub struct SectorRaw {
	height_floor: i16,
	height_ceil: i16,
	tex_floor: [u8; 8],
	tex_ceil: [u8; 8],
	light_level: u16,
	special: u16,
	trigger: u16,
}

impl SectorRaw {
	#[must_use]
	pub fn floor_height(&self) -> i16 {
		i16::from_le(self.height_floor)
	}

	#[must_use]
	pub fn ceiling_height(&self) -> i16 {
		i16::from_le(self.height_ceil)
	}

	/// Returns `None` if the first byte of the floor texture field is NUL.
	#[must_use]
	pub fn floor_texture(&self) -> Option<Id8> {
		read_id8(self.tex_floor)
	}

	/// Returns `None` if the first byte of the ceiling texture field is NUL.
	#[must_use]
	pub fn ceiling_texture(&self) -> Option<Id8> {
		read_id8(self.tex_ceil)
	}

	#[must_use]
	pub fn light_level(&self) -> u16 {
		u16::from_le(self.light_level)
	}

	#[must_use]
	pub fn special(&self) -> u16 {
		u16::from_le(self.special)
	}

	/// Corresponds to [`LineDefRaw::trigger`].
	#[must_use]
	pub fn trigger(&self) -> u16 {
		u16::from_le(self.trigger)
	}
}

/// Casts a slice of raw bytes to sector definitions (without allocating).
/// Returns [`Error::MalformedFile`] if the length of `lump` is not divisible by 26.
pub fn sectors(lump: &[u8]) -> Result<&[SectorRaw], Error> {
	if (lump.len() % std::mem::size_of::<SectorRaw>()) != 0 {
		return Err(Error::MalformedFile("SECTORS"));
	}

	Ok(bytemuck::cast_slice(lump))
}

/// See [`sectors`].
pub fn sectors_mut(lump: &mut [u8]) -> Result<&mut [SectorRaw], Error> {
	if (lump.len() % std::mem::size_of::<SectorRaw>()) != 0 {
		return Err(Error::MalformedFile("SECTORS"));
	}

	Ok(bytemuck::cast_slice_mut(lump))
}

/// Like [`sectors`], but any bytes at the end of slice which do not fit into
/// another [`SectorRaw`] are truncated.
#[must_use]
pub fn sectors_lossy(lump: &[u8]) -> &[SectorRaw] {
	let sz = std::mem::size_of::<SectorRaw>();
	let count = lump.len() / sz;
	let subslice = &lump[..(count * sz)];
	bytemuck::cast_slice(subslice)
}

/// Like [`sectors_mut`], but any bytes at the end of slice which do not fit into
/// another [`SectorRaw`] are truncated.
#[must_use]
pub fn sectors_lossy_mut(lump: &mut [u8]) -> &mut [SectorRaw] {
	let sz = std::mem::size_of::<SectorRaw>();
	let count = lump.len() / sz;
	let subslice = &mut lump[..(count * sz)];
	bytemuck::cast_slice_mut(subslice)
}

// SEGS ////////////////////////////////////////////////////////////////////////

/// See <https://doomwiki.org/wiki/Seg>. Acquired via [`segs`].
/// These are cast directly from the bytes of a WAD's lump;
/// attached methods automatically convert from Little Endian.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::Zeroable, bytemuck::Pod)]
pub struct SegRaw {
	v_start: u16,
	v_end: u16,
	angle: i16,
	linedef: u16,
	direction: i16,
	offset: i16,
}

impl SegRaw {
	/// To be used as an index into a slice of [`VertexRaw`].
	#[must_use]
	pub fn start_vertex(&self) -> u16 {
		u16::from_le(self.v_start)
	}

	/// To be used as an index into a slice of [`VertexRaw`].
	#[must_use]
	pub fn end_vertex(&self) -> u16 {
		u16::from_le(self.v_end)
	}

	/// Returns a binary angle measurement (or "BAMS",
	/// see <https://en.wikipedia.org/wiki/Binary_angular_measurement>).
	#[must_use]
	pub fn angle(&self) -> i16 {
		i16::from_le(self.angle)
	}

	/// To be used as an index into a slice of [`LineDefRaw`].
	#[must_use]
	pub fn linedef(&self) -> u16 {
		u16::from_le(self.linedef)
	}

	#[must_use]
	pub fn direction(&self) -> SegDirection {
		if i16::from_le(self.direction) == 0 {
			SegDirection::Front
		} else {
			SegDirection::Back
		}
	}

	#[must_use]
	pub fn offset(&self) -> i16 {
		i16::from_le(self.offset)
	}
}

/// See [`SegRaw::direction`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SegDirection {
	/// This seg runs along the right of a linedef.
	Front,
	/// This seg runs along the left of a linedef.
	Back,
}

/// Casts a slice of raw bytes to segment definitions (without allocating).
/// Returns [`Error::MalformedFile`] if the length of `lump` is not divisible by 12.
pub fn segs(lump: &[u8]) -> Result<&[SegRaw], Error> {
	if (lump.len() % std::mem::size_of::<SegRaw>()) != 0 {
		return Err(Error::MalformedFile("SEGS"));
	}

	Ok(bytemuck::cast_slice(lump))
}

/// See [`segs`].
pub fn segs_mut(lump: &mut [u8]) -> Result<&mut [SegRaw], Error> {
	if (lump.len() % std::mem::size_of::<SegRaw>()) != 0 {
		return Err(Error::MalformedFile("SEGS"));
	}

	Ok(bytemuck::cast_slice_mut(lump))
}

/// Like [`segs`], but any bytes at the end of slice which do not fit into
/// another [`SegRaw`] are truncated.
#[must_use]
pub fn segs_lossy(lump: &[u8]) -> &[SegRaw] {
	let sz = std::mem::size_of::<SegRaw>();
	let count = lump.len() / sz;
	let subslice = &lump[..(count * sz)];
	bytemuck::cast_slice(subslice)
}

/// Like [`segs_mut`], but any bytes at the end of slice which do not fit into
/// another [`SegRaw`] are truncated.
#[must_use]
pub fn segs_lossy_mut(lump: &mut [u8]) -> &mut [SegRaw] {
	let sz = std::mem::size_of::<SegRaw>();
	let count = lump.len() / sz;
	let subslice = &mut lump[..(count * sz)];
	bytemuck::cast_slice_mut(subslice)
}

// SIDEDEFS ////////////////////////////////////////////////////////////////////

/// See <https://doomwiki.org/wiki/Sidedef>. Acquired via [`sidedefs`].
/// These are cast directly from the bytes of a WAD's lump;
/// attached methods automatically convert from Little Endian.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::Zeroable, bytemuck::Pod)]
pub struct SideDefRaw {
	offs_x: i16,
	offs_y: i16,
	tex_top: [u8; 8],
	tex_bottom: [u8; 8],
	tex_mid: [u8; 8],
	sector: u16,
}

impl SideDefRaw {
	#[must_use]
	pub fn offset(&self) -> [i16; 2] {
		[i16::from_le(self.offs_x), i16::from_le(self.offs_y)]
	}

	/// To be used as an index into a slice of [`SectorRaw`].
	#[must_use]
	pub fn sector(&self) -> u16 {
		u16::from_le(self.sector)
	}

	/// Returns `None` if the first byte of the top texture field is NUL.
	#[must_use]
	pub fn top_texture(&self) -> Option<Id8> {
		read_id8(self.tex_top)
	}

	/// Returns `None` if the first byte of the middle texture field is NUL.
	#[must_use]
	pub fn mid_texture(&self) -> Option<Id8> {
		read_id8(self.tex_mid)
	}

	/// Returns `None` if the first byte of the bottom texture field is NUL.
	#[must_use]
	pub fn bottom_texture(&self) -> Option<Id8> {
		read_id8(self.tex_bottom)
	}
}

/// Casts a slice of raw bytes to side definitions (without allocating).
/// Returns [`Error::MalformedFile`] if the length of `lump` is not divisible by 30.
pub fn sidedefs(lump: &[u8]) -> Result<&[SideDefRaw], Error> {
	if (lump.len() % std::mem::size_of::<SideDefRaw>()) != 0 {
		return Err(Error::MalformedFile("SIDEDEFS"));
	}

	Ok(bytemuck::cast_slice(lump))
}

/// See [`sidedefs`].
pub fn sidedefs_mut(lump: &mut [u8]) -> Result<&mut [SideDefRaw], Error> {
	if (lump.len() % std::mem::size_of::<SideDefRaw>()) != 0 {
		return Err(Error::MalformedFile("SIDEDEFS"));
	}

	Ok(bytemuck::cast_slice_mut(lump))
}

/// Like [`sidedefs`], but any bytes at the end of slice which do not fit into
/// another [`SideDefRaw`] are truncated.
#[must_use]
pub fn sidedefs_lossy(lump: &[u8]) -> &[SideDefRaw] {
	let sz = std::mem::size_of::<SideDefRaw>();
	let count = lump.len() / sz;
	let subslice = &lump[..(count * sz)];
	bytemuck::cast_slice(subslice)
}

/// Like [`sidedefs_mut`], but any bytes at the end of slice which do not fit into
/// another [`SideDefRaw`] are truncated.
#[must_use]
pub fn sidedefs_lossy_mut(lump: &mut [u8]) -> &mut [SideDefRaw] {
	let sz = std::mem::size_of::<SideDefRaw>();
	let count = lump.len() / sz;
	let subslice = &mut lump[..(count * sz)];
	bytemuck::cast_slice_mut(subslice)
}

// SSECTORS ////////////////////////////////////////////////////////////////////

/// See <https://doomwiki.org/wiki/Subsector>. Acquired via [`ssectors`].
/// These are cast directly from the bytes of a WAD's lump;
/// attached methods automatically convert from Little Endian.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::Zeroable, bytemuck::Pod)]
pub struct SSectorRaw {
	seg_count: u16,
	seg: u16,
}

impl SSectorRaw {
	#[must_use]
	pub fn seg_count(self) -> u16 {
		u16::from_le(self.seg_count)
	}

	/// To be used as an index into a slice of [`SegRaw`].
	#[must_use]
	pub fn first_seg(self) -> u16 {
		u16::from_le(self.seg)
	}

	/// To be used to take a part of a slice of [`SegRaw`].
	#[must_use]
	pub fn segs(self) -> Range<usize> {
		let seg0 = self.first_seg() as usize;
		seg0..(seg0 + self.seg_count() as usize)
	}
}

/// Casts a slice of raw bytes to sub-sector definitions (without allocating).
/// Returns [`Error::MalformedFile`] if the length of `lump` is not divisible by 4.
pub fn ssectors(lump: &[u8]) -> Result<&[SSectorRaw], Error> {
	if (lump.len() % std::mem::size_of::<SSectorRaw>()) != 0 {
		return Err(Error::MalformedFile("SSECTORS"));
	}

	Ok(bytemuck::cast_slice(lump))
}

/// See [`ssectors`].
pub fn ssectors_mut(lump: &mut [u8]) -> Result<&mut [SSectorRaw], Error> {
	if (lump.len() % std::mem::size_of::<SSectorRaw>()) != 0 {
		return Err(Error::MalformedFile("SSECTORS"));
	}

	Ok(bytemuck::cast_slice_mut(lump))
}

/// Like [`ssectors`], but any bytes at the end of slice which do not fit into
/// another [`SSectorRaw`] are truncated.
#[must_use]
pub fn ssectors_lossy(lump: &[u8]) -> &[SSectorRaw] {
	let sz = std::mem::size_of::<SSectorRaw>();
	let count = lump.len() / sz;
	let subslice = &lump[..(count * sz)];
	bytemuck::cast_slice(subslice)
}

/// Like [`ssectors_mut`], but any bytes at the end of slice which do not fit into
/// another [`SSectorRaw`] are truncated.
#[must_use]
pub fn ssectors_lossy_mut(lump: &mut [u8]) -> &mut [SSectorRaw] {
	let sz = std::mem::size_of::<SSectorRaw>();
	let count = lump.len() / sz;
	let subslice = &mut lump[..(count * sz)];
	bytemuck::cast_slice_mut(subslice)
}

// THINGS //////////////////////////////////////////////////////////////////////

/// See <https://doomwiki.org/wiki/Thing>. Acquired via [`things`].
/// These are cast directly from the bytes of a WAD's lump;
/// attached methods automatically convert from Little Endian.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::Zeroable, bytemuck::Pod)]
pub struct ThingRaw {
	x: i16,
	y: i16,
	angle: u16,
	ednum: u16,
	flags: i16,
}

impl ThingRaw {
	#[must_use]
	pub fn position(&self) -> [i16; 2] {
		[i16::from_le(self.x), i16::from_le(self.y)]
	}

	#[must_use]
	pub fn editor_num(&self) -> u16 {
		u16::from_le(self.ednum)
	}

	/// In degrees. 0 is east, north is 90, et cetera.
	#[must_use]
	pub fn angle(&self) -> u16 {
		u16::from_le(self.angle)
	}

	#[must_use]
	pub fn flags(&self) -> ThingFlags {
		let f = i16::from_le(self.flags);
		let mut flags = ThingFlags::empty();

		// TODO: Strife thing flag support.

		if (f & (1 << 0)) != 0 {
			flags.insert(ThingFlags::SKILL_1 | ThingFlags::SKILL_2);
		}

		if (f & (1 << 1)) != 0 {
			flags.insert(ThingFlags::SKILL_3);
		}

		if (f & (1 << 2)) != 0 {
			flags.insert(ThingFlags::SKILL_4 | ThingFlags::SKILL_5);
		}

		if (f & (1 << 3)) != 0 {
			flags.insert(ThingFlags::AMBUSH);
		}

		if (f & (1 << 4)) != 0 {
			flags.insert(ThingFlags::COOP);
		} else {
			flags.insert(ThingFlags::SINGLEPLAY);
		}

		if (f & (1 << 5)) != 0 {
			flags.remove(ThingFlags::DEATHMATCH);
		}

		if (f & (1 << 6)) != 0 {
			flags.remove(ThingFlags::COOP);
		}

		if (f & (1 << 7)) != 0 {
			flags.insert(ThingFlags::FRIEND);
		}

		flags
	}
}

bitflags::bitflags! {
	/// See [`ThingRaw`] and [`ThingExtRaw`].
	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	pub struct ThingFlags: u16 {
		const SKILL_1 = 1 << 0;
		const SKILL_2 = 1 << 1;
		const SKILL_3 = 1 << 2;
		const SKILL_4 = 1 << 3;
		const SKILL_5 = 1 << 4;
		/// Alternatively "deaf", but not in terms of sound propagation.
		const AMBUSH = 1 << 5;
		const SINGLEPLAY = 1 << 6;
		const DEATHMATCH = 1 << 7;
		const COOP = 1 << 8;
		const FRIEND = 1 << 9;
		const DORMANT = 1 << 10;
		/// If unset, this is absent to e.g. Hexen's Fighter class.
		const CLASS_1 = 1 << 11;
		/// If unset, this is absent to e.g. Hexen's Cleric class.
		const CLASS_2 = 1 << 12;
		/// If unset, this is absent to e.g. Hexen's Mage class.
		const CLASS_3 = 1 << 13;
	}
}

/// Casts a slice of raw bytes to thing definitions (without allocating).
/// Returns [`Error::MalformedFile`] if the length of `lump` is not divisible by 10.
pub fn things(lump: &[u8]) -> Result<&[ThingRaw], Error> {
	if (lump.len() % std::mem::size_of::<ThingRaw>()) != 0 {
		return Err(Error::MalformedFile("THINGS"));
	}

	Ok(bytemuck::cast_slice(lump))
}

/// See [`things`].
pub fn things_mut(lump: &mut [u8]) -> Result<&mut [ThingRaw], Error> {
	if (lump.len() % std::mem::size_of::<ThingRaw>()) != 0 {
		return Err(Error::MalformedFile("THINGS"));
	}

	Ok(bytemuck::cast_slice_mut(lump))
}

/// Like [`things`], but any bytes at the end of slice which do not fit into
/// another [`ThingRaw`] are truncated.
#[must_use]
pub fn things_lossy(lump: &[u8]) -> &[ThingRaw] {
	let sz = std::mem::size_of::<ThingRaw>();
	let count = lump.len() / sz;
	let subslice = &lump[..(count * sz)];
	bytemuck::cast_slice(subslice)
}

/// Like [`things_mut`], but any bytes at the end of slice which do not fit into
/// another [`ThingRaw`] are truncated.
#[must_use]
pub fn things_lossy_mut(lump: &mut [u8]) -> &mut [ThingRaw] {
	let sz = std::mem::size_of::<ThingRaw>();
	let count = lump.len() / sz;
	let subslice = &mut lump[..(count * sz)];
	bytemuck::cast_slice_mut(subslice)
}

// THINGS, extended ////////////////////////////////////////////////////////////

/// See <https://doomwiki.org/wiki/Thing#Hexen_format>. Acquired via [`things`].
/// These are cast directly from the bytes of a WAD's lump;
/// attached methods automatically convert from Little Endian.
///
/// Please note that due to padding, this type cannot be cast to mutable slices.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
pub struct ThingExtRaw {
	tid: i16,
	x: i16,
	y: i16,
	z: i16,
	angle: u16,
	ednum: u16,
	flags: i16,
	args: [u8; 5],
}

impl ThingExtRaw {
	/// Returns, in order, X, Y, and Z coordinates.
	#[must_use]
	pub fn position(&self) -> [i16; 3] {
		[
			i16::from_le(self.x),
			i16::from_le(self.y),
			i16::from_le(self.z),
		]
	}

	#[must_use]
	pub fn editor_num(&self) -> u16 {
		u16::from_le(self.ednum)
	}

	/// In degrees. 0 is east, north is 90, et cetera.
	#[must_use]
	pub fn angle(&self) -> u16 {
		u16::from_le(self.angle)
	}

	#[must_use]
	pub fn flags(&self) -> ThingFlags {
		let f = i16::from_le(self.flags);
		let mut flags = ThingFlags::empty();

		if (f & (1 << 0)) != 0 {
			flags.insert(ThingFlags::SKILL_1 | ThingFlags::SKILL_2);
		}

		if (f & (1 << 1)) != 0 {
			flags.insert(ThingFlags::SKILL_3);
		}

		if (f & (1 << 2)) != 0 {
			flags.insert(ThingFlags::SKILL_4 | ThingFlags::SKILL_5);
		}

		if (f & (1 << 3)) != 0 {
			flags.insert(ThingFlags::AMBUSH);
		}

		if (f & (1 << 4)) != 0 {
			flags.insert(ThingFlags::DORMANT);
		}

		if (f & (1 << 5)) != 0 {
			flags.insert(ThingFlags::CLASS_1);
		}

		if (f & (1 << 6)) != 0 {
			flags.insert(ThingFlags::CLASS_2);
		}

		if (f & (1 << 7)) != 0 {
			flags.insert(ThingFlags::CLASS_3);
		}

		if (f & (1 << 8)) != 0 {
			flags.insert(ThingFlags::SINGLEPLAY);
		}

		if (f & (1 << 9)) != 0 {
			flags.insert(ThingFlags::COOP);
		}

		if (f & (1 << 10)) != 0 {
			flags.insert(ThingFlags::DEATHMATCH);
		}

		flags
	}

	#[must_use]
	pub fn args(&self) -> [u8; 5] {
		self.args
	}
}

/// Casts a slice of raw bytes to extended thing definitions (without allocating).
/// Returns [`Error::MalformedFile`] if the length of `lump` is not divisible by 160.
pub fn things_ext(lump: &[u8]) -> Result<&[ThingExtRaw], Error> {
	if (lump.len() % std::mem::size_of::<ThingExtRaw>()) != 0 {
		return Err(Error::MalformedFile("THINGS (extended)"));
	}

	Ok(bytemuck::cast_slice(lump))
}

/// Like [`things_ext`], but any bytes at the end of slice which do not fit into
/// another [`ThingExtRaw`] are truncated.
#[must_use]
pub fn things_ext_lossy(lump: &[u8]) -> &[ThingExtRaw] {
	let sz = std::mem::size_of::<ThingExtRaw>();
	let count = lump.len() / sz;
	let subslice = &lump[..(count * sz)];
	bytemuck::cast_slice(subslice)
}

// VERTEXES ////////////////////////////////////////////////////////////////////

/// See <https://doomwiki.org/wiki/Vertex>. Acquired via [`vertexes`].
/// These are cast directly from the bytes of a WAD's lump;
/// attached methods automatically convert from Little Endian.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::Zeroable, bytemuck::Pod)]
pub struct VertexRaw {
	x: i16,
	y: i16,
}

impl VertexRaw {
	/// Returns a "minimum" and "maximum" corner, respectively.
	#[must_use]
	pub fn bounds(verts: &[Self]) -> ([i16; 2], [i16; 2]) {
		let mut min = [0, 0];
		let mut max = [0, 0];

		for vert in verts {
			if vert.x < min[0] {
				min[0] = vert.x;
			} else if vert.x > max[0] {
				max[0] = vert.x;
			}

			if vert.y < min[1] {
				min[1] = vert.y;
			} else if vert.y > max[1] {
				max[1] = vert.y;
			}
		}

		(min, max)
	}

	#[must_use]
	pub fn position(&self) -> [i16; 2] {
		[i16::from_le(self.x), i16::from_le(self.y)]
	}
}

/// Casts a slice of raw bytes to vertex definitions (without allocating).
/// Returns [`Error::MalformedFile`] if the length of `lump` is not divisible by 10.
pub fn vertexes(lump: &[u8]) -> Result<&[VertexRaw], Error> {
	if (lump.len() % std::mem::size_of::<VertexRaw>()) != 0 {
		return Err(Error::MalformedFile("VERTEXES"));
	}

	Ok(bytemuck::cast_slice(lump))
}

/// See [`vertexes`].
pub fn vertexes_mut(lump: &mut [u8]) -> Result<&mut [VertexRaw], Error> {
	if (lump.len() % std::mem::size_of::<VertexRaw>()) != 0 {
		return Err(Error::MalformedFile("VERTEXES"));
	}

	Ok(bytemuck::cast_slice_mut(lump))
}

/// Like [`vertexes`], but any bytes at the end of slice which do not fit into
/// another [`VertexRaw`] are truncated.
#[must_use]
pub fn vertexes_lossy(lump: &[u8]) -> &[VertexRaw] {
	let sz = std::mem::size_of::<VertexRaw>();
	let count = lump.len() / sz;
	let subslice = &lump[..(count * sz)];
	bytemuck::cast_slice(subslice)
}

/// Like [`vertexes_mut`], but any bytes at the end of slice which do not fit into
/// another [`VertexRaw`] are truncated.
#[must_use]
pub fn vertexes_lossy_mut(lump: &mut [u8]) -> &mut [VertexRaw] {
	let sz = std::mem::size_of::<VertexRaw>();
	let count = lump.len() / sz;
	let subslice = &mut lump[..(count * sz)];
	bytemuck::cast_slice_mut(subslice)
}
