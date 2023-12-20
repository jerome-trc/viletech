//! One of the HiLith primitive types; a combined growable array and hash map like Lua's.

use gc_arena::{lock::RefLock, Gc};
use rustc_hash::FxHashMap;

use super::dynval::DynVal;

/// One of the HiLith primitive types; a combined growable array and hash map like Lua's.
///
/// Note that unlike Lua, however, that HiLith has no concept of metatables.
#[derive(gc_arena::Collect, Debug, Clone, Copy)]
#[collect(no_drop)]
pub struct Table<'rt>(pub(crate) Gc<'rt, RefLock<TableData<'rt>>>);

impl<'rt> Table<'rt> {
	#[must_use]
	pub fn seq_len(&self) -> usize {
		self.0.borrow().array.len()
	}

	#[must_use]
	pub fn map_len(&self) -> usize {
		self.0.borrow().map.len()
	}

	#[must_use]
	pub fn total_len(&self) -> usize {
		let guard = self.0.borrow();
		guard.array.len() + guard.map.len()
	}

	#[must_use]
	pub(crate) fn new(muta: &gc_arena::Mutation<'rt>) -> Self {
		Self(Gc::new(
			muta,
			RefLock::new(TableData {
				array: vec![],
				map: FxHashMap::default(),
			}),
		))
	}
}

#[derive(gc_arena::Collect, Debug, Clone)]
#[collect(no_drop)]
pub(crate) struct TableData<'rt> {
	pub(crate) array: Vec<DynVal<'rt>>,
	pub(crate) map: FxHashMap<DynVal<'rt>, DynVal<'rt>>,
}
