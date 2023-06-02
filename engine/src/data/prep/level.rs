//! Functions for turning vanilla lumps into levels.

use std::io::Cursor;

use util::read_id8;

use crate::{
	data::{
		dobj::{
			BspNode, BspNodeChild, Image, Level, LevelFormat, LineDef, Sector, Seg, SegDirection,
			SideDef, SubSector, Thing, ThingFlags,
		},
		prep::*,
		Catalog, FileRef, LevelError, PrepError, PrepErrorKind, SideTexture,
	},
	sim::{level::Vertex, line::Flags},
	util::io::CursorExt,
};

use super::SubContext;

/// All 16-bit integer position values get reduced by this
/// to fit VileTech's floating-point space.
const VANILLA_SCALEDOWN: f32 = 0.01;

impl Catalog {
	/// Covers both Doom- and Hexen-format levels.
	/// Returns `None` if `dir` is unlikely to represent a vanilla level definition.
	pub(super) fn try_prep_level_vanilla(
		&self,
		ctx: &SubContext,
		dir: FileRef,
	) -> Outcome<Level, ()> {
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

		for child in dir.children().unwrap() {
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
				_ => {}
			}
		}

		for lump in &[
			nodes, linedefs, sectors, segs, sidedefs, ssectors, things, vertexes,
		] {
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
			// TODO: After integrating a node builder, SEGS, SSECTORS, and NODES
			// will no longer be mandatory.
			if !lump.is_readable() {
				ctx.raise_error(PrepError {
					path: dir.path().to_path_buf(),
					kind: PrepErrorKind::Level(LevelError::UnreadableFile(
						lump.path().to_path_buf(),
					)),
				});

				return Outcome::Err(());
			}
		}

		// Sanity checks.

		let mut malformed = false;

		if (linedefs.byte_len() % 14) != 0 {
			ctx.raise_error(PrepError {
				path: dir.path().to_path_buf(),
				kind: PrepErrorKind::Level(LevelError::MalformedFile(
					linedefs.path().to_path_buf(),
				)),
			});

			malformed = true;
		}

		if (sectors.byte_len() % 26) != 0 {
			ctx.raise_error(PrepError {
				path: dir.path().to_path_buf(),
				kind: PrepErrorKind::Level(LevelError::MalformedFile(sectors.path().to_path_buf())),
			});

			malformed = true;
		}

		if (segs.byte_len() % 12) != 0 {
			ctx.raise_error(PrepError {
				path: dir.path().to_path_buf(),
				kind: PrepErrorKind::Level(LevelError::MalformedFile(segs.path().to_path_buf())),
			});

			malformed = true;
		}

		if (sidedefs.byte_len() % 30) != 0 {
			ctx.raise_error(PrepError {
				path: dir.path().to_path_buf(),
				kind: PrepErrorKind::Level(LevelError::MalformedFile(
					sidedefs.path().to_path_buf(),
				)),
			});

			malformed = true;
		}

		if (ssectors.byte_len() % 4) != 0 {
			ctx.raise_error(PrepError {
				path: dir.path().to_path_buf(),
				kind: PrepErrorKind::Level(LevelError::MalformedFile(
					ssectors.path().to_path_buf(),
				)),
			});

			malformed = true;
		}

		if (things.byte_len() % 10) != 0 {
			ctx.raise_error(PrepError {
				path: dir.path().to_path_buf(),
				kind: PrepErrorKind::Level(LevelError::MalformedFile(things.path().to_path_buf())),
			});

			malformed = true;
		}

