//! A collection of modules and a table for accessing their symbols.

use std::sync::Arc;

use dashmap::DashMap;
use slotmap::SlotMap;

use super::{
	module::{ModuleSlotKey, SymbolSlotKey},
	Module, Symbol, SymbolKey, TypeInfo,
};

/// A collection of modules and a table for accessing their symbols.
#[derive(Debug)]
pub struct Project {
	pub(super) modules: SlotMap<ModuleSlotKey, Module>,
	/// In each value:
	/// - Field `0` is a key into `Self::modules`.
	/// - Field `1` is a key into [`Module::symbols`].
	pub(super) symbols: DashMap<SymbolKey, (ModuleSlotKey, SymbolSlotKey)>,
}

impl Project {
	pub fn get<S: Symbol>(&self, name: &str) -> Option<&Arc<S>> {
		let key = SymbolKey::new::<S>(name);

		if let Some(kvp) = self.symbols.get(&key) {
			self.modules[kvp.0].symbols[kvp.1].as_any().downcast_ref()
		} else {
			None
		}
	}

	pub(super) fn add_module(&mut self, module: Module) -> ModuleSlotKey {
		let mod_sk = self.modules.insert(module);

		for (sym_sk, symbol) in &self.modules[mod_sk].symbols {
			let key = SymbolKey::new::<TypeInfo>(&symbol.header().name);
			self.symbols.insert(key, (mod_sk, sym_sk));
		}

		mod_sk
	}

	/// Panics if no module named `name` exists, or if attempting to remove the corelib.
	pub fn remove_module(&mut self, name: &str) {
		assert_ne!(
			name,
			Module::CORELIB_NAME,
			"Attempted to remove the corelib."
		);

		let (key, _) = self
			.modules
			.iter()
			.find(|(_, module)| module.name() == name)
			.unwrap_or_else(|| panic!("No VZS module named: {name}"));

		self.modules.remove(key);

		self.symbols
			.retain(|_, (msk, _)| self.modules.contains_key(*msk));
	}

	/// Never removes the VZS corelib.
	pub fn retain<F>(&mut self, mut predicate: F)
	where
		F: FnMut(&Module) -> bool,
	{
		self.modules.retain(|_, module| {
			if module.name() == Module::CORELIB_NAME {
				return true;
			}

			predicate(module)
		});

		self.symbols
			.retain(|_, (msk, _)| self.modules.contains_key(*msk));
	}

	pub fn modules(&self) -> impl Iterator<Item = &Module> {
		self.modules.values()
	}
}

impl Default for Project {
	fn default() -> Self {
		let core = Module::core();

		let mut ret = Self {
			modules: SlotMap::default(),
			symbols: DashMap::with_capacity(core.symbols.len() * 3),
		};

		ret.add_module(core);

		ret
	}
}
