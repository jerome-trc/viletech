//! Functions for turning vanilla lumps and UDMF TEXTMAP into levels.

use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};
use glam::{DVec2, DVec3, IVec2};

use crate::{
	data::{
		detail::AssetKey, AssetHeader, Catalog, FileRef, Level, LevelError, LevelFlags, LevelMeta,
		LineDef, LineDefFlags, LineSpecial, PrepError, PrepErrorKind, Sector, SectorSpecial, Seg,
		SideDef, SubSector, Thing, ThingFlags, Vertex,
	},
	EditorNum, ShortId,
};

use super::SubContext;

#[derive(Debug)]
struct LevelContext<'ctx> {
	sub: &'ctx SubContext<'ctx>,
	dir: FileRef<'ctx>,
} // (RAT) Context with context with context...

impl Catalog {
	// Q: `bytemuck` for reading vanilla level data?
	// Only if little endianness can be forced.

	/// Returns `None` if `dir` is unlikely to represent a vanilla level definition.
	#[must_use]
	pub(super) fn try_prep_level_vanilla(
		&self,
		ctx: &SubContext,
		dir: FileRef,
	) -> Option<Result<(), ()>> {
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

		for child in dir.child_refs() {
			match child.file_stem() {
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
					return None;
				}
			}
		}

