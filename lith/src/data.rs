//! Pieces of data declared and inspected by the frontend.

use doomfront::rowan::TextRange;

use crate::{arena::CPtr, filetree::FileIx, intern::NameIx};

pub(crate) type SymPtr = CPtr<Symbol>;
// pub(crate) type ScopePtr = CPtr<Scope>;
pub(crate) type DefPtr = CPtr<Definition>;

#[derive(Debug)]
pub(crate) struct Symbol {
	pub(crate) location: Location,
	pub(crate) def: DefPtr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Location {
	/// The start is always the very start of a symbol's highest node.
	/// Use this to locate the part of the AST a symbol came from.
	pub(crate) span: TextRange,
	/// Index to an element in [`crate::compile::Compiler::sources`].
	pub(crate) lib_ix: u16,
	/// Index to an element in [`crate::filetree::FileTree::files`].
	pub(crate) file_ix: FileIx,
}

#[derive(Debug)]
pub(crate) enum Definition {
	Error,
	Pending,
	Function(Function),
}

#[derive(Debug)]
pub(crate) struct Function {
	pub(crate) visibility: Visibility,
	pub(crate) confine: Confinement,
	pub(crate) params: Vec<Parameter>,
}

#[derive(Debug)]
pub(crate) struct Parameter {
	pub(crate) name: NameIx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Visibility {
	/// Visible to all libraries.
	Export,
	/// Visible only to declaring library.
	Public,
	/// Visible within container only.
	Private,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Confinement {
	None,
	Ui,
	Sim,
}
