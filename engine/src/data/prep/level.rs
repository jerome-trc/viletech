//! Functions for turning vanilla lumps into levels.

use std::io::Cursor;

use crate::{
	data::{
		detail::Outcome,
		dobj::{
			BspNode, BspNodeChild, DatumHeader, Level, LevelFlags, LevelFormat, LevelMeta, LineDef,
			Sector, Seg, SegDirection, SideDef, SubSector, Thing, ThingFlags,
		},
		prep::*,
		Catalog, FileRef, LevelError, PrepError, PrepErrorKind,
	},
	sim::{level::Vertex, line::Flags},
	utils::io::CursorExt,
};

use super::SubContext;

/// All 16-bit integer position values get reduced by this
/// to fit VileTech's floating-point space.
const VANILLA_SCALEDOWN: f32 = 0.01;

impl Catalog {
	/// Covers both Doom- and Hexen-format levels.
	/// Returns `None` if `dir` is unlikely to represent a vanilla level definition.
	pub(super) fn try_prep_level_vanilla(&self, ctx: &SubContext, dir: FileRef) -> Outcome<(), ()> {
		let mut _blockmap = None;
		let mut linedefs = None;
		let mut nodes = None;
		let mut _reject = None;
		let mut sectors = None;
		let mut segs = None;
		let mut sidedefs = None;
		let mut ssectors = None;
		let mut things = None;
		let mut vertexes = None;
		let mut behavior = None;

		for child in dir.child_refs().unwrap() {
			match child.file_prefix() {
				"BLOCKMAP" => _blockmap = Some(child),
				"LINEDEFS" => linedefs = Some(child),
				"NODES" => nodes = Some(child),
				"REJECT" => _reject = Some(child),
				"SECTORS" => sectors = Some(child),
				"SEGS" => segs = Some(child),
				"SIDEDEFS" => sidedefs = Some(child),
				"SSECTORS" => ssectors = Some(child),
				"THINGS" => things = Some(child),
				"VERTEXES" => vertexes = Some(child),
				"BEHAVIOR" => behavior = Some(child),
				_ => {
					// Q: This is probably not a vanilla level, but could it be?
					// Might be a WAD out there with extra data in a level folder.
					return Outcome::None;
				}
			}
		}

		for lump in &[
			nodes, linedefs, sectors, segs, sidedefs, ssectors, things, vertexes,
		] {
			// TODO: What is mandatory might change if a nodebuilder gets integrated.

			if lump.is_none() {
				return Outcome::None;
			}
		}

		let linedefs = linedefs.unwrap();
		let nodes = nodes.unwrap();
		let segs = segs.unwrap();
		let sectors = sectors.unwrap();
		let sidedefs = sidedefs.unwrap();
		let ssectors = ssectors.unwrap();
		let things = things.unwrap();
		let vertexes = vertexes.unwrap();

		for lump in &[
			linedefs, nodes, sectors, segs, sidedefs, ssectors, things, vertexes,
		] {
			if !lump.is_readable() {
				ctx.errors.lock().push(PrepError {
					path: dir.path.to_path_buf(),
					kind: PrepErrorKind::Level(LevelError::UnreadableFile(lump.path.to_path_buf())),
				});

				return Outcome::Err(());
			}
		}

		// Sanity checks.

		let mut malformed = false;

		if (linedefs.byte_len() % 14) != 0 {
			ctx.errors.lock().push(PrepError {
				path: dir.path.to_path_buf(),
				kind: PrepErrorKind::Level(LevelError::MalformedFile(linedefs.path.to_path_buf())),
			});

			malformed = true;
		}

		if (sectors.byte_len() % 26) != 0 {
			ctx.errors.lock().push(PrepError {
				path: dir.path.to_path_buf(),
				kind: PrepErrorKind::Level(LevelError::MalformedFile(sectors.path.to_path_buf())),
			});

			malformed = true;
		}

		if (segs.byte_len() % 12) != 0 {
			ctx.errors.lock().push(PrepError {
				path: dir.path.to_path_buf(),
				kind: PrepErrorKind::Level(LevelError::MalformedFile(segs.path.to_path_buf())),
			});

			malformed = true;
		}

		if (sidedefs.byte_len() % 30) != 0 {
			ctx.errors.lock().push(PrepError {
				path: dir.path.to_path_buf(),
				kind: PrepErrorKind::Level(LevelError::MalformedFile(sidedefs.path.to_path_buf())),
			});

			malformed = true;
		}

		if (ssectors.byte_len() % 4) != 0 {
			ctx.errors.lock().push(PrepError {
				path: dir.path.to_path_buf(),
				kind: PrepErrorKind::Level(LevelError::MalformedFile(ssectors.path.to_path_buf())),
			});

			malformed = true;
		}

		if (things.byte_len() % 10) != 0 {
			ctx.errors.lock().push(PrepError {
				path: dir.path.to_path_buf(),
				kind: PrepErrorKind::Level(LevelError::MalformedFile(things.path.to_path_buf())),
			});

			malformed = true;
		}

		if (vertexes.byte_len() % 4) != 0 {
			ctx.errors.lock().push(PrepError {
				path: dir.path.to_path_buf(),
				kind: PrepErrorKind::Level(LevelError::MalformedFile(vertexes.path.to_path_buf())),
			});

			malformed = true;
		}

		if malformed {
			return Outcome::Err(());
		}

		let linedefs = Self::prep_linedefs(linedefs.read_bytes());
		let nodes = Self::prep_nodes(nodes.read_bytes());
		let sectors = Self::prep_sectors(sectors.read_bytes());
		let segs = Self::prep_segs(segs.read_bytes());
		let sidedefs = Self::prep_sidedefs(sidedefs.read_bytes());
		let subsectors = Self::prep_ssectors(ssectors.read_bytes());
		let vertices = Self::prep_vertexes(vertexes.read_bytes());

		let things = if behavior.is_none() {
			Self::prep_things_doom(things.read_bytes())
		} else {
			Self::prep_things_hexen(things.read_bytes())
		};

		let level = Level {
			header: DatumHeader {
				id: format!(
					"{mount_id}/{id}",
					mount_id = ctx.mntinfo.id(),
					id = dir.file_prefix()
				),
			},
			meta: LevelMeta {
				name: String::default().into(),
				author_name: String::default().into(),
				music: None,
				next: None,
				next_secret: None,
				par_time: 0,
				special_num: 0,
				flags: LevelFlags::empty(),
			},
			format: if behavior.is_some() {
				LevelFormat::Hexen
			} else {
				LevelFormat::Doom
			},
			linedefs,
			nodes,
			sectors,
			segs,
			sidedefs,
			subsectors,
			things,
			vertices,
		};

		ctx.add_datum(level);

		Outcome::Ok(())
	}

