//! Functions for reading data objects from ZDoom's PK3 and PK7 archives.

use util::Outcome;

use crate::catalog::Catalog;

use super::SubContext;

impl Catalog {
	pub(super) fn prep_pass1_pk(&self, _ctx: &SubContext) -> Outcome<(), ()> {
		// TODO: Soon!
		Outcome::None
	}
}
