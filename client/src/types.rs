use std::hash::BuildHasherDefault;

use rustc_hash::FxHasher;

pub type FxDashMap<K, V> = dashmap::DashMap<K, V, BuildHasherDefault<FxHasher>>;
