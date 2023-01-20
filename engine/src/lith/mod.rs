//! Infrastructure powering the LithScript language.

pub mod ast;
mod func;
mod interop;
mod module;
pub mod parse;
pub mod syn;
mod tsys;
mod word;

use std::{collections::HashMap, path::Path, sync::Arc};

use indexmap::IndexMap;
use parking_lot::RwLock;

pub use interop::{Params, Returns};
pub use module::{Builder as ModuleBuilder, Handle, Module, OpenModule};
pub use tsys::*;

use crate::{
	data::{self, Catalog},
	VPathBuf,
};

use self::parse::ParseTree;

/// No LithScript identifier in human-readable form may exceed this byte length.
/// Mind that Lith only allows ASCII alphanumerics and underscores for identifiers,
/// so this is also a character limit.
/// For reference, the following string is exactly 64 ASCII characters:
/// `_0_i_weighed_down_the_earth_through_the_stars_to_the_pavement_9_`
pub const MAX_IDENT_LEN: usize = 64;

/// The maximum number of parameters which may be declared in a LithScript
/// function's signature; also the maximum number of arguments which may be passed.
/// Native functions are also bound to this limit.
pub const MAX_PARAMS: usize = 12;

/// The maximum number of return values which may be declared in a LithScript
/// function's signature; also the maximum number of values that a function
/// may return. Native functions are also bound to this limit.
pub const MAX_RETS: usize = 4;

/// Create one and store it permanently in your application's state.
/// Call [`clear`] if you need to perform a recompilation.
///
/// [`clear`]: Self::clear
pub struct Project {
	catalog: Arc<RwLock<Catalog>>,
	/// Used for generating error output.
	sources: HashMap<VPathBuf, ariadne::Source>,
	modules: IndexMap<String, Module>,
}

impl Project {
	#[must_use]
	pub fn new(catalog: Arc<RwLock<Catalog>>) -> Self {
		Self {
			catalog,
			sources: HashMap::default(),
			modules: IndexMap::default(),
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

impl ariadne::Cache<Path> for Project {
	fn fetch(&mut self, id: &Path) -> Result<&ariadne::Source, Box<dyn std::fmt::Debug + '_>> {
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

	fn display<'a>(&self, id: &'a Path) -> Option<Box<dyn std::fmt::Display + 'a>> {
		Some(Box::new(id.display()))
	}
}

impl std::fmt::Debug for Project {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Project")
			.field("modules", &self.modules)
			.finish()
	}
}

/// If a mount (i.e. a mod or game) has LithScript, it has at least one
/// "include tree". This is an unordered collection of source files brought
/// together via `#include` preprocessor directives.
///
/// A manifest is its own include tree, and the manifest may dictate another file
/// to act as the root for the mount's other include tree, which can contain
/// game-modifying scripts.
pub struct IncludeTree {
	pub roots: Vec<ParseTree>,
}

#[derive(Debug)]
pub enum Error {
	/// Tried to retrieve a symbol from a module using an identifier that didn't
	/// resolve to anything.
	UnknownIdentifier,
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
			Self::SignatureMismatch => write!(
				f,
				"Module symbol lookup failure; function signature mismatch."
			),
		}
	}
}
