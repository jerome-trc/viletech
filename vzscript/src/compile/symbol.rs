//! Information used by the semantic mid-section but not the backend.

use std::{any::Any, sync::Arc};

use arc_swap::ArcSwap;
use crossbeam::atomic::AtomicCell;
use doomfront::rowan::{TextRange, TextSize};
use smallvec::SmallVec;

use crate::{
	ast,
	back::AbiTypes,
	rti,
	sema::{CEval, CEvalVec},
	tsys::{
		ClassType, EnumType, FuncType, PrimitiveType, StructType, TypeDef, TypeHandle, UnionType,
	},
	vir,
};

use super::{intern::SymbolIx, Compiler, Scope};

#[derive(Debug)]
pub(crate) struct Symbol {
	pub(crate) location: Option<Location>,
	/// Dictates what kind of definition Sema. will try to provide this with.
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

/// See [`Symbol::kind`]; this isn't used for anything else.
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
		typedef: TypeHandle<FuncType>,
		code: FunctionCode,
	},
	Type {
		record: rti::Record,
	},
}

#[derive(Debug)]
pub(crate) enum FunctionCode {
	Builtin(unsafe extern "C" fn(AbiTypes) -> AbiTypes),
	/// The string slice parameter is a path to the calling file.
	BuiltinCEval(fn(&Compiler, &str, ast::ArgList) -> CEval),
	Native(&'static str),
	Ir(vir::Block),
}
