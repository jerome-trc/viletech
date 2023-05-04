//! Functions for turning vanilla lumps and UDMF TEXTMAP into levels.

use crate::{
	data::{
		detail::Outcome, AssetHeader, Catalog, FileRef, Level, LevelError, LevelFlags, LevelMeta,
		LineDef, LineSpecial, PrepError, PrepErrorKind, Sector, SectorSpecial, Seg, SideDef,
		SubSector, Thing, ThingFlags,
	},
	sim::{level::Vertex, line::LineFlags},
	EditorNum, ShortId,
};

use super::SubContext;

#[derive(Debug)]
struct LevelContext<'ctx> {
	_sub: &'ctx SubContext<'ctx>,
	_dir: FileRef<'ctx>,
} // (RAT) Context with context with context...

impl Catalog {
	/// Returns `None` if `dir` is unlikely to represent a vanilla level definition.
	pub(super) fn try_prep_level_vanilla(&self, ctx: &SubContext, dir: FileRef) -> Outcome<(), ()> {
		let mut _blockmap = None;
		let mut linedefs = None;
		let mut _nodes = None;
		let mut _reject = None;
		let mut sectors = None;
		let mut segs = None;
		let mut sidedefs = None;
		let mut ssectors = None;
		let mut things = None;
		let mut vertexes = None;

		for child in dir.child_refs().unwrap() {
			match child.file_prefix() {
				"BLOCKMAP" => _blockmap = Some(child),
				"LINEDEFS" => linedefs = Some(child),
				"NODES" => _nodes = Some(child),
				"REJECT" => _reject = Some(child),
				"SECTORS" => sectors = Some(child),
				"SEGS" => segs = Some(child),
				"SIDEDEFS" => sidedefs = Some(child),
				"SSECTORS" => ssectors = Some(child),
				"THINGS" => things = Some(child),
				"VERTEXES" => vertexes = Some(child),
				_ => {
					// Q: This is probably not a vanilla level, but could it be?
					// Might be a WAD out there with extra data in a level folder.
					return Outcome::None;
				}
			}
		}

		for lump in &[
			linedefs, sectors, segs, sidedefs, ssectors, things, vertexes,
		] {
			if lump.is_none() {
				return Outcome::None;
			}
		}

		let linedefs = linedefs.unwrap();
		let segs = segs.unwrap();
		let sectors = sectors.unwrap();
		let sidedefs = sidedefs.unwrap();
		let ssectors = ssectors.unwrap();
		let things = things.unwrap();
		let vertexes = vertexes.unwrap();

		for lump in &[
			linedefs, sectors, segs, sidedefs, ssectors, things, vertexes,
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

		let lctx = LevelContext {
			_sub: ctx,
			_dir: dir,
		};

		let vertices = Self::prep_vertexes(vertexes.read_bytes());
		let linedefs = Self::prep_linedefs(&lctx, linedefs.read_bytes());
		let sectors = Self::prep_sectors(&lctx, sectors.read_bytes());
		let segs = Self::prep_segs(segs.read_bytes());
		let sidedefs = Self::prep_sidedefs(sidedefs.read_bytes());
		let subsectors = Self::prep_ssectors(ssectors.read_bytes());
		let things = Self::prep_things(things.read_bytes());

		let level = Level {
			header: AssetHeader {
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
			udmf_namespace: None,
			vertices,
			linedefs,
			sectors,
			segs,
			sidedefs,
			subsectors,
			things,
		};

		ctx.add_asset(level);

		Outcome::Ok(())
	}

	#[must_use]
	fn prep_linedefs(ctx: &LevelContext, bytes: &[u8]) -> Vec<LineDef> {
		#[repr(C)]
		#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
		struct LinedefRaw {
			v_from: i16,
			v_to: i16,
			flags: i16,
			special: i16,
			trigger: i16,
			right: i16,
			left: i16,
		}

		const SIZE: usize = std::mem::size_of::<LinedefRaw>();

		let mut ret = Vec::with_capacity(bytes.len() / SIZE);
		let mut pos = 0;

		while pos < bytes.len() {
			let raw = bytemuck::from_bytes::<LinedefRaw>(&bytes[pos..(pos + SIZE)]);

			ret.push(LineDef {
				id: -1,
				vert_from: i16::from_le(raw.v_from) as i32,
				vert_to: i16::from_le(raw.v_to) as i32,
				flags: LineFlags::from_bits_truncate(raw.flags as u32),
				special: Self::linedef_special_from_vanilla(ctx, raw.special),
				args: [0; 5],
				side_right: i16::from_le(raw.right) as i32,
				side_left: i16::from_le(raw.left) as i32,
			});

			pos += SIZE;
		}

		ret
	}

	#[must_use]
	fn linedef_special_from_vanilla(_ctx: &LevelContext, _short: i16) -> LineSpecial {
		// TODO: Not going to write all conversions until the internal
		// representation is finalized.

		/*

		ctx.sub.errors.lock().push(PrepError {
			path: ctx.dir.path.to_path_buf(),
			kind: PrepErrorKind::Level(LevelError::UnknownLineSpecial(short)),
		});

		*/

		LineSpecial::Unknown
	}

	#[must_use]
	fn prep_sectors(ctx: &LevelContext, bytes: &[u8]) -> Vec<Sector> {
		#[repr(C)]
		#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
		struct SectorRaw {
			height_floor: i16,
			height_ceil: i16,
			tex_floor: [u8; 8],
			tex_ceil: [u8; 8],
			light_level: i16,
			special: i16,
			tag: i16,
		}

		const SIZE: usize = std::mem::size_of::<SectorRaw>();

		let mut ret = Vec::with_capacity(bytes.len() / 26);
		let mut pos = 0;

		while pos < bytes.len() {
			let raw = bytemuck::from_bytes::<SectorRaw>(&bytes[pos..(pos + SIZE)]);

			ret.push(Sector {
				id: 0,
				height_floor: i16::from_le(raw.height_floor) as i32,
				height_ceil: i16::from_le(raw.height_ceil) as i32,
				tex_floor: read_shortid(raw.tex_floor),
				tex_ceil: read_shortid(raw.tex_ceil),
				light_level: i16::from_le(raw.light_level) as i32,
				special: Self::sector_special_from_vanilla(ctx, i16::from_le(raw.special)),
				tag: i16::from_le(raw.tag),
			});

			pos += SIZE;
		}

		ret
	}

	#[must_use]
	fn prep_segs(bytes: &[u8]) -> Vec<Seg> {
		#[repr(C)]
		#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
		struct SegRaw {
			vert_start: i16,
			vert_end: i16,
			angle: i16,
			linedef: i16,
			direction: i16,
			offset: i16,
		}

		const SIZE: usize = std::mem::size_of::<SegRaw>();

		let mut ret = Vec::with_capacity(bytes.len() / 12);
		let mut pos = 0;

		while pos < bytes.len() {
			let raw = bytemuck::from_bytes::<SegRaw>(&bytes[pos..(pos + SIZE)]);

			ret.push(Seg {
				vert_start: i16::from_le(raw.vert_start) as i32,
				vert_end: i16::from_le(raw.vert_end) as i32,
				angle: i16::from_le(raw.angle),
				linedef: i16::from_le(raw.linedef) as i32,
				direction: i16::from_le(raw.direction),
				offset: i16::from_le(raw.offset),
			});

			pos += SIZE;
		}

		ret
	}

	#[must_use]
	fn sector_special_from_vanilla(_ctx: &LevelContext, _short: i16) -> SectorSpecial {
		// TODO: Not going to write all conversions until the internal
		// representation is finalized.

		/*

		ctx.sub.errors.lock().push(PrepError {
			path: ctx.dir.path.to_path_buf(),
			kind: PrepErrorKind::Level(LevelError::UnknownSectorSpecial(short)),
		});

		*/

		SectorSpecial::Unknown
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
			sector: i16,
		}

		const SIZE: usize = std::mem::size_of::<SidedefRaw>();

		let mut ret = Vec::with_capacity(bytes.len() / 30);
		let mut pos = 0;

		while pos < bytes.len() {
			let raw = bytemuck::from_bytes::<SidedefRaw>(&bytes[pos..(pos + SIZE)]);

			ret.push(SideDef {
				offset: glam::ivec2(
					i16::from_le(raw.offs_x) as i32,
					i16::from_le(raw.offs_y) as i32,
				),
				tex_top: read_shortid(raw.tex_top),
				tex_bottom: read_shortid(raw.tex_bottom),
				tex_mid: read_shortid(raw.tex_mid),
				sector: i16::from_le(raw.sector) as i32,
			});

			pos += SIZE;
		}

		ret
	}

	#[must_use]
	fn prep_ssectors(bytes: &[u8]) -> Vec<SubSector> {
		#[repr(C)]
		#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
		struct SSectorRaw {
			seg_count: i16,
			seg: i16,
		}

		const SIZE: usize = std::mem::size_of::<SSectorRaw>();

		let mut ret = Vec::with_capacity(bytes.len() / 4);
		let mut pos = 0;

		while pos < bytes.len() {
			let raw = bytemuck::from_bytes::<SSectorRaw>(&bytes[pos..(pos + SIZE)]);

			ret.push(SubSector {
				seg_count: i16::from_le(raw.seg_count) as i32,
				seg: i16::from_le(raw.seg) as i32,
			});

			pos += SIZE;
		}

		ret
	}

	#[must_use]
	fn prep_things(bytes: &[u8]) -> Vec<Thing> {
		#[repr(C)]
		#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
		struct ThingRaw {
			x: i16,
			y: i16,
			angle: i16,
			num: i16,
			flags: i16,
		}

		const SIZE: usize = std::mem::size_of::<ThingRaw>();

		let mut ret = Vec::with_capacity(bytes.len() / 10);
		let mut pos = 0;

		while pos < bytes.len() {
			let raw = bytemuck::from_bytes::<ThingRaw>(&bytes[pos..(pos + SIZE)]);

			ret.push(Thing {
				num: i16::from_le(raw.num).max(0) as EditorNum,
				pos: glam::vec3(i16::from_le(raw.x) as f32, i16::from_le(raw.y) as f32, 0.0),
				angle: i16::from_le(raw.angle) as f64,
				flags: {
					let flags = i16::from_le(raw.flags);
					let mut f = ThingFlags::empty();

					if (flags & (1 << 0)) != 0 {
						f.insert(ThingFlags::SKILL_1 | ThingFlags::SKILL_2);
					}

					if (flags & (1 << 1)) != 0 {
						f.insert(ThingFlags::SKILL_3);
					}

					if (flags & (1 << 2)) != 0 {
						f.insert(ThingFlags::SKILL_4 | ThingFlags::SKILL_5);
					}

					if (flags & (1 << 3)) != 0 {
						f.insert(ThingFlags::AMBUSH);
					}

					if (flags & (1 << 4)) != 0 {
						f.insert(ThingFlags::COOP);
					} else {
						f.insert(ThingFlags::SINGLEPLAY);
					}

					f
				},
			});

			pos += SIZE;
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
		let mut pos = 0;

		while pos < bytes.len() {
			let raw = bytemuck::from_bytes::<VertexRaw>(&bytes[pos..(pos + SIZE)]);

			ret.push(Vertex(glam::vec3(
				i16::from_le(raw.x) as f32,
				i16::from_le(raw.y) as f32,
				0.0,
			)));

			pos += SIZE;
		}

		ret
	}

	/// Returns `None` if `dir` is unlikely to represent a UDMF level definition.
	pub(super) fn try_prep_level_udmf(&self, _ctx: &SubContext, dir: FileRef) -> Outcome<(), ()> {
		let mut _behavior = None;
		let mut _dialogue = None;
		let mut _scripts = None;
		let mut _textmap = None;
		let mut _znodes = None;

		for child in dir.child_refs().unwrap() {
			match child.file_prefix() {
				"BEHAVIOR" => _behavior = Some(child),
				"DIALOGUE" => _dialogue = Some(child),
				"SCRIPTS" => _scripts = Some(child),
				"TEXTMAP" => _textmap = Some(child),
				"ZNODES" => _znodes = Some(child),
				_ => {
					// Q: This is probably not a UDMF level, but could it be?
					// Might be a WAD out there with extra data in a level folder.
					return Outcome::None;
				}
			}
		}

		Outcome::None // TODO
	}
}

/// Returns `None` if `shortid` starts with a NUL.
/// Return values have no trailing NUL bytes.
#[must_use]
fn read_shortid(shortid: [u8; 8]) -> Option<ShortId> {
	if shortid.starts_with(&[b'\0']) {
		return None;
	}

	let mut ret = ShortId::new();

	for byte in shortid {
		if byte == b'\0' {
			break;
		}

		ret.push(char::from(byte));
	}

	Some(ret)
}
