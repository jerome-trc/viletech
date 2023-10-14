//! [`NameInterner`] and [`NameIx`].

use std::{
	borrow::Borrow,
	hash::{Hash, Hasher},
};

use append_only_vec::AppendOnlyVec;
use doomfront::rowan::GreenToken;

use crate::{FxDashMap, SyntaxToken};

/// An index into a [`NameInterner`]. Used for symbol lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct NameIx(u32);

/// A concurrent interner for [`IName`],
/// allowing 32-bit indices to be used as map keys in place of pointers.
#[derive(Debug)]
pub(crate) struct NameInterner {
	array: AppendOnlyVec<IName>,
	map: FxDashMap<IName, NameIx>,
}

impl NameInterner {
	#[must_use]
	pub(crate) fn intern(&self, token: &SyntaxToken) -> NameIx {
		self.add(token.green().to_owned())
	}

	#[must_use]
	fn add(&self, green: GreenToken) -> NameIx {
		let iname = IName(green);

		let vac = match self.map.entry(iname.clone()) {
			dashmap::mapref::entry::Entry::Occupied(occ) => return *occ.get(),
			dashmap::mapref::entry::Entry::Vacant(vac) => vac,
		};

		let ix = self.array.push(iname);
		debug_assert!(ix < (u32::MAX as usize));
		let ret = NameIx(ix as u32);
		vac.insert(ret);
		ret
	}
}

impl Default for NameInterner {
	fn default() -> Self {
		Self {
			array: AppendOnlyVec::new(),
			map: FxDashMap::default(),
		}
	}
}

/// Add a [`Borrow`] impl to [`GreenToken`] causing name literals to be treated
/// the same way as identifiers.
#[derive(Debug, Clone)]
#[repr(transparent)]
pub(crate) struct IName(pub(crate) GreenToken);

impl PartialEq for IName {
	fn eq(&self, other: &Self) -> bool {
		Borrow::<str>::borrow(self) == Borrow::<str>::borrow(other)
	}
}

impl Eq for IName {}

impl Hash for IName {
	fn hash<H: Hasher>(&self, state: &mut H) {
		Borrow::<str>::borrow(self).hash(state)
	}
}

impl Borrow<GreenToken> for IName {
	fn borrow(&self) -> &GreenToken {
		&self.0
	}
}

impl Borrow<str> for IName {
	fn borrow(&self) -> &str {
		let whole = self.0.text();

		if whole.ends_with('\'') {
			// Name literal
			&whole[1..(whole.len() - 1)]
		} else {
			whole
		}
	}
}

impl std::fmt::Display for IName {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.text().fmt(f)
	}
}
