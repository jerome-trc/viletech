//! Functions for turning vanilla and UDMF lumps into levels.

use crate::level::{
	self,
	repr::{LevelBsp, LevelFormat, LevelGeom},
	LevelDef,
};

use crate::catalog::{dobj::Image, prep::*, Catalog, FileRef, PrepError, PrepErrorKind};

use super::SubContext;

impl Catalog {
	/// Covers both Doom- and Hexen-format levels.
	/// Returns `None` if `dir` is unlikely to represent a vanilla level definition.
	pub(super) fn try_prep_level_vanilla(
		&self,
		ctx: &SubContext,
		dir: FileRef,
	) -> Outcome<LevelDef, ()> {
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
					kind: PrepErrorKind::Unreadable(lump.path().to_path_buf()),
				});

				return Outcome::Err(());
			}
		}

		let mut malformed = false;

		let linedefs = match level::read::linedefs(linedefs.read_bytes()) {
			Ok(ld) => ld,
			Err(err) => {
				ctx.raise_error(PrepError {
					path: dir.path().to_path_buf(),
					kind: PrepErrorKind::Level(err),
				});

				malformed = true;

				vec![]
			}
		};

		let nodes = match level::read::nodes(nodes.read_bytes()) {
			Ok(n) => n,
			Err(err) => {
				ctx.raise_error(PrepError {
					path: dir.path().to_path_buf(),
					kind: PrepErrorKind::Level(err),
				});

				malformed = true;

				vec![]
			}
		};

		let sectors = match level::read::sectors(sectors.read_bytes()) {
			Ok(s) => s,
			Err(err) => {
				ctx.raise_error(PrepError {
					path: dir.path().to_path_buf(),
					kind: PrepErrorKind::Level(err),
				});

				malformed = true;

				vec![]
			}
		};

		let segs = match level::read::segs(segs.read_bytes()) {
			Ok(s) => s,
			Err(err) => {
				ctx.raise_error(PrepError {
					path: dir.path().to_path_buf(),
					kind: PrepErrorKind::Level(err),
				});

				malformed = true;

				vec![]
			}
		};

		let sidedefs = match level::read::sidedefs(sidedefs.read_bytes()) {
			Ok(s) => s,
			Err(err) => {
				ctx.raise_error(PrepError {
					path: dir.path().to_path_buf(),
					kind: PrepErrorKind::Level(err),
				});

				malformed = true;

				vec![]
			}
		};

		let subsectors = match level::read::ssectors(ssectors.read_bytes()) {
			Ok(ss) => ss,
			Err(err) => {
				ctx.raise_error(PrepError {
					path: dir.path().to_path_buf(),
					kind: PrepErrorKind::Level(err),
				});

				malformed = true;

				vec![]
			}
		};

		let vertices = match level::read::vertexes(vertexes.read_bytes()) {
			Ok(v) => v,
			Err(err) => {
				ctx.raise_error(PrepError {
					path: dir.path().to_path_buf(),
					kind: PrepErrorKind::Level(err),
				});

				malformed = true;

				vec![]
			}
		};

		let things_result = if behavior.is_none() {
			level::read::things_doom(things.read_bytes())
		} else {
			level::read::things_extended(things.read_bytes())
		};

		let things = match things_result {
			Ok(t) => t,
			Err(err) => {
				ctx.raise_error(PrepError {
					path: dir.path().to_path_buf(),
					kind: PrepErrorKind::Level(err),
				});

				malformed = true;

				vec![]
			}
		};

		let mut level = LevelDef::new(if behavior.is_some() {
			LevelFormat::Extended
		} else {
			LevelFormat::Doom
		});

		// As a placeholder in case map-info provides nothing.
		level.meta.name = dir.file_prefix().to_string().into();

		level.geom = LevelGeom {
			linedefs,
			sectordefs: sectors,
			sidedefs,
			vertdefs: vertices,
		};

		level.bsp = LevelBsp {
			nodes,
			segs,
			subsectors,
		};

		level.thingdefs = things;
		level.bounds = LevelDef::bounds(&level.geom.vertdefs);

		let err_handler = |err| {
			ctx.raise_error(PrepError {
				path: dir.path().to_path_buf(),
				kind: PrepErrorKind::Level(err),
			});
		};

		if level.validate(
			err_handler,
			|texname| self.last_by_nick::<Image>(texname).is_none(),
			|ednum| self.bp_by_ednum(ednum).is_none(),
		) > 0 || malformed
		{
			return Outcome::Err(());
		}

		Outcome::Ok(level)
	}
}
