//! Small types and type aliases that don't belong anywhere else.

use std::hash::BuildHasherDefault;

use dashmap::DashMap;
use rustc_hash::FxHasher;

pub type FxDashMap<K, V> = DashMap<K, V, BuildHasherDefault<FxHasher>>;
