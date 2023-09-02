//! Types relevant to both the frontend and backend.

use std::{
	hash::{Hash, Hasher},
	ops::Deref,
};

use doomfront::zdoom::{decorate, inctree::IncludeTree};
use parking_lot::Mutex;
use sharded_slab::Slab;
use util::rstring::RString;

use crate::{
	front::{Scope, Symbol, SymbolPtr},
	issue::Issue,
	FxDashMap, Version,
};

#[derive(Debug)]
pub struct LibSource {
	pub name: String,
	pub version: Version,
	pub inctree: crate::IncludeTree,
	pub decorate: Option<IncludeTree<decorate::Syn>>,
}

#[derive(Debug)]
pub struct Compiler {
	pub(crate) stage: Stage,
	pub(crate) names: Interner<NameKey, ZName>,
	pub(crate) paths: Interner<PathKey, RString>,
	pub(crate) symbols: Slab<SymbolPtr>,
	pub(crate) global: Scope,
	pub(crate) sources: Vec<LibSource>,
	pub(crate) issues: Mutex<Vec<Issue>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stage {
	/// A newly-constructed [`Compiler`] is in this state.
	/// It should be passed to [`crate::front::declare_symbols`].
	Declaration,
	/// The compiler is in this state after calling [`crate::front::declare_symbols`].
	/// It should be passed to [`crate::front::finalize`].
	Finalize,
}

impl Compiler {
	#[must_use]
	pub fn new(sources: impl IntoIterator<Item = LibSource>) -> Self {
		let sources: Vec<_> = sources
			.into_iter()
			.map(|s| {
				assert!(
					!s.inctree.any_errors(),
					"cannot compile due to parse errors"
				);
				assert!(s.inctree.missing.is_empty());
				s
			})
			.collect();

		assert!(
			!sources.is_empty(),
			"`Compiler::new` needs at least one `LibSource`"
		);

		Self {
			stage: Stage::Declaration,
			names: Interner::default(),
			paths: Interner::default(),
			symbols: Slab::default(),
			global: Scope::default(),
			sources,
			issues: Mutex::default(),
		}
	}

	#[must_use]
	pub fn any_errors(&self) -> bool {
		let guard = self.issues.lock();
		guard.iter().any(|iss| iss.is_error())
	}

	/// If `Err` is returned, it gives back `symbol` along with the key to the
	/// symbol that would have been clobbered.
	pub(crate) fn declare(
		&self,
		scope: &mut Scope,
		name_k: NameKey,
		symbol: Symbol,
	) -> Result<SymbolKey, (Symbol, SymbolKey)> {
		use std::collections::hash_map;

		match scope.entry(name_k) {
			hash_map::Entry::Vacant(vac) => {
				let ix = self.symbols.insert(SymbolPtr::from(symbol)).unwrap();
				let key = SymbolKey::from(ix);
				vac.insert(key);
				Ok(key)
			}
			hash_map::Entry::Occupied(occ) => Err((symbol, *occ.get())),
		}
	}

	pub(crate) fn raise(&self, issue: Issue) {
		self.issues.lock().push(issue);
	}

	#[must_use]
	pub(crate) fn get_symbol(&self, key: SymbolKey) -> sharded_slab::Entry<SymbolPtr> {
		self.symbols.get(key.0).unwrap()
	}
}

#[derive(Debug)]
pub(crate) struct Interner<K, V: Hash + Eq> {
	slab: Slab<V>,
	map: FxDashMap<V, K>,
}

impl<K, V: Hash + Eq> Default for Interner<K, V> {
	fn default() -> Self {
		Self {
			slab: Slab::default(),
			map: FxDashMap::default(),
		}
	}
}

impl<K, V> Interner<K, V>
where
	K: From<usize> + Into<usize> + Copy,
	V: From<RString> + std::borrow::Borrow<str> + Hash + Eq + Clone,
{
	#[must_use]
	pub(crate) fn intern(&self, string: &str) -> K {
		if let Some(kvp) = self.map.get(string) {
			return *kvp.value();
		}

		let v = V::from(RString::new(string));
		let ix = self.slab.insert(v.clone()).unwrap();
		let ret = K::from(ix);
		self.map.insert(v, ret);
		ret
	}

	#[must_use]
	pub(crate) fn resolve(&self, key: K) -> Option<sharded_slab::Entry<V>> {
		self.slab.get(key.into())
	}
}

/// Index into [`Compiler::names`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct NameKey(usize);

impl From<usize> for NameKey {
	fn from(value: usize) -> Self {
		Self(value)
	}
}

impl From<NameKey> for usize {
	fn from(value: NameKey) -> Self {
		value.0
	}
}

/// Index into [`Compiler::paths`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PathKey(usize);

impl From<usize> for PathKey {
	fn from(value: usize) -> Self {
		Self(value)
	}
}

impl From<PathKey> for usize {
	fn from(value: PathKey) -> Self {
		value.0
	}
}

/// Index into [`Compiler::symbols`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SymbolKey(usize);

impl From<usize> for SymbolKey {
	fn from(value: usize) -> Self {
		Self(value)
	}
}

impl From<SymbolKey> for usize {
	fn from(value: SymbolKey) -> Self {
		value.0
	}
}

/// [`RString`] but with case-insensitive comparison and hashing.
#[derive(Debug, Clone)]
pub(crate) struct ZName(RString);

impl PartialEq for ZName {
	fn eq(&self, other: &Self) -> bool {
		self.0.deref().eq_ignore_ascii_case(other.0.as_ref())
	}
}

impl Eq for ZName {}

impl Hash for ZName {
	fn hash<H: Hasher>(&self, state: &mut H) {
		for c in self.0.deref().chars() {
			c.to_ascii_lowercase().hash(state);
		}
	}
}

impl std::borrow::Borrow<str> for ZName {
	fn borrow(&self) -> &str {
		self.0.deref()
	}
}

impl Deref for ZName {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.0.deref()
	}
}

impl From<RString> for ZName {
	fn from(value: RString) -> Self {
		Self(value)
	}
}

impl From<&RString> for ZName {
	fn from(value: &RString) -> Self {
		Self(value.clone())
	}
}
