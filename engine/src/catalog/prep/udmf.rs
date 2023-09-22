//! Functions for turning [UDMF] TEXTMAP and related lumps into levels.
//!
//! [UDMF]: https://doomwiki.org/wiki/UDMF

use util::Outcome;

use crate::{
	catalog::{dobj::Image, Catalog, PrepError, PrepErrorKind},
	level::LevelDef,
	vfs::FileRef,
};

use super::SubContext;

impl Catalog {
	/// Returns `None` if `dir` is unlikely to represent a UDMF level definition.
	pub(super) fn try_prep_level_udmf(
		&self,
		ctx: &SubContext,
		dir: FileRef,
	) -> Outcome<LevelDef, ()> {
		#![allow(unreachable_code)]

		let mut _behavior = None;
		let mut _dialogue = None;
		let mut _scripts = None;
		let mut textmap = None;
		let mut _znodes = None;

		for child in dir.children().unwrap() {
			match child.file_prefix() {
				"BEHAVIOR" => _behavior = Some(child),
				"DIALOGUE" => _dialogue = Some(child),
				"SCRIPTS" => _scripts = Some(child),
				"TEXTMAP" => textmap = Some(child),
				"ZNODES" => _znodes = Some(child),
				_ => {}
			}
		}

		let textmap = if let Some(tm) = textmap {
			tm
		} else {
			return Outcome::None;
		};

		let _ = match textmap.try_read_str() {
			Ok(s) => s,
			Err(_) => {
				ctx.raise_error(PrepError {
					path: dir.path().to_path_buf(),
					kind: PrepErrorKind::Unreadable(textmap.path().to_path_buf()),
				});

				return Outcome::Err(());
			}
		};

		let mut _level: LevelDef = unimplemented!("new UDMF-to-ECS code upcoming");

		let err_handler = |err| {
			ctx.raise_error(PrepError {
				path: dir.path().to_path_buf(),
				kind: PrepErrorKind::Level(err),
			});
		};

		if _level.validate(
			err_handler,
			|texname| self.last_by_nick::<Image>(texname).is_none(),
			|ednum| self.bp_by_ednum(ednum).is_none(),
		) > 0
		{
			return Outcome::Err(());
		}

		Outcome::Ok(_level)
	}
}
