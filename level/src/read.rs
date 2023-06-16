//! Functions for turning vanilla ["map lumps"] into levels.
//!
//! ["map lumps"]: https://doomwiki.org/wiki/Lump#Standard_lumps

use std::{collections::HashMap, io::Cursor, mem::MaybeUninit};

use rayon::prelude::*;
use util::{io::CursorExt, read_id8};

use crate::{
	repr::{
		BspNode, BspNodeChild, LineDef, LineFlags, SectorDef, Seg, SegDirection, SideDef,
		SubSector, ThingDef, ThingFlags, Vertex,
	},
	Error, VANILLA_SCALEDOWN,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
struct LineDefRaw {
	v_start: u16,
	v_end: u16,
	flags: i16,
	special: u16,
	trigger: u16,
	right: u16,
	left: u16,
}

impl LineDefRaw {
	const SIZE: usize = std::mem::size_of::<Self>();
}

impl From<&LineDefRaw> for LineDef {
	fn from(value: &LineDefRaw) -> Self {
		Self {
			udmf_id: -1,
			vert_start: u16::from_le(value.v_start) as usize,
			vert_end: u16::from_le(value.v_end) as usize,
			flags: LineFlags::from_bits_truncate(value.flags as u32),
			special: u16::from_le(value.special) as i32,
			trigger: u16::from_le(value.trigger),
			args: [0; 5],
			side_right: u16::from_le(value.right) as usize,
			side_left: {
				let s = u16::from_le(value.left);

				if s == 0xFFFF {
					None
				} else {
					Some(s as usize)
				}
			},
			udmf: HashMap::default(),
		}
	}
}

/// Returns [`Error::MalformedFile`] if the length `bytes` is not divisible by 14.
pub fn linedefs(bytes: &[u8]) -> Result<Vec<LineDef>, Error> {
	if (bytes.len() % LineDefRaw::SIZE) != 0 {
		return Err(Error::MalformedFile("LINEDEFS"));
	}

	let mut ret = Vec::with_capacity(bytes.len() / LineDefRaw::SIZE);
	let mut cursor = Cursor::new(bytes);

	for _ in 0..(bytes.len() / LineDefRaw::SIZE) {
		let raw = cursor.read_from_bytes::<LineDefRaw>();
		ret.push(LineDef::from(raw));
	}

	Ok(ret)
}

/// Same as [`linedefs`], but uses [`rayon`]'s global thread pool.
pub fn linedefs_par(bytes: &[u8]) -> Result<Vec<LineDef>, Error> {
	if (bytes.len() % LineDefRaw::SIZE) != 0 {
		return Err(Error::MalformedFile("LINEDEFS"));
	}

	let mut ret = Vec::<MaybeUninit<LineDef>>::new();
	ret.resize_with(bytes.len() / LineDefRaw::SIZE, MaybeUninit::uninit);

	ret.par_iter_mut().enumerate().for_each(|(i, linedef)| {
		let range = (i * LineDefRaw::SIZE)..(LineDefRaw::SIZE + (i * LineDefRaw::SIZE));
		let raw = bytemuck::from_bytes::<LineDefRaw>(&bytes[range]);
		linedef.write(LineDef::from(raw));
	});

	// SAFETY: `MaybeUninit<T>` is `repr(transparent)` over `T`.
	Ok(unsafe { std::mem::transmute::<_, _>(ret) })
}

// NODES ///////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
struct NodeRaw {
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
	const SIZE: usize = std::mem::size_of::<Self>();
}

impl From<&NodeRaw> for BspNode {
	fn from(value: &NodeRaw) -> Self {
		let child_r = i16::from_le(value.child_r);
		let child_l = i16::from_le(value.child_l);

		let start = glam::vec2(
			-((i16::from_le(value.y) as f32) * VANILLA_SCALEDOWN),
			-((i16::from_le(value.x) as f32) * VANILLA_SCALEDOWN),
		);

		Self {
			seg_start: start,
			seg_end: start
				+ glam::vec2(
					-((i16::from_le(value.delta_y) as f32) * VANILLA_SCALEDOWN),
					-((i16::from_le(value.delta_x) as f32) * VANILLA_SCALEDOWN),
				),
			child_r: if child_r.is_negative() {
				BspNodeChild::SubSector((child_r & 0x7FFF) as usize)
			} else {
				BspNodeChild::SubNode(child_r as usize)
			},
			child_l: if child_l.is_negative() {
				BspNodeChild::SubSector((child_l & 0x7FFF) as usize)
			} else {
				BspNodeChild::SubNode(child_l as usize)
			},
		}
	}
}

/// Returns [`Error::MalformedFile`] if the length of `bytes` is not divisible by 28.
pub fn nodes(bytes: &[u8]) -> Result<Vec<BspNode>, Error> {
	if (bytes.len() % NodeRaw::SIZE) != 0 {
		return Err(Error::MalformedFile("NODES"));
	}

	let mut ret = Vec::with_capacity(bytes.len() / NodeRaw::SIZE);
	let mut cursor = Cursor::new(bytes);

	for _ in 0..(bytes.len() / NodeRaw::SIZE) {
		let raw = cursor.read_from_bytes::<NodeRaw>();
		ret.push(BspNode::from(raw));
	}

	Ok(ret)
}

/// Same as [`nodes`], but uses [`rayon`]'s global thread pool.
pub fn nodes_par(bytes: &[u8]) -> Result<Vec<BspNode>, Error> {
	if (bytes.len() % NodeRaw::SIZE) != 0 {
		return Err(Error::MalformedFile("NODES"));
	}

	let mut ret = Vec::<MaybeUninit<BspNode>>::new();
	ret.resize_with(bytes.len() / NodeRaw::SIZE, MaybeUninit::uninit);

	ret.par_iter_mut().enumerate().for_each(|(i, node)| {
		let range = (i * NodeRaw::SIZE)..(NodeRaw::SIZE + (i * NodeRaw::SIZE));
		let raw = bytemuck::from_bytes::<NodeRaw>(&bytes[range]);
		node.write(BspNode::from(raw));
	});

	// SAFETY: `MaybeUninit<T>` is `repr(transparent)` over `T`.
	Ok(unsafe { std::mem::transmute::<_, _>(ret) })
}

// SECTORS /////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
struct SectorRaw {
	height_floor: i16,
	height_ceil: i16,
	tex_floor: [u8; 8],
	tex_ceil: [u8; 8],
	light_level: u16,
	special: u16,
	trigger: u16,
}

impl SectorRaw {
	const SIZE: usize = std::mem::size_of::<Self>();
}

impl From<&SectorRaw> for SectorDef {
	fn from(value: &SectorRaw) -> Self {
		SectorDef {
			udmf_id: 0,
			height_floor: (i16::from_le(value.height_floor) as f32) * VANILLA_SCALEDOWN,
			height_ceil: (i16::from_le(value.height_ceil) as f32) * VANILLA_SCALEDOWN,
			tex_floor: read_id8(value.tex_floor),
			tex_ceil: read_id8(value.tex_ceil),
			light_level: u16::from_le(value.light_level) as i32,
			special: value.special as i32,
			trigger: u16::from_le(value.trigger),
			udmf: HashMap::default(),
		}
	}
}

/// Returns [`Error::MalformedFile`] if the length of `bytes` is not divisible by 26.
pub fn sectors(bytes: &[u8]) -> Result<Vec<SectorDef>, Error> {
	if (bytes.len() % SectorRaw::SIZE) != 0 {
		return Err(Error::MalformedFile("SECTORS"));
	}

	let mut ret = Vec::with_capacity(bytes.len() / SectorRaw::SIZE);
	let mut cursor = Cursor::new(bytes);

	for _ in 0..(bytes.len() / SectorRaw::SIZE) {
		let raw = cursor.read_from_bytes::<SectorRaw>();
		ret.push(SectorDef::from(raw));
	}

	Ok(ret)
}

/// Same as [`sectors`], but uses [`rayon`]'s global thread pool.
pub fn sectors_par(bytes: &[u8]) -> Result<Vec<SectorDef>, Error> {
	if (bytes.len() % SectorRaw::SIZE) != 0 {
		return Err(Error::MalformedFile("SECTORS"));
	}

	let mut ret = Vec::<MaybeUninit<SectorDef>>::new();
	ret.resize_with(bytes.len() / SectorRaw::SIZE, MaybeUninit::uninit);

	ret.par_iter_mut().enumerate().for_each(|(i, sectordef)| {
		let range = (i * SectorRaw::SIZE)..(SectorRaw::SIZE + (i * SectorRaw::SIZE));
		let raw = bytemuck::from_bytes::<SectorRaw>(&bytes[range]);
		sectordef.write(SectorDef::from(raw));
	});

	// SAFETY: `MaybeUninit<T>` is `repr(transparent)` over `T`.
	Ok(unsafe { std::mem::transmute::<_, _>(ret) })
}

// SEGS ////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
struct SegRaw {
	v_start: u16,
	v_end: u16,
	angle: i16,
	linedef: u16,
	direction: i16,
	offset: i16,
}

impl SegRaw {
	const SIZE: usize = std::mem::size_of::<Self>();
}

impl From<&SegRaw> for Seg {
	fn from(value: &SegRaw) -> Self {
		Seg {
			vert_start: u16::from_le(value.v_start) as usize,
			vert_end: u16::from_le(value.v_end) as usize,
			angle: i16::from_le(value.angle),
			linedef: u16::from_le(value.linedef) as usize,
			direction: if i16::from_le(value.direction) == 0 {
				SegDirection::Front
			} else {
				SegDirection::Back
			},
			offset: i16::from_le(value.offset),
		}
	}
}

/// Returns [`Error::MalformedFile`] if the length of `bytes` is not divisible by 12.
pub fn segs(bytes: &[u8]) -> Result<Vec<Seg>, Error> {
	if (bytes.len() % SegRaw::SIZE) != 0 {
		return Err(Error::MalformedFile("SEGS"));
	}

	let mut ret = Vec::with_capacity(bytes.len() / SegRaw::SIZE);
	let mut cursor = Cursor::new(bytes);

	for _ in 0..(bytes.len() / SegRaw::SIZE) {
		let raw = cursor.read_from_bytes::<SegRaw>();
		ret.push(Seg::from(raw));
	}

	Ok(ret)
}

/// Same as [`segs`], but uses [`rayon`]'s global thread pool.
pub fn segs_par(bytes: &[u8]) -> Result<Vec<Seg>, Error> {
	if (bytes.len() % SegRaw::SIZE) != 0 {
		return Err(Error::MalformedFile("SEGS"));
	}

	let mut ret = Vec::<MaybeUninit<Seg>>::new();
	ret.resize_with(bytes.len() / SegRaw::SIZE, MaybeUninit::uninit);

	ret.par_iter_mut().enumerate().for_each(|(i, seg)| {
		let range = (i * SegRaw::SIZE)..(SegRaw::SIZE + (i * SegRaw::SIZE));
		let raw = bytemuck::from_bytes::<SegRaw>(&bytes[range]);
		seg.write(Seg::from(raw));
	});

	// SAFETY: `MaybeUninit<T>` is `repr(transparent)` over `T`.
	Ok(unsafe { std::mem::transmute::<_, _>(ret) })
}

// SIDEDEFS ////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
struct SideDefRaw {
	offs_x: i16,
	offs_y: i16,
	tex_top: [u8; 8],
	tex_bottom: [u8; 8],
	tex_mid: [u8; 8],
	sector: u16,
}

impl SideDefRaw {
	const SIZE: usize = std::mem::size_of::<Self>();
}

impl From<&SideDefRaw> for SideDef {
	fn from(value: &SideDefRaw) -> Self {
		SideDef {
			offset: glam::ivec2(
				i16::from_le(value.offs_x) as i32,
				i16::from_le(value.offs_y) as i32,
			),
			tex_top: read_id8(value.tex_top).filter(|id8| id8 != "-"),
			tex_bottom: read_id8(value.tex_bottom).filter(|id8| id8 != "-"),
			tex_mid: read_id8(value.tex_mid).filter(|id8| id8 != "-"),
			sector: u16::from_le(value.sector) as usize,
			udmf: HashMap::default(),
		}
	}
}

/// Returns [`Error::MalformedFile`] if the length of `bytes` is not divisible by 30.
pub fn sidedefs(bytes: &[u8]) -> Result<Vec<SideDef>, Error> {
	if (bytes.len() % SideDefRaw::SIZE) != 0 {
		return Err(Error::MalformedFile("SIDEDEFS"));
	}

	let mut ret = Vec::with_capacity(bytes.len() / SideDefRaw::SIZE);
	let mut cursor = Cursor::new(bytes);

	for _ in 0..(bytes.len() / SideDefRaw::SIZE) {
		let raw = cursor.read_from_bytes::<SideDefRaw>();
		ret.push(SideDef::from(raw));
	}

	Ok(ret)
}

/// Same as [`sidedefs`], but uses [`rayon`]'s global thread pool.
pub fn sidedefs_par(bytes: &[u8]) -> Result<Vec<SideDef>, Error> {
	if (bytes.len() % SideDefRaw::SIZE) != 0 {
		return Err(Error::MalformedFile("SIDEDEFS"));
	}

	let mut ret = Vec::<MaybeUninit<SideDef>>::new();
	ret.resize_with(bytes.len() / SideDefRaw::SIZE, MaybeUninit::uninit);

	ret.par_iter_mut().enumerate().for_each(|(i, sidedef)| {
		let range = (i * SideDefRaw::SIZE)..(SideDefRaw::SIZE + (i * SideDefRaw::SIZE));
		let raw = bytemuck::from_bytes::<SideDefRaw>(&bytes[range]);
		sidedef.write(SideDef::from(raw));
	});

	// SAFETY: `MaybeUninit<T>` is `repr(transparent)` over `T`.
	Ok(unsafe { std::mem::transmute::<_, _>(ret) })
}

// SSECTORS ////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
struct SSectorRaw {
	seg_count: u16,
	seg: u16,
}

impl SSectorRaw {
	const SIZE: usize = std::mem::size_of::<Self>();
}

impl From<&SSectorRaw> for SubSector {
	fn from(value: &SSectorRaw) -> Self {
		SubSector {
			seg_count: u16::from_le(value.seg_count) as usize,
			seg0: u16::from_le(value.seg) as usize,
		}
	}
}

/// Returns [`Error::MalformedFile`] if the length of `bytes` is not divisible by 4.
pub fn ssectors(bytes: &[u8]) -> Result<Vec<SubSector>, Error> {
	if (bytes.len() % SSectorRaw::SIZE) != 0 {
		return Err(Error::MalformedFile("SSECTORS"));
	}

	let mut ret = Vec::with_capacity(bytes.len() / SSectorRaw::SIZE);
	let mut cursor = Cursor::new(bytes);

	for _ in 0..(bytes.len() / SSectorRaw::SIZE) {
		let raw = cursor.read_from_bytes::<SSectorRaw>();
		ret.push(SubSector::from(raw));
	}

	Ok(ret)
}

/// Same as [`ssectors`], but uses [`rayon`]'s global thread pool.
pub fn ssectors_par(bytes: &[u8]) -> Result<Vec<SubSector>, Error> {
	if (bytes.len() % SSectorRaw::SIZE) != 0 {
		return Err(Error::MalformedFile("SSECTORS"));
	}

	let mut ret = Vec::<MaybeUninit<SubSector>>::new();
	ret.resize_with(bytes.len() / SSectorRaw::SIZE, MaybeUninit::uninit);

	ret.par_iter_mut().enumerate().for_each(|(i, subsector)| {
		let range = (i * SSectorRaw::SIZE)..(SSectorRaw::SIZE + (i * SSectorRaw::SIZE));
		let raw = bytemuck::from_bytes::<SSectorRaw>(&bytes[range]);
		subsector.write(SubSector::from(raw));
	});

	// SAFETY: `MaybeUninit<T>` is `repr(transparent)` over `T`.
	Ok(unsafe { std::mem::transmute::<_, _>(ret) })
}

// THINGS //////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
struct ThingRaw {
	x: i16,
	y: i16,
	angle: u16,
	ednum: u16,
	flags: i16,
}

impl ThingRaw {
	const SIZE: usize = std::mem::size_of::<Self>();
}

impl From<&ThingRaw> for ThingDef {
	fn from(value: &ThingRaw) -> Self {
		ThingDef {
			tid: 0,
			ed_num: u16::from_le(value.ednum),
			pos: glam::vec3(
				(i16::from_le(value.x) as f32) * VANILLA_SCALEDOWN,
				0.0,
				(i16::from_le(value.y) as f32) * VANILLA_SCALEDOWN,
			),
			angle: u16::from_le(value.angle) as u32,
			flags: {
				let f = i16::from_le(value.flags);
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
			},
			special: 0,
			args: [0, 0, 0, 0, 0],
			udmf: HashMap::default(),
		}
	}
}

/// Returns [`Error::MalformedFile`] if the length of `bytes` is not divisible by 10.
pub fn things_doom(bytes: &[u8]) -> Result<Vec<ThingDef>, Error> {
	if (bytes.len() % ThingRaw::SIZE) != 0 {
		return Err(Error::MalformedFile("THINGS"));
	}

	let mut ret = Vec::with_capacity(bytes.len() / ThingRaw::SIZE);
	let mut cursor = Cursor::new(bytes);

	for _ in 0..(bytes.len() / ThingRaw::SIZE) {
		let raw = cursor.read_from_bytes::<ThingRaw>();
		ret.push(ThingDef::from(raw));
	}

	Ok(ret)
}

/// Same as [`things_doom`], but uses [`rayon`]'s global thread pool.
pub fn things_doom_par(bytes: &[u8]) -> Result<Vec<ThingDef>, Error> {
	if (bytes.len() % ThingRaw::SIZE) != 0 {
		return Err(Error::MalformedFile("THINGS"));
	}

	let mut ret = Vec::<MaybeUninit<ThingDef>>::new();
	ret.resize_with(bytes.len() / ThingRaw::SIZE, MaybeUninit::uninit);

	ret.par_iter_mut().enumerate().for_each(|(i, thingdef)| {
		let range = (i * ThingRaw::SIZE)..(ThingRaw::SIZE + (i * ThingRaw::SIZE));
		let raw = bytemuck::from_bytes::<ThingRaw>(&bytes[range]);
		thingdef.write(ThingDef::from(raw));
	});

	// SAFETY: `MaybeUninit<T>` is `repr(transparent)` over `T`.
	Ok(unsafe { std::mem::transmute::<_, _>(ret) })
}

// THINGS (extended) ///////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
struct ThingExtRaw {
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
	const SIZE: usize = std::mem::size_of::<Self>();
}

impl From<&ThingExtRaw> for ThingDef {
	fn from(value: &ThingExtRaw) -> Self {
		ThingDef {
			tid: i16::from_le(value.tid) as i32,
			ed_num: u16::from_le(value.ednum),
			pos: glam::vec3(
				(i16::from_le(value.x) as f32) * VANILLA_SCALEDOWN,
				(i16::from_le(value.z) as f32) * VANILLA_SCALEDOWN,
				(i16::from_le(value.y) as f32) * VANILLA_SCALEDOWN,
			),
			angle: u16::from_le(value.angle) as u32,
			flags: {
				let f = i16::from_le(value.flags);
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
			},
			special: 0,
			args: [
				value.args[0] as i32,
				value.args[1] as i32,
				value.args[2] as i32,
				value.args[3] as i32,
				value.args[4] as i32,
			],
			udmf: HashMap::default(),
		}
	}
}

/// Returns [`Error::MalformedFile`] if the length of `bytes` is not divisible by 20.
pub fn things_extended(bytes: &[u8]) -> Result<Vec<ThingDef>, Error> {
	if (bytes.len() % ThingExtRaw::SIZE) != 0 {
		return Err(Error::MalformedFile("THINGS (extended)"));
	}

	let mut ret = Vec::with_capacity(bytes.len() / ThingExtRaw::SIZE);
	let mut cursor = Cursor::new(bytes);

	for _ in 0..(bytes.len() / ThingExtRaw::SIZE) {
		let raw = cursor.read_from_bytes::<ThingExtRaw>();
		ret.push(ThingDef::from(raw));
	}

	Ok(ret)
}

/// Same as [`things_extended`], but uses [`rayon`]'s global thread pool.
pub fn things_extended_par(bytes: &[u8]) -> Result<Vec<ThingDef>, Error> {
	if (bytes.len() % ThingExtRaw::SIZE) != 0 {
		return Err(Error::MalformedFile("THINGS (extended)"));
	}

	let mut ret = Vec::<MaybeUninit<ThingDef>>::new();
	ret.resize_with(bytes.len() / ThingExtRaw::SIZE, MaybeUninit::uninit);

	ret.par_iter_mut().enumerate().for_each(|(i, thingdef)| {
		let range = (i * ThingExtRaw::SIZE)..(ThingExtRaw::SIZE + (i * ThingExtRaw::SIZE));
		let raw = bytemuck::from_bytes::<ThingExtRaw>(&bytes[range]);
		thingdef.write(ThingDef::from(raw));
	});

	// SAFETY: `MaybeUninit<T>` is `repr(transparent)` over `T`.
	Ok(unsafe { std::mem::transmute::<_, _>(ret) })
}

// VERTEXES ////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
struct VertexRaw {
	x: i16,
	y: i16,
}

impl VertexRaw {
	const SIZE: usize = std::mem::size_of::<Self>();
}

impl From<&VertexRaw> for Vertex {
	fn from(value: &VertexRaw) -> Self {
		Vertex(glam::vec4(
			(i16::from_le(value.x) as f32) * VANILLA_SCALEDOWN,
			0.0,
			(i16::from_le(value.y) as f32) * VANILLA_SCALEDOWN,
			0.0,
		))
	}
}

/// Returns [`Error::MalformedFile`] if the length of `bytes` is not divisible by 4.
pub fn vertexes(bytes: &[u8]) -> Result<Vec<Vertex>, Error> {
	if (bytes.len() % VertexRaw::SIZE) != 0 {
		return Err(Error::MalformedFile("VERTEXES"));
	}

	let mut ret = Vec::with_capacity(bytes.len() / VertexRaw::SIZE);
	let mut cursor = Cursor::new(bytes);

	for _ in 0..(bytes.len() / VertexRaw::SIZE) {
		let raw = cursor.read_from_bytes::<VertexRaw>();
		ret.push(Vertex::from(raw));
	}

	Ok(ret)
}

/// Same as [`vertexes`], but uses [`rayon`]'s global thread pool.
pub fn vertexes_par(bytes: &[u8]) -> Result<Vec<Vertex>, Error> {
	if (bytes.len() % VertexRaw::SIZE) != 0 {
		return Err(Error::MalformedFile("VERTEXES"));
	}

	let mut ret = Vec::<MaybeUninit<Vertex>>::new();
	ret.resize_with(bytes.len() / VertexRaw::SIZE, MaybeUninit::uninit);

	ret.par_iter_mut().enumerate().for_each(|(i, vertex)| {
		let range = (i * VertexRaw::SIZE)..(VertexRaw::SIZE + (i * VertexRaw::SIZE));
		let raw = bytemuck::from_bytes::<VertexRaw>(&bytes[range]);
		vertex.write(Vertex::from(raw));
	});

	// SAFETY: `MaybeUninit<T>` is `repr(transparent)` over `T`.
	Ok(unsafe { std::mem::transmute::<_, _>(ret) })
}
