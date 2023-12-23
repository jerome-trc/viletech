//! Small types and type aliases that don't belong anywhere else.

use std::hash::BuildHasherDefault;

use dashmap::DashMap;
use indexmap::{IndexMap, IndexSet};
use rustc_hash::FxHasher;

pub type FxDashMap<K, V> = DashMap<K, V, BuildHasherDefault<FxHasher>>;
pub type FxDashView<K, V> = dashmap::ReadOnlyView<K, V, BuildHasherDefault<FxHasher>>;

pub type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;
pub type FxIndexSet<K> = IndexSet<K, BuildHasherDefault<FxHasher>>;
