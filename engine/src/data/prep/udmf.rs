//! Functions for turning [UDMF] TEXTMAP and related lumps into levels.
//!
//! [UDMF]: https://doomwiki.org/wiki/UDMF

use crate::{
	data::{
		detail::Outcome, dobj::Level, vfs::FileRef, Catalog, LevelError, PrepError, PrepErrorKind,
	},
	udmf,
};

use super::SubContext;

impl Catalog {
	/// Returns `None` if `dir` is unlikely to represent a UDMF level definition.
	pub(super) fn try_prep_level_udmf(&self, ctx: &SubContext, dir: FileRef) -> Outcome<Level, ()> {
		let mut _behavior = None;
		let mut _dialogue = None;
		let mut _scripts = None;
		let mut textmap = None;
		let mut _znodes = None;

		for child in dir.child_refs().unwrap() {
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

		let source = match textmap.try_read_str() {
			Ok(s) => s,
			Err(_) => {
				ctx.errors.lock().push(PrepError {
					path: dir.path.to_path_buf(),
					kind: PrepErrorKind::Level(LevelError::UnreadableFile(
						textmap.path().to_path_buf(),
					)),
				});

				return Outcome::Err(());
			}
		};

		match udmf::parse_textmap(source) {
			Ok(level) => Outcome::Ok(level),
			Err(errs) => {
				let mut ctx_errs = ctx.errors.lock();

				for err in errs {
					ctx_errs.push(PrepError {
						path: dir.path.to_path_buf(),
						kind: PrepErrorKind::Level(LevelError::TextmapParse(err)),
					})
				}

				Outcome::Err(())
			}
		}
	}
}
