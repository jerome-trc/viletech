//! Functions for reading data objects from ZDoom's PK3 and PK7 archives.

use crate::{data::Catalog, Outcome};

use super::SubContext;

impl Catalog {
	pub(super) fn prep_pass1_pk(&self, _ctx: &SubContext) -> Outcome<(), ()> {
		// TODO: Soon!
		Outcome::None
	}
}
