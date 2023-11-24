use std::{
	borrow::Borrow,
	hash::{Hash, Hasher},
};

use doomfront::rowan::{cursor::SyntaxToken, GreenToken, SyntaxKind};
use util::pushvec::PushVec;

use crate::types::FxDashMap;

/// An index into a [`NameInterner`]. Used for symbol lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct NameIx(u32);

/// A [`NameIx`] with a namespace discriminant attached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum NsName {
	Type(NameIx),
	Value(NameIx),
}

impl NsName {
	#[must_use]
	#[allow(unused)]
	fn index(self) -> NameIx {
		match self {
			Self::Type(ix) | Self::Value(ix) => ix,
		}
	}
}

/// A concurrent interner for [`IName`],
/// allowing 32-bit indices to be used as map keys in place of pointers.
#[derive(Debug, Default)]
pub(crate) struct NameInterner {
	array: PushVec<IName>,
	map: FxDashMap<IName, NameIx>,
}

impl NameInterner {
	#[must_use]
	pub(crate) fn intern(&self, token: &SyntaxToken) -> NameIx {
		let green = unsafe { std::mem::transmute::<_, &GreenTokenData>(token.green()) };

		if let Some(kvp) = self.map.get(green) {
			return *kvp.value();
		}

		self.add(green.0.to_owned())
	}

	#[must_use]
	pub(crate) fn intern_str(&self, string: &str) -> NameIx {
		if let Some(kvp) = self.map.get(string) {
			return *kvp.value();
		}

		self.add(GreenToken::new(SyntaxKind(0), string))
	}

	#[must_use]
	pub(crate) fn resolve(&self, ns_name: NsName) -> &str {
		self.array[ns_name.index().0 as usize].0.text()
	}

	#[must_use]
	fn add(&self, token: GreenToken) -> NameIx {
		let v = IName(token);
		let ix = self.array.push(v.clone());
		debug_assert!(ix < (u32::MAX as usize));
		let ret = NameIx(ix as u32);
		self.map.insert(v, ret);
		ret
	}
}

// IName ///////////////////////////////////////////////////////////////////////

/// "Interned name"; a [`GreenToken`] with case-insensitive comparison and hashing.
#[derive(Debug, Clone)]
#[repr(transparent)]
pub(crate) struct IName(pub(crate) GreenToken);

impl PartialEq for IName {
	fn eq(&self, other: &Self) -> bool {
		let self_text: &str = self.borrow();
		let o_text: &str = other.borrow();
		self_text.eq_ignore_ascii_case(o_text)
	}
}

impl Eq for IName {}

impl Hash for IName {
	fn hash<H: Hasher>(&self, state: &mut H) {
		let text: &str = self.borrow();

		for c in text.chars() {
			c.to_ascii_lowercase().hash(state);
		}
	}
}

impl Borrow<str> for IName {
	fn borrow(&self) -> &str {
		let whole = self.0.text();

		if whole.ends_with(['\'', '"']) {
			// Name or string literal respectively
			&whole[1..(whole.len() - 1)]
		} else {
			whole
		}
	}
}

impl From<GreenToken> for IName {
	fn from(value: GreenToken) -> Self {
		Self(value)
	}
}

impl From<&GreenToken> for IName {
	fn from(value: &GreenToken) -> Self {
		Self(value.clone())
	}
}

impl Borrow<GreenTokenData> for IName {
	fn borrow(&self) -> &GreenTokenData {
		unsafe {
			std::mem::transmute::<&doomfront::rowan::GreenTokenData, &GreenTokenData>(
				self.0.borrow(),
			)
		}
	}
}

impl std::fmt::Display for IName {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.text().fmt(f)
	}
}

#[derive(Debug)]
#[repr(transparent)]
struct GreenTokenData(doomfront::rowan::GreenTokenData);

impl PartialEq for GreenTokenData {
	fn eq(&self, other: &Self) -> bool {
		self.0.text().eq_ignore_ascii_case(other.0.text())
	}
}

impl Eq for GreenTokenData {}

impl Hash for GreenTokenData {
	fn hash<H: Hasher>(&self, state: &mut H) {
		for c in self.0.text().chars() {
			c.to_ascii_lowercase().hash(state);
		}
	}
}