		if (vertexes.byte_len() % 4) != 0 {
			ctx.raise_error(PrepError {
				path: dir.path().to_path_buf(),
				kind: PrepErrorKind::Level(LevelError::MalformedFile(
					vertexes.path().to_path_buf(),
				)),
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
			Self::prep_things_extended(things.read_bytes())
		};

		let mut level = Level::new(if behavior.is_some() {
			LevelFormat::Extended
		} else {
			LevelFormat::Doom
		});

		// As a placeholder in case map-info provides nothing.
		level.meta.name = dir.file_prefix().to_string().into();

		level.linedefs = linedefs;
		level.nodes = nodes;
		level.sectors = sectors;
		level.segs = segs;
		level.sidedefs = sidedefs;
		level.subsectors = subsectors;
		level.things = things;
		level.vertices = vertices;
		level.bounds = Level::bounds(&level.vertices);

		self.level_prep_sanity_checks(ctx, &level);

		Outcome::Ok(level)
	}

	pub(super) fn level_prep_sanity_checks(&self, ctx: &SubContext, level: &Level) {
		for (i, linedef) in level.linedefs.iter().enumerate() {
			if linedef.side_right >= level.sidedefs.len() {
				ctx.raise_error(PrepError {
					path: ctx.mntinfo.mount_point().join(level.meta.name.as_ref()),
					kind: PrepErrorKind::Level(LevelError::InvalidLinedefSide {
						linedef: i,
						left: false,
						sidedef: linedef.side_right,
						sides_len: level.sidedefs.len(),
					}),
				});
			}

			let Some(side_left) = linedef.side_left else { continue; };

			if side_left >= level.sidedefs.len() {
				ctx.raise_error(PrepError {
					path: ctx.mntinfo.mount_point().join(level.meta.name.as_ref()),
					kind: PrepErrorKind::Level(LevelError::InvalidLinedefSide {
						linedef: i,
						left: true,
						sidedef: side_left,
						sides_len: level.sidedefs.len(),
					}),
				});
			}
		}

		for (i, node) in level.nodes.iter().enumerate() {
			match node.child_l {
				BspNodeChild::SubSector(ssector) => {
					ctx.raise_error(PrepError {
						path: ctx.mntinfo.mount_point().join(level.meta.name.as_ref()),
						kind: PrepErrorKind::Level(LevelError::InvalidNodeSubsector {
							node: i,
							left: true,
							ssector,
							ssectors_len: level.subsectors.len(),
						}),
					});
				}
				BspNodeChild::SubNode(subnode) => {
					ctx.raise_error(PrepError {
						path: ctx.mntinfo.mount_point().join(level.meta.name.as_ref()),
						kind: PrepErrorKind::Level(LevelError::InvalidSubnode {
							node: i,
							left: true,
							subnode,
							nodes_len: level.nodes.len(),
						}),
					});
				}
			}

			match node.child_r {
				BspNodeChild::SubSector(ssector) => {
					ctx.raise_error(PrepError {
						path: ctx.mntinfo.mount_point().join(level.meta.name.as_ref()),
						kind: PrepErrorKind::Level(LevelError::InvalidNodeSubsector {
							node: i,
							left: false,
							ssector,
							ssectors_len: level.subsectors.len(),
						}),
					});
				}
				BspNodeChild::SubNode(subnode) => {
					ctx.raise_error(PrepError {
						path: ctx.mntinfo.mount_point().join(level.meta.name.as_ref()),
						kind: PrepErrorKind::Level(LevelError::InvalidSubnode {
							node: i,
							left: false,
							subnode,
							nodes_len: level.nodes.len(),
						}),
					});
				}
			}
		}

		for (i, sector) in level.sectors.iter().enumerate() {
			if let Some(tex_floor) = &sector.tex_floor {
				if self.last_by_nick::<Image>(tex_floor.as_str()).is_none() {
					ctx.raise_error(PrepError {
						path: ctx.mntinfo.mount_point().join(level.meta.name.as_ref()),
						kind: PrepErrorKind::Level(LevelError::UnknownFlat {
							sector: i,
							ceiling: false,
							name: *tex_floor,
						}),
					});
				}
			}

			if let Some(tex_ceil) = &sector.tex_ceil {
				if self.last_by_nick::<Image>(tex_ceil.as_str()).is_none() {
					ctx.raise_error(PrepError {
						path: ctx.mntinfo.mount_point().join(level.meta.name.as_ref()),
						kind: PrepErrorKind::Level(LevelError::UnknownFlat {
							sector: i,
							ceiling: true,
							name: *tex_ceil,
						}),
					});
				}
			}
		}

		for (i, seg) in level.segs.iter().enumerate() {
			if seg.linedef >= level.linedefs.len() {
				ctx.raise_error(PrepError {
					path: ctx.mntinfo.mount_point().join(level.meta.name.as_ref()),
					kind: PrepErrorKind::Level(LevelError::InvalidSegLinedef {
						seg: i,
						linedef: seg.linedef,
						lines_len: level.linedefs.len(),
					}),
				});
			}
		}

		for (i, sidedef) in level.sidedefs.iter().enumerate() {
			if let Some(tex_bottom) = &sidedef.tex_bottom {
				if self.last_by_nick::<Image>(tex_bottom.as_str()).is_none() {
					ctx.raise_error(PrepError {
						path: ctx.mntinfo.mount_point().join(level.meta.name.as_ref()),
						kind: PrepErrorKind::Level(LevelError::UnknownSideTex {
							sidedef: i,
							which: SideTexture::Bottom,
							name: *tex_bottom,
						}),
					});
				}
			}

			if let Some(tex_mid) = &sidedef.tex_mid {
				if self.last_by_nick::<Image>(tex_mid.as_str()).is_none() {
					ctx.raise_error(PrepError {
						path: ctx.mntinfo.mount_point().join(level.meta.name.as_ref()),
						kind: PrepErrorKind::Level(LevelError::UnknownSideTex {
							sidedef: i,
							which: SideTexture::Middle,
							name: *tex_mid,
						}),
					});
				}
			}

			if let Some(tex_top) = &sidedef.tex_top {
				if self.last_by_nick::<Image>(tex_top.as_str()).is_none() {
					ctx.raise_error(PrepError {
						path: ctx.mntinfo.mount_point().join(level.meta.name.as_ref()),
						kind: PrepErrorKind::Level(LevelError::UnknownSideTex {
							sidedef: i,
							which: SideTexture::Top,
							name: *tex_top,
						}),
					});
				}
			}

			if sidedef.sector >= level.sectors.len() {
				ctx.raise_error(PrepError {
					path: ctx.mntinfo.mount_point().join(level.meta.name.as_ref()),
					kind: PrepErrorKind::Level(LevelError::InvalidSidedefSector {
						sidedef: i,
						sector: sidedef.sector,
						sectors_len: level.sectors.len(),
					}),
				});
			}
		}

		for (i, subsector) in level.subsectors.iter().enumerate() {
			if subsector.seg0 >= level.segs.len() {
				ctx.raise_error(PrepError {
					path: ctx.mntinfo.mount_point().join(level.meta.name.as_ref()),
					kind: PrepErrorKind::Level(LevelError::InvalidSubsectorSeg {
						subsector: i,
						seg: subsector.seg0,
						segs_len: level.segs.len(),
					}),
				});
			}
		}

		let mut player1start = false;

		for (_, thingdef) in level.things.iter().enumerate() {
			if thingdef.ed_num == 1 {
				player1start = true;
			}

			#[cfg(any())] // TODO: Re-enable when VZScript load scripts are in.
			if self.bp_by_ednum(thingdef.ed_num).is_none() {
				ctx.raise_error(PrepError {
					path: ctx.mntinfo.mount_point().join(level.meta.name.as_ref()),
					kind: PrepErrorKind::Level(LevelError::UnknownEdNum {
						thingdef: i,
						ed_num: thingdef.ed_num,
					}),
				});
			}
		}

		if !player1start {
			ctx.raise_error(PrepError {
				path: ctx.mntinfo.mount_point().join(level.meta.name.as_ref()),
				kind: PrepErrorKind::Level(LevelError::NoPlayer1Start),
			});
		}
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
				args: [0, 0, 0, 0, 0],
			});
		}

		ret
	}

	#[must_use]
	fn prep_things_extended(bytes: &[u8]) -> Vec<Thing> {
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