	#[must_use]
	fn prep_linedefs(bytes: &[u8]) -> Vec<LineDef> {
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

		debug_assert_eq!(bytes.len() % SIZE, 0);

		let mut ret = Vec::with_capacity(bytes.len() / SIZE);
		let mut cursor = Cursor::new(bytes);

		for _ in 0..(bytes.len() / SIZE) {
			let raw = cursor.read_from_bytes::<LinedefRaw>();

			ret.push(LineDef {
				udmf_id: -1,
				vert_start: u16::from_le(raw.v_start) as usize,
				vert_end: u16::from_le(raw.v_end) as usize,
				flags: Flags::from_bits_truncate(raw.flags as u32),
				special: u16::from_le(raw.special),
				trigger: u16::from_le(raw.trigger),
				args: None,
				side_right: u16::from_le(raw.right) as usize,
				side_left: {
					let s = u16::from_le(raw.left);

					if s == 0xFFFF {
						None
					} else {
						Some(s as usize)
					}
				},
			});
		}

		ret
	}

	#[must_use]
	fn prep_nodes(bytes: &[u8]) -> Vec<BspNode> {
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

		debug_assert_eq!(bytes.len() % SIZE, 0);

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

		ret
	}

	#[must_use]
	fn prep_sectors(bytes: &[u8]) -> Vec<Sector> {
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

		debug_assert_eq!(bytes.len() % SIZE, 0);

		let mut ret = Vec::with_capacity(bytes.len() / 26);
		let mut cursor = Cursor::new(bytes);

		for _ in 0..(bytes.len() / SIZE) {
			let raw = cursor.read_from_bytes::<SectorRaw>();

			ret.push(Sector {
				udmf_id: 0,
				height_floor: (i16::from_le(raw.height_floor) as f32) * VANILLA_SCALEDOWN,
				height_ceil: (i16::from_le(raw.height_ceil) as f32) * VANILLA_SCALEDOWN,
				tex_floor: read_id8(raw.tex_floor),
				tex_ceil: read_id8(raw.tex_ceil),
				light_level: u16::from_le(raw.light_level) as i32,
				special: raw.special,
				trigger: u16::from_le(raw.trigger),
			});
		}

		ret
	}

