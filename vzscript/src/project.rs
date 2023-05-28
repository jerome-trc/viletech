//! A collection of [libraries](Library) and a global symbol table.

use std::{borrow::Borrow, sync::Arc};

use dashmap::DashMap;

use crate::{
	library::Library,
	module,
	sym::{Symbol, SymbolKey, SymbolRef, SymbolStore},
};

/// A collection of [libraries](Library) and a global symbol table.
///
/// Intended usage of this structure is to:
/// - Keep one as part of your application's permanent state.
/// - Call [`Project::compile`] to perform a complete build.
/// - Call [`Project::clear`] to purge all compilation artifacts.
/// - Repeat as many times as necessary.
#[derive(Debug)]
pub struct Project {
	pub(super) populated: bool,
	pub(super) libs: Vec<Library>,
	pub(super) symbols: dashmap::ReadOnlyView<SymbolKey, Arc<dyn SymbolStore>>,
}

impl Project {
	#[must_use]
	pub fn get<'i, S: Symbol>(&self, input: impl Borrow<S::HashInput<'i>>) -> Option<SymbolRef<S>> {
		let key = SymbolKey::new::<S>(input);
		self.symbols.get(&key).map(|arc| SymbolRef::new(self, arc))
	}

	pub fn compile(&mut self, _: impl IntoIterator<Item = module::Builder>) {
		assert!(!self.populated);

		self.populated = true;

		unimplemented!()
	}

	pub fn clear(&mut self) {
		self.libs.clear();

		let symbols =
			std::mem::replace(&mut self.symbols, DashMap::default().into_read_only()).into_inner();
		symbols.clear();
		self.symbols = symbols.into_read_only();

		self.populated = false;
	}
}

impl Default for Project {
	fn default() -> Self {
		Self {
			populated: false,
			libs: vec![],
			symbols: DashMap::default().into_read_only(),
		}
	}
}
