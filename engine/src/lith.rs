//! Infrastructure powering the LithScript language.
//!
//! LithScript or just "Lith" is the bespoke scripting language embedded in
//! VileTech, designed to cover all its needs for custom user-defined behavior
//! as well as data definition. It is deliberately designed after GZDoom's ZScript,
//! as both a superset and subset of it, intended to correct its mistakes and
//! provide backwards compatibility via transpilation.

mod abi;
pub mod ast;
mod func;
mod inode;
mod module;
mod parse;
mod runtime;
mod symbol;
mod syn;
#[cfg(test)]
mod test;
mod tsys;

use std::{any::TypeId, collections::HashMap, sync::Arc};

use parking_lot::RwLock;

use crate::{
	data::{self, Catalog},
	VPath, VPathBuf,
};

pub use self::{
	func::{Flags as FunctionFlags, Function},
	inode::*,
	module::{Builder as ModuleBuilder, Handle, Module},
	parse::*,
	runtime::*,
	symbol::Symbol,
	syn::Syn,
	tsys::*,
};

/// Create one and store it permanently in your application's state.
/// Call [`clear`](Self::clear) if you need to perform a recompilation.
pub struct Project {
	catalog: Arc<RwLock<Catalog>>,
	sources: HashMap<VPathBuf, ariadne::Source>,
	modules: HashMap<String, Module>,
}

impl Project {
	#[must_use]
	pub fn new(catalog: Arc<RwLock<Catalog>>) -> Self {
		Self {
			catalog,
			sources: HashMap::default(),
			modules: HashMap::default(),
		}
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.sources.is_empty() && self.modules.is_empty()
	}

	pub fn clear(&mut self) {
		self.sources.clear();
		self.modules.clear();
	}
}

impl ariadne::Cache<VPath> for Project {
	fn fetch(&mut self, id: &VPath) -> Result<&ariadne::Source, Box<dyn std::fmt::Debug + '_>> {
		use ariadne::Source;

		if !self.sources.contains_key(id) {
			let catalog = self.catalog.read();

			let file = if let Some(f) = catalog.get_file(id) {
				f
			} else {
				return Err(Box::new(data::VfsError::NotFound(id.to_path_buf())));
			};

			let text = match file.try_read_str() {
				Ok(t) => t,
				Err(err) => {
					return Err(Box::new(err));
				}
			};

			let entry = self
				.sources
				.entry(id.to_path_buf())
				.or_insert_with(|| Source::from(text));

			Ok(entry)
		} else {
			// The weakness of `HashMap`'s API forces us to run the lookup again
			// to satisfy the borrow checker...[Rat] and it mildly annoys me
			Ok(&self.sources[id])
		}
	}

	fn display<'a>(&self, id: &'a VPath) -> Option<Box<dyn std::fmt::Display + 'a>> {
		Some(Box::new(id.display()))
	}
}

impl std::fmt::Debug for Project {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Project")
			.field("catalog", &self.catalog)
			.field("modules", &self.modules)
			.finish()
	}
}

/// No LithScript identifier in human-readable form may exceed this byte length.
/// Mind that Lith only allows ASCII alphanumerics and underscores for identifiers,
/// so this is also a character limit.
/// For reference, the following string is exactly 64 ASCII characters:
/// `_0_i_weighed_down_the_earth_through_the_stars_to_the_pavement_9_`
pub const MAX_IDENT_LEN: usize = 64;

/// In terms of values, not [quad-words](abi::QWord).
pub const MAX_PARAMS: usize = 16;

/// In terms of values, not [quad-words](abi::QWord).
pub const MAX_RETURNS: usize = 4;

#[derive(Debug)]
pub enum Error {
	/// Tried to retrieve a symbol from a module using an identifier that didn't
	/// resolve to anything.
	UnknownIdentifier,
	/// A caller tried to get a [`Handle`] to a symbol and found it,
	/// but requested a type different to that of its stored data.
	///
	/// [`Handle`]: Handle
	TypeMismatch { expected: TypeId, given: TypeId },
	/// Tried to retrieve a function from a module and found it, but failed to
	/// pass the generic arguments matching its signature.
	SignatureMismatch,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::UnknownIdentifier => write!(
				f,
				"Module symbol lookup failure; identifier didn't resolve to anything."
			),
			Self::TypeMismatch { expected, given } => {
				write!(
					f,
					"Type mismatch during symbol lookup. \
					Expected {expected:#?}, got {given:#?}.",
				)
			}
			Self::SignatureMismatch => {
				write!(f, "Symbol lookup failure; function signature mismatch.")
			}
		}
	}
}
