//! Information used by the semantic mid-section but not the backend.

use crossbeam::atomic::AtomicCell;
use doomfront::rowan::{TextRange, TextSize};

use crate::{
	rti,
	tsys::{FuncType, TypeDef, TypeHandle},
	vir,
};

use super::{intern::SymbolIx, Scope};

#[derive(Debug)]
pub(crate) struct Symbol {
	/// `None` for primitives and namespaces.
	pub(crate) location: Option<Location>,
	pub(crate) kind: SymbolKind,
	/// Determines whether name lookups need to happen through all namespaces
	/// (to imitate the reference implementation's global namespace) or just
	/// the library's own.
	pub(crate) zscript: bool,
	/// TODO: Test if boxing this leads to better end-to-end performance.
	pub(crate) scope: Scope,
	pub(crate) definition: AtomicCell<DefIx>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DefIx {
	None,
	Pending,
	Error,
	/// An index into [`crate::compile::Compiler::defs`].
	Some(u32),
}

impl DefIx {
	#[allow(unused)]
	const STATIC_ASSERT_LOCK_FREE: () = {
		assert!(AtomicCell::<Self>::is_lock_free());
	};
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Location {
	/// The start is always the very start of a symbol's highest node.
	/// Use this to locate the part of the AST a symbol came from.
	pub(crate) span: TextRange,
	/// Combine with [`Self::span`]'s start for reporting issues.
	pub(crate) short_end: TextSize,
	/// Index to an element in [`crate::compile::Compiler::sources`].
	pub(crate) lib_ix: u16,
	/// Index to an element in [`crate::IncludeTree::files`].
	pub(crate) file_ix: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SymbolKind {
	Class,
	Enum,
	Field,
	FlagDef,
	Function,
	/// Index within is to an element in [`crate::compile::Compiler::namespaces`].
	Import(u16),
	Mixin,
	Primitive,
	Property,
	/// Index within is to an element in [`crate::compile::Compiler::symbols`].
	Rename(SymbolIx),
	Struct,
	Union,
	Value,
}

impl SymbolKind {
	#[must_use]
	pub(crate) fn user_facing_name(&self) -> &'static str {
		match self {
			Self::Class => "class",
			Self::Enum => "enum",
			Self::Field => "field",
			Self::FlagDef => "flagDef",
			Self::Function => "function",
			Self::Import(_) => "namespace",
			Self::Mixin => "mixin",
			Self::Primitive => "primitive type",
			Self::Property => "property",
			Self::Rename(_) => "type alias",
			Self::Struct => "struct",
			Self::Union => "union",
			Self::Value => "constant",
		}
	}
}

#[derive(Debug)]
pub(crate) enum Definition {
	Constant {
		tdef: rti::Handle<TypeDef>,
		init: vir::Block,
	},
	Data {
		typedef: rti::Handle<TypeDef>,
		init: vir::Block,
	},
	Field {
		typedef: rti::Handle<TypeDef>,
	},
	Function {
		tdef: TypeHandle<FuncType>,
		code: FunctionCode,
	},
	Type(rti::Record),
}

#[derive(Debug)]
pub(crate) enum FunctionCode {
	Native(&'static str),
	Ir(vir::Block),
}
