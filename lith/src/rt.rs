//! The symbols making up the Lithica runtime.

use gc_arena::{Arena, Rootable};
use slotmap::SlotMap;

use crate::{dynval::DynVal, parse, table::Table, LexContext, ParseTree};

/// Context for Lithica compilation and execution.
///
/// Fully re-entrant; Lith has no global state.
pub struct Runtime {
	pub(crate) gc: Arena<Rootable! { State<'_> }>,
}

impl Runtime {
	pub fn exec<F, T, R>(&mut self, userdata: T, mut function: F) -> R
	where
		F: FnMut(&Runner<T>) -> R,
	{
		self.gc.mutate(|muta, root| {
			let runner = Runner {
				state: root,
				muta,
				userdata,
			};

			function(&runner)
		})
	}
}

impl Default for Runtime {
	fn default() -> Self {
		Self {
			gc: Arena::new(|muta| State {
				globals: Table::new(muta),
				registry: Registry {
					slots: SlotMap::default(),
				},
			}),
		}
	}
}

impl std::fmt::Debug for Runtime {
	fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		unimplemented!("TODO")
	}
}

unsafe impl Send for Runtime {}

#[derive(gc_arena::Collect, Debug)]
#[collect(no_drop)]
pub struct State<'rt> {
	pub globals: Table<'rt>,
	pub registry: Registry<'rt>,
}

#[derive(Debug)]
pub struct Registry<'rt> {
	slots: SlotMap<RegSlot, DynVal<'rt>>,
}

impl<'rt> Registry<'rt> {
	#[must_use]
	pub fn insert(&mut self, val: DynVal<'rt>) -> RegSlot {
		self.slots.insert(val)
	}

	pub fn remove(&mut self, slot: RegSlot) -> Option<DynVal<'rt>> {
		self.slots.remove(slot)
	}
}

unsafe impl gc_arena::Collect for Registry<'_> {
	fn trace(&self, cc: &gc_arena::Collection) {
		for v in self.slots.values() {
			v.trace(cc);
		}
	}
}

slotmap::new_key_type! {
	/// A key to a [`DynVal`] stored inside a [`Registry`].
	pub struct RegSlot;
}

pub struct Runner<'rt, T> {
	pub state: &'rt State<'rt>,
	pub muta: &'rt gc_arena::Mutation<'rt>,
	pub userdata: T,
}

impl<'rt, T> Runner<'rt, T> {
	pub fn load<'r, 'src>(
		&'r self,
		source: &'src str,
	) -> Result<Chunk<'rt, 'r, 'src, T>, Vec<parse::Error>>
	where
		'rt: 'r + 'src,
	{
		let ptree = doomfront::parse(source.as_ref(), parse::chunk, LexContext::default());

		if ptree.any_errors() {
			return Err(ptree.into_errors());
		}

		Ok(Chunk {
			runner: self,
			source: source.as_ref(),
			ptree,
		})
	}
}

pub struct Chunk<'rt: 'r + 'src, 'r, 'src, T> {
	pub(crate) runner: &'r Runner<'rt, T>,
	pub(crate) source: &'src str,
	pub(crate) ptree: ParseTree,
}
