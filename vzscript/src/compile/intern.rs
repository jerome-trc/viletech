//! A [concurrent string interner](Interner) and some strong index types.

use std::hash::Hash;

use append_only_vec::AppendOnlyVec;
use util::rstring::RString;

use crate::FxDashMap;

#[derive(Debug)]
pub(crate) struct Interner<K, V: Hash + Eq> {
	/// TODO: Test if `Mutex<SegVec>` would make end-to-end compiles faster.
	array: AppendOnlyVec<V>,
	map: FxDashMap<V, K>,
}

impl<K, V: Hash + Eq> Default for Interner<K, V> {
	fn default() -> Self {
		Self {
			array: AppendOnlyVec::new(),
			map: FxDashMap::default(),
		}
	}
}

impl<K, V> Interner<K, V>
where
	K: From<u32> + Into<u32> + Copy,
	V: From<RString> + std::borrow::Borrow<str> + Hash + Eq + Clone,
{
	#[must_use]
	pub(crate) fn intern(&self, string: &str) -> K {
		if let Some(kvp) = self.map.get(string) {
			return *kvp.value();
		}

		let v = V::from(RString::new(string));
		let ix = self.array.push(v.clone());
		debug_assert!(ix < (u32::MAX as usize));
		let ret = K::from(ix as u32);
		self.map.insert(v, ret);
		ret
	}

	#[must_use]
	pub(crate) fn resolve(&self, key: K) -> &V {
		&self.array[Into::<u32>::into(key) as usize]
	}
}

/// An index into [`crate::compile::Compiler::names`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct NameIx(pub(crate) u32);

impl From<u32> for NameIx {
	fn from(value: u32) -> Self {
		Self(value)
	}
}

impl From<NameIx> for u32 {
	fn from(value: NameIx) -> Self {
		value.0
	}
}

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