	#[must_use]
	fn prep_segs(bytes: &[u8]) -> Vec<Seg> {
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

		debug_assert_eq!(bytes.len() % SIZE, 0);

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

		ret
	}

	#[must_use]
	fn prep_sidedefs(bytes: &[u8]) -> Vec<SideDef> {
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

		debug_assert_eq!(bytes.len() % SIZE, 0);

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
			});
		}

		ret
	}

	#[must_use]
	fn prep_ssectors(bytes: &[u8]) -> Vec<SubSector> {
		#[repr(C)]
		#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
		struct SSectorRaw {
			seg_count: u16,
			seg: u16,
		}

		const SIZE: usize = std::mem::size_of::<SSectorRaw>();

		debug_assert_eq!(bytes.len() % SIZE, 0);

		let mut ret = Vec::with_capacity(bytes.len() / 4);
		let mut cursor = Cursor::new(bytes);

		for _ in 0..(bytes.len() / SIZE) {
			let raw = cursor.read_from_bytes::<SSectorRaw>();

			ret.push(SubSector {
				seg_count: u16::from_le(raw.seg_count) as usize,
				seg0: u16::from_le(raw.seg) as usize,
			});
		}

		ret
	}

	#[must_use]
	fn prep_things_doom(bytes: &[u8]) -> Vec<Thing> {
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

		debug_assert_eq!(bytes.len() % SIZE, 0);

		let mut ret = Vec::with_capacity(bytes.len() / SIZE);
		let mut cursor = Cursor::new(bytes);

		for _ in 0..(bytes.len() / SIZE) {
			let raw = cursor.read_from_bytes::<ThingRaw>();

			ret.push(Thing {
				tid: 0,
				num: u16::from_le(raw.ednum),
				pos: glam::vec3(
					(i16::from_le(raw.x) as f32) * VANILLA_SCALEDOWN,
					0.0,
					(i16::from_le(raw.y) as f32) * VANILLA_SCALEDOWN,
				),
				angle: u16::from_le(raw.angle),
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
				args: [0, 0, 0, 0, 0],
			});
		}

		ret
	}

	#[must_use]
	fn prep_things_hexen(bytes: &[u8]) -> Vec<Thing> {
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

		debug_assert_eq!(bytes.len() % SIZE, 0);

		let mut ret = Vec::with_capacity(bytes.len() / SIZE);
		let mut cursor = Cursor::new(bytes);

		for _ in 0..(bytes.len() / SIZE) {
			let raw = cursor.read_from_bytes::<ThingRaw>();

			ret.push(Thing {
				tid: i16::from_le(raw.tid) as i32,
				num: u16::from_le(raw.ednum),
				pos: glam::vec3(
					(i16::from_le(raw.x) as f32) * VANILLA_SCALEDOWN,
					(i16::from_le(raw.z) as f32) * VANILLA_SCALEDOWN,
					(i16::from_le(raw.y) as f32) * VANILLA_SCALEDOWN,
				),
				angle: u16::from_le(raw.angle),
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
				args: [
					raw.args[0] as i32,
					raw.args[1] as i32,
					raw.args[2] as i32,
					raw.args[3] as i32,
					raw.args[4] as i32,
				],
			});
		}

		ret
	}

	#[must_use]
	fn prep_vertexes(bytes: &[u8]) -> Vec<Vertex> {
		#[repr(C)]
		#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
		struct VertexRaw {
			x: i16,
			y: i16,
		}

		const SIZE: usize = std::mem::size_of::<VertexRaw>();

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

		ret
	}
}
