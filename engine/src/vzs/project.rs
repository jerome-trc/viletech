//! A collection of modules and a table for accessing their symbols.

use std::{borrow::Borrow, collections::HashMap, sync::Arc};

use slotmap::SlotMap;

use super::{
	module::{self, ModuleSlotKey, SymbolSlotKey},
	Module, SymbolHash, SymbolKey,
};

/// A collection of modules and a table for accessing their symbols.
#[derive(Debug, Default)]
pub struct Project {
	pub(super) modules: SlotMap<ModuleSlotKey, Module>,
	/// In each value:
	/// - Field `0` is a key into `Self::modules`.
	/// - Field `1` is a key into [`Module::symbols`].
	pub(super) symbols: HashMap<SymbolKey, (ModuleSlotKey, SymbolSlotKey)>,
	/// Upon a project clear, every existing module is compressed to a byte array
	/// and stored here; each is keyed by name and keeps a checksum of all its
	/// text source.
	///
	/// When a project is compiled again, each module submitted for compilation
	/// checks its hash against the hash in this map (if its name is a key in this
	/// map). A matching hash means the module did not change between compilations
	/// and can be deserialized from the blob, saving time on checks and code-gen.
	pub(super) cache: HashMap<String, module::Blob>,
}

impl Project {
	#[must_use]
	pub fn get<'i, S: SymbolHash<'i>>(&self, input: impl Borrow<S::HashInput>) -> Option<&Arc<S>> {
		let key = SymbolKey::new::<S>(input);

		if let Some(kvp) = self.symbols.get(&key) {
			self.modules[kvp.0].symbols[kvp.1].as_any().downcast_ref()
		} else {
			None
		}
	}

	/// Panics if any [symbol handles] still point to a stored module.
	///
	/// [symbol handles]: crate::vzs::Handle
	pub fn clear(&mut self) {
		self.cache.clear();

		for (_, module) in self.modules.drain() {
			for (_, sym) in &module.symbols {
				if Arc::strong_count(sym) > 0 || Arc::weak_count(sym) > 0 {
					panic!(
						"Tried to clear a VZS project with outstanding handle to symbol: {}",
						&sym.header().name
					);
				}
			}

			// TODO: Serialization strategy used for module caching.
			// Dependent on what Cranelift's developers decide to do.
		}

		self.symbols.clear();
	}

	pub(super) fn _add_module(&mut self, module: Module) {
		let mod_sk = self.modules.insert(module);

		for (sym_sk, symbol) in &self.modules[mod_sk].symbols {
			self.symbols.insert(symbol.key(), (mod_sk, sym_sk));
		}
	}

	pub fn modules(&self) -> impl Iterator<Item = &Module> {
		self.modules.values()
	}
}
