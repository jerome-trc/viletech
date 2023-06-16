//! Functions for turning vanilla ["map lumps"] into levels.
//!
//! ["map lumps"]: https://doomwiki.org/wiki/Lump#Standard_lumps

use std::{collections::HashMap, io::Cursor};

use util::{io::CursorExt, read_id8};

use crate::{
	repr::{
		BspNode, BspNodeChild, LineDef, LineFlags, SectorDef, Seg, SegDirection, SideDef,
		SubSector, ThingDef, ThingFlags, Vertex,
	},
	Error, VANILLA_SCALEDOWN,
};

/// Returns [`Error::MalformedFile`] if the length `bytes` is not divisible by 14.
pub fn linedefs(bytes: &[u8]) -> Result<Vec<LineDef>, Error> {
	#[repr(C)]
	#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
	struct LinedefRaw {
		v_start: u16,
		v_end: u16,
		flags: i16,
		special: u16,
		trigger: u16,
		right: u16,
		left: u16,
	}

	const SIZE: usize = std::mem::size_of::<LinedefRaw>();

	if (bytes.len() % SIZE) != 0 {
		return Err(Error::MalformedFile("LINEDEFS"));
	}

	let mut ret = Vec::with_capacity(bytes.len() / SIZE);
	let mut cursor = Cursor::new(bytes);

	for _ in 0..(bytes.len() / SIZE) {
		let raw = cursor.read_from_bytes::<LinedefRaw>();

		ret.push(LineDef {
			udmf_id: -1,
			vert_start: u16::from_le(raw.v_start) as usize,
			vert_end: u16::from_le(raw.v_end) as usize,
			flags: LineFlags::from_bits_truncate(raw.flags as u32),
			special: u16::from_le(raw.special) as i32,
			trigger: u16::from_le(raw.trigger),
			args: [0; 5],
			side_right: u16::from_le(raw.right) as usize,
			side_left: {
				let s = u16::from_le(raw.left);

				if s == 0xFFFF {
					None
				} else {
					Some(s as usize)
				}
			},
			udmf: HashMap::default(),
		});
	}

	Ok(ret)
}

