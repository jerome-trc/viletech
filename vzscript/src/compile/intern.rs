//! A [concurrent string interner](Interner) and some strong index types.

use std::{
	borrow::Borrow,
	hash::{Hash, Hasher},
	ops::Deref,
};

use append_only_vec::AppendOnlyVec;
use doomfront::{
	rowan::{GreenToken, GreenTokenData, SyntaxToken},
	LangExt,
};
use util::rstring::RString;

use crate::FxDashMap;

// Interner ////////////////////////////////////////////////////////////////////

/// A concurrent interner for [`IName`],
/// allowing 32-bit indices to be used as map keys in place of pointers.
#[derive(Debug)]
pub(crate) struct NameInterner {
	array: AppendOnlyVec<IName>,
	map: FxDashMap<IName, NameIx>,
}

impl NameInterner {
	#[must_use]
	pub(crate) fn intern<L: LangExt>(&self, token: &SyntaxToken<L>) -> NameIx {
		let green = INameData(token.green());

		#[repr(transparent)]
		struct INameData<'g>(&'g GreenTokenData);

		impl<'g> Borrow<INameData<'g>> for IName {
			fn borrow(&self) -> &INameData<'g> {
				// SAFETY: This type is `repr(transparent)` over `GreenTokenData`.
				unsafe { std::mem::transmute::<_, _>(self.0.deref()) }
			}
		}

		impl Hash for INameData<'_> {
			fn hash<H: Hasher>(&self, state: &mut H) {
				for c in self.0.text().chars() {
					c.to_ascii_lowercase().hash(state);
				}
			}
		}

		impl PartialEq for INameData<'_> {
			fn eq(&self, other: &Self) -> bool {
				self.0.text().eq_ignore_ascii_case(other.0.text())
			}
		}

		impl Eq for INameData<'_> {}

		if let Some(kvp) = self.map.get(&green) {
			return *kvp.value();
		}

		self.add(green.0.to_owned())
	}

	#[must_use]
	pub(crate) fn intern_str(&self, string: &str) -> NameIx {
		if let Some(kvp) = self.map.get(string) {
			return *kvp.value();
		}

		self.add(GreenToken::new(crate::Syn::Ident.into(), string))
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

impl Default for NameInterner {
	fn default() -> Self {
		Self {
			array: AppendOnlyVec::new(),
			map: FxDashMap::default(),
		}
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

		if whole.ends_with('\'') {
			// Name literal
			&whole[1..(whole.len() - 1)]
		} else if whole.ends_with('"') {
			// String literal
			let start = whole.chars().position(|c| c == '"').unwrap();
			&whole[(start + 1)..(whole.len() - 1)]
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

impl std::fmt::Display for IName {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.text().fmt(f)
	}
}

// Indices /////////////////////////////////////////////////////////////////////

/// An index into [`crate::compile::Compiler::names`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct NameIx(pub(crate) u32);

impl From<NameIx> for i32 {
	/// For Cranelift, which only deals in two's complement.
	fn from(value: NameIx) -> Self {
		value.0 as i32
	}
}

/// An index into [`crate::compile::Compiler::symbols`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SymbolIx(pub(crate) u32);

/// A [`NameIx`] with an attached "symbol-space" discriminant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum NsName {
	Type(NameIx),
	/// All VZScript names use this symbol-space.
	Value(NameIx),

	FlagDef(NameIx),
	Property(NameIx),
}
