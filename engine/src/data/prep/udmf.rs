//! Functions for turning [UDMF] TEXTMAP and related lumps into levels.

use crate::data::{detail::Outcome, vfs::FileRef, Catalog};

use super::SubContext;

impl Catalog {
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

		Outcome::None // TODO: Soon!
	}
}