		for lump in &[
			linedefs, sectors, segs, sidedefs, ssectors, things, vertexes,
		] {
			if lump.is_none() {
				return None;
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

				return Some(Err(()));
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
			return Some(Err(()));
		}

		let lctx = LevelContext { sub: ctx, dir };

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
					id = dir.file_stem()
				),
			},
			meta: LevelMeta {
				name: String::default(),
				author_name: String::default(),
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

		self.register_asset(ctx, level);

		Some(Ok(()))
	}

	#[must_use]
	fn prep_linedefs(ctx: &LevelContext, bytes: &[u8]) -> Vec<LineDef> {
		let mut ret = Vec::with_capacity(bytes.len() / 14);
		let mut cursor = Cursor::new(bytes);

		while cursor.position() < (bytes.len() as u64) {
			let v_from = cursor.read_i16::<LittleEndian>().unwrap();
			let v_to = cursor.read_i16::<LittleEndian>().unwrap();
			let flags = cursor.read_i16::<LittleEndian>().unwrap();
			let special = cursor.read_i16::<LittleEndian>().unwrap();
			let right = cursor.read_i16::<LittleEndian>().unwrap();
			let left = cursor.read_i16::<LittleEndian>().unwrap();

			ret.push(LineDef {
				id: -1,
				vert_from: v_from as i32,
				vert_to: v_to as i32,
				flags: LineDefFlags::from_bits_truncate(flags as u32),
				special: Self::linedef_special_from_vanilla(ctx, special),
				args: [0; 5],
				side_right: right as i32,
				side_left: left as i32,
			});
		}

		ret
	}

	#[must_use]
	fn linedef_special_from_vanilla(ctx: &LevelContext, short: i16) -> LineSpecial {
		// TODO: Not going to write all conversions until the internal
		// representation is finalized.

		ctx.sub.errors.lock().push(PrepError {
			path: ctx.dir.path.to_path_buf(),
			kind: PrepErrorKind::Level(LevelError::UnknownLineSpecial(short)),
		});

		LineSpecial::Unknown
	}

	#[must_use]
	fn prep_sectors(ctx: &LevelContext, bytes: &[u8]) -> Vec<Sector> {
		let mut ret = Vec::with_capacity(bytes.len() / 26);
		let mut cursor = Cursor::new(bytes);

		while cursor.position() < (bytes.len() as u64) {
			let height_floor = cursor.read_i16::<LittleEndian>().unwrap();
			let height_ceil = cursor.read_i16::<LittleEndian>().unwrap();
			let tex_floor = read_shortid(&mut cursor);
			let tex_ceil = read_shortid(&mut cursor);
			let light_level = cursor.read_i16::<LittleEndian>().unwrap();
			let special = cursor.read_i16::<LittleEndian>().unwrap();
			let tag = cursor.read_i16::<LittleEndian>().unwrap();

			ret.push(Sector {
				id: 0,
				height_floor: height_floor as i32,
				height_ceil: height_ceil as i32,
				tex_floor: if tex_floor.find(|c| c != '\0').is_none() {
					None
				} else {
					Some(tex_floor)
				},
				tex_ceil: if tex_ceil.find(|c| c != '\0').is_none() {
					None
				} else {
					Some(tex_ceil)
				},
				light_level: light_level as i32,
				special: Self::sector_special_from_vanilla(ctx, special),
				tag,
			});
		}

		ret
	}

	#[must_use]
	fn prep_segs(bytes: &[u8]) -> Vec<Seg> {
		let mut ret = Vec::with_capacity(bytes.len() / 12);
		let mut cursor = Cursor::new(bytes);

		while cursor.position() < (bytes.len() as u64) {
			let vert_start = cursor.read_i16::<LittleEndian>().unwrap();
			let vert_end = cursor.read_i16::<LittleEndian>().unwrap();
			let angle = cursor.read_i16::<LittleEndian>().unwrap();
			let linedef = cursor.read_i16::<LittleEndian>().unwrap();
			let direction = cursor.read_i16::<LittleEndian>().unwrap();
			let offset = cursor.read_i16::<LittleEndian>().unwrap();

			ret.push(Seg {
				vert_start: vert_start as i32,
				vert_end: vert_end as i32,
				angle,
				linedef: linedef as i32,
				direction,
				offset,
			});
		}

		ret
	}

	#[must_use]
	fn sector_special_from_vanilla(ctx: &LevelContext, short: i16) -> SectorSpecial {
		// TODO: Not going to write all conversions until the internal
		// representation is finalized.

		ctx.sub.errors.lock().push(PrepError {
			path: ctx.dir.path.to_path_buf(),
			kind: PrepErrorKind::Level(LevelError::UnknownSectorSpecial(short)),
		});

		SectorSpecial::Unknown
	}

	#[must_use]
	fn prep_sidedefs(bytes: &[u8]) -> Vec<SideDef> {
		let mut ret = Vec::with_capacity(bytes.len() / 30);
		let mut cursor = Cursor::new(bytes);

		while cursor.position() < (bytes.len() as u64) {
			let offs_x = cursor.read_i16::<LittleEndian>().unwrap();
			let offs_y = cursor.read_i16::<LittleEndian>().unwrap();
			let tex_top = read_shortid(&mut cursor);
			let tex_bottom = read_shortid(&mut cursor);
			let tex_mid = read_shortid(&mut cursor);
			let sector = cursor.read_i16::<LittleEndian>().unwrap();

			ret.push(SideDef {
				offset: IVec2::new(offs_x as i32, offs_y as i32),
				tex_top: if tex_top.as_bytes() == b"-\0\0\0\0\0\0\0" {
					None
				} else {
					Some(tex_top)
				},
				tex_bottom: if tex_bottom.as_bytes() == b"-\0\0\0\0\0\0\0" {
					None
				} else {
					Some(tex_bottom)
				},
				tex_mid: if tex_mid.as_bytes() == b"-\0\0\0\0\0\0\0" {
					None
				} else {
					Some(tex_mid)
				},
				sector: sector as i32,
			});
		}

		ret
	}

	#[must_use]
	fn prep_ssectors(bytes: &[u8]) -> Vec<SubSector> {
		let mut ret = Vec::with_capacity(bytes.len() / 4);
		let mut cursor = Cursor::new(bytes);

		while cursor.position() < (bytes.len() as u64) {
			let seg_count = cursor.read_i16::<LittleEndian>().unwrap();
			let seg = cursor.read_i16::<LittleEndian>().unwrap();

			ret.push(SubSector {
				seg_count: seg_count as i32,
				seg: seg as i32,
			});
		}

		ret
	}

	#[must_use]
	fn prep_things(bytes: &[u8]) -> Vec<Thing> {
		let mut ret = Vec::with_capacity(bytes.len() / 10);
		let mut cursor = Cursor::new(bytes);

		while cursor.position() < (bytes.len() as u64) {
			let x = cursor.read_i16::<LittleEndian>().unwrap();
			let y = cursor.read_i16::<LittleEndian>().unwrap();
			let angle = cursor.read_i16::<LittleEndian>().unwrap();
			let num = cursor.read_i16::<LittleEndian>().unwrap();
			let flags = cursor.read_i16::<LittleEndian>().unwrap();

			ret.push(Thing {
				num: num.max(0) as EditorNum,
				pos: DVec3::new(x as f64, y as f64, 0.0),
				angle: angle as f64,
				flags: {
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
		}

		ret
	}

	#[must_use]
	fn prep_vertexes(bytes: &[u8]) -> Vec<Vertex> {
		let mut ret = Vec::with_capacity(bytes.len() / 4);
		let mut cursor = Cursor::new(bytes);

		while cursor.position() < (bytes.len() as u64) {
			let x = cursor.read_i16::<LittleEndian>().unwrap();
			let y = cursor.read_i16::<LittleEndian>().unwrap();

			ret.push(Vertex(DVec2::new(x as f64, y as f64)));
		}

		ret
	}

	/// Returns `None` if `dir` is unlikely to represent a UDMF level definition.
	#[must_use]
	pub(super) fn try_prep_level_udmf(
		&self,
		_ctx: &SubContext,
		dir: FileRef,
	) -> Option<Result<AssetKey, ()>> {
		let mut _behavior = None;
		let mut _dialogue = None;
		let mut _scripts = None;
		let mut _textmap = None;
		let mut _znodes = None;

		for child in dir.child_refs() {
			match child.file_stem() {
				"BEHAVIOR" => _behavior = Some(child),
				"DIALOGUE" => _dialogue = Some(child),
				"SCRIPTS" => _scripts = Some(child),
				"TEXTMAP" => _textmap = Some(child),
				"ZNODES" => _znodes = Some(child),
				_ => {
					// Q: This is probably not a UDMF level, but could it be?
					// Might be a WAD out there with extra data in a level folder.
					return None;
				}
			}
		}

		None // TODO
	}
}

/// Panics if the wrapped byte slice does not have 8 remaining bytes.
#[must_use]
fn read_shortid(cursor: &mut Cursor<&[u8]>) -> ShortId {
	let mut ret = ShortId::new();

	for _ in 0..8 {
		ret.push(char::from(cursor.read_u8().unwrap()));
	}

	ret
}