/// Returns [`Error::MalformedFile`] if the length `bytes` is not divisible by 28.
pub fn nodes(bytes: &[u8]) -> Result<Vec<BspNode>, Error> {
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

	const SIZE: usize = std::mem::size_of::<NodeRaw>();

	if (bytes.len() % SIZE) != 0 {
		return Err(Error::MalformedFile("NODES"));
	}

	let mut ret = Vec::with_capacity(bytes.len() / SIZE);
	let mut cursor = Cursor::new(bytes);

	for _ in 0..(bytes.len() / SIZE) {
		let raw = cursor.read_from_bytes::<NodeRaw>();

		let child_r = i16::from_le(raw.child_r);
		let child_l = i16::from_le(raw.child_l);

		let start = glam::vec2(
			-((i16::from_le(raw.y) as f32) * VANILLA_SCALEDOWN),
			-((i16::from_le(raw.x) as f32) * VANILLA_SCALEDOWN),
		);

		ret.push(BspNode {
			seg_start: start,
			seg_end: start
				+ glam::vec2(
					-((i16::from_le(raw.delta_y) as f32) * VANILLA_SCALEDOWN),
					-((i16::from_le(raw.delta_x) as f32) * VANILLA_SCALEDOWN),
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
		});
	}

	Ok(ret)
}

/// Returns [`Error::MalformedFile`] if the length `bytes` is not divisible by 26.
pub fn sectors(bytes: &[u8]) -> Result<Vec<SectorDef>, Error> {
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

	const SIZE: usize = std::mem::size_of::<SectorRaw>();

	if (bytes.len() % SIZE) != 0 {
		return Err(Error::MalformedFile("SECTORS"));
	}

	let mut ret = Vec::with_capacity(bytes.len() / 26);
	let mut cursor = Cursor::new(bytes);

	for _ in 0..(bytes.len() / SIZE) {
		let raw = cursor.read_from_bytes::<SectorRaw>();

		ret.push(SectorDef {
			udmf_id: 0,
			height_floor: (i16::from_le(raw.height_floor) as f32) * VANILLA_SCALEDOWN,
			height_ceil: (i16::from_le(raw.height_ceil) as f32) * VANILLA_SCALEDOWN,
			tex_floor: read_id8(raw.tex_floor),
			tex_ceil: read_id8(raw.tex_ceil),
			light_level: u16::from_le(raw.light_level) as i32,
			special: raw.special as i32,
			trigger: u16::from_le(raw.trigger),
			udmf: HashMap::default(),
		});
	}

	Ok(ret)
}

/// Returns [`Error::MalformedFile`] if the length `bytes` is not divisible by 12.
pub fn segs(bytes: &[u8]) -> Result<Vec<Seg>, Error> {
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

	const SIZE: usize = std::mem::size_of::<SegRaw>();

	if (bytes.len() % SIZE) != 0 {
		return Err(Error::MalformedFile("SEGS"));
	}

	let mut ret = Vec::with_capacity(bytes.len() / 12);
	let mut cursor = Cursor::new(bytes);

	for _ in 0..(bytes.len() / SIZE) {
		let raw = cursor.read_from_bytes::<SegRaw>();

		ret.push(Seg {
			vert_start: u16::from_le(raw.v_start) as usize,
			vert_end: u16::from_le(raw.v_end) as usize,
			angle: i16::from_le(raw.angle),
			linedef: u16::from_le(raw.linedef) as usize,
			direction: if i16::from_le(raw.direction) == 0 {
				SegDirection::Front
			} else {
				SegDirection::Back
			},
			offset: i16::from_le(raw.offset),
		});
	}

	Ok(ret)
}

/// Returns [`Error::MalformedFile`] if the length `bytes` is not divisible by 30.
pub fn sidedefs(bytes: &[u8]) -> Result<Vec<SideDef>, Error> {
	#[repr(C)]
	#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
	struct SidedefRaw {
		offs_x: i16,
		offs_y: i16,
		tex_top: [u8; 8],
		tex_bottom: [u8; 8],
		tex_mid: [u8; 8],
		sector: u16,
	}

	const SIZE: usize = std::mem::size_of::<SidedefRaw>();

	if (bytes.len() % SIZE) != 0 {
		return Err(Error::MalformedFile("SIDEDEFS"));
	}

	let mut ret = Vec::with_capacity(bytes.len() / 30);
	let mut cursor = Cursor::new(bytes);

	for _ in 0..(bytes.len() / SIZE) {
		let raw = cursor.read_from_bytes::<SidedefRaw>();

		ret.push(SideDef {
			offset: glam::ivec2(
				i16::from_le(raw.offs_x) as i32,
				i16::from_le(raw.offs_y) as i32,
			),
			tex_top: read_id8(raw.tex_top).filter(|id8| id8 != "-"),
			tex_bottom: read_id8(raw.tex_bottom).filter(|id8| id8 != "-"),
			tex_mid: read_id8(raw.tex_mid).filter(|id8| id8 != "-"),
			sector: u16::from_le(raw.sector) as usize,
			udmf: HashMap::default(),
		});
	}

	Ok(ret)
}

/// Returns [`Error::MalformedFile`] if the length `bytes` is not divisible by 4.
pub fn ssectors(bytes: &[u8]) -> Result<Vec<SubSector>, Error> {
	#[repr(C)]
	#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
	struct SSectorRaw {
		seg_count: u16,
		seg: u16,
	}

	const SIZE: usize = std::mem::size_of::<SSectorRaw>();

	if (bytes.len() % SIZE) != 0 {
		return Err(Error::MalformedFile("SSECTORS"));
	}

	let mut ret = Vec::with_capacity(bytes.len() / 4);
	let mut cursor = Cursor::new(bytes);

	for _ in 0..(bytes.len() / SIZE) {
		let raw = cursor.read_from_bytes::<SSectorRaw>();

		ret.push(SubSector {
			seg_count: u16::from_le(raw.seg_count) as usize,
			seg0: u16::from_le(raw.seg) as usize,
		});
	}

	Ok(ret)
}

/// Returns [`Error::MalformedFile`] if the length `bytes` is not divisible by 10.
pub fn things_doom(bytes: &[u8]) -> Result<Vec<ThingDef>, Error> {
	#[repr(C)]
	#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
	struct ThingRaw {
		x: i16,
		y: i16,
		angle: u16,
		ednum: u16,
		flags: i16,
	}

	const SIZE: usize = std::mem::size_of::<ThingRaw>();

	if (bytes.len() % SIZE) != 0 {
		return Err(Error::MalformedFile("THINGS"));
	}

	let mut ret = Vec::with_capacity(bytes.len() / SIZE);
	let mut cursor = Cursor::new(bytes);

	for _ in 0..(bytes.len() / SIZE) {
		let raw = cursor.read_from_bytes::<ThingRaw>();

		ret.push(ThingDef {
			tid: 0,
			ed_num: u16::from_le(raw.ednum),
			pos: glam::vec3(
				(i16::from_le(raw.x) as f32) * VANILLA_SCALEDOWN,
				0.0,
				(i16::from_le(raw.y) as f32) * VANILLA_SCALEDOWN,
			),
			angle: u16::from_le(raw.angle) as u32,
			flags: {
				let f = i16::from_le(raw.flags);
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
		});
	}

	Ok(ret)
}

/// Returns [`Error::MalformedFile`] if the length `bytes` is not divisible by 20.
pub fn things_extended(bytes: &[u8]) -> Result<Vec<ThingDef>, Error> {
	#[repr(C)]
	#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
	struct ThingRaw {
		tid: i16,
		x: i16,
		y: i16,
		z: i16,
		angle: u16,
		ednum: u16,
		flags: i16,
		args: [u8; 5],
	}

	const SIZE: usize = std::mem::size_of::<ThingRaw>();

	if (bytes.len() % SIZE) != 0 {
		return Err(Error::MalformedFile("THINGS (extended)"));
	}

	let mut ret = Vec::with_capacity(bytes.len() / SIZE);
	let mut cursor = Cursor::new(bytes);

	for _ in 0..(bytes.len() / SIZE) {
		let raw = cursor.read_from_bytes::<ThingRaw>();

		ret.push(ThingDef {
			tid: i16::from_le(raw.tid) as i32,
			ed_num: u16::from_le(raw.ednum),
			pos: glam::vec3(
				(i16::from_le(raw.x) as f32) * VANILLA_SCALEDOWN,
				(i16::from_le(raw.z) as f32) * VANILLA_SCALEDOWN,
				(i16::from_le(raw.y) as f32) * VANILLA_SCALEDOWN,
			),
			angle: u16::from_le(raw.angle) as u32,
			flags: {
				let f = i16::from_le(raw.flags);
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
				raw.args[0] as i32,
				raw.args[1] as i32,
				raw.args[2] as i32,
				raw.args[3] as i32,
				raw.args[4] as i32,
			],
			udmf: HashMap::default(),
		});
	}

	Ok(ret)
}

/// Returns [`Error::MalformedFile`] if the length `bytes` is not divisible by 4.
pub fn vertexes(bytes: &[u8]) -> Result<Vec<Vertex>, Error> {
	#[repr(C)]
	#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
	struct VertexRaw {
		x: i16,
		y: i16,
	}

	const SIZE: usize = std::mem::size_of::<VertexRaw>();

	if (bytes.len() % SIZE) != 0 {
		return Err(Error::MalformedFile("VERTEXES"));
	}

	let mut ret = Vec::with_capacity(bytes.len() / 4);
	let mut cursor = Cursor::new(bytes);

	for _ in 0..(bytes.len() / SIZE) {
		let raw = cursor.read_from_bytes::<VertexRaw>();

		ret.push(Vertex(glam::vec4(
			(i16::from_le(raw.x) as f32) * VANILLA_SCALEDOWN,
			0.0,
			(i16::from_le(raw.y) as f32) * VANILLA_SCALEDOWN,
			0.0,
		)));
	}

	Ok(ret)
}
