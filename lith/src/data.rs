//! Pieces of data declared and inspected by the frontend.

use doomfront::rowan::TextRange;

use crate::{arena::CPtr, filetree::FileIx, intern::NameIx, Scope};

pub(crate) type SymPtr = CPtr<Symbol>;
pub(crate) type DefPtr = CPtr<Definition>;

#[derive(Debug)]
pub(crate) struct Symbol {
	pub(crate) location: Location,
	pub(crate) def: DefPtr,
}

impl Drop for Symbol {
	fn drop(&mut self) {
		if let Some(ptr) = self.def.as_ptr() {
			unsafe {
				std::ptr::drop_in_place(ptr.as_ptr());
			}
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Location {
	/// The start is always the very start of a symbol's highest node.
	/// Use this to locate the part of the AST a symbol came from.
	pub(crate) span: TextRange,
	/// Index to an element in [`crate::filetree::FileTree::graph`].
	pub(crate) file_ix: FileIx,
}

#[derive(Debug)]
pub(crate) enum Definition {
	Function(Function),
	MassImport(Scope),
	SymConst(SymConst),
}

// Common details //////////////////////////////////////////////////////////////

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Visibility {
	/// Visible to all libraries.
	/// Corresponds to the `public` keyword.
	Export,
	/// Visible only to declaring library.
	/// Corresponds to the absence of a visibility specifier.
	#[default]
	Default,
	/// Visible within container only.
	/// Corresponds to the `private` keyword.
	Hidden,
}

/// The "confinement" system is designed for use in games which have both a
/// single- and multi-player component and need some symbols to operate only
/// "client-side" without affecting the gameplay simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Confinement {
	None,
	Ui,
	Sim,
}

// Function ////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub(crate) struct Function {
	pub(crate) flags: FunctionFlags,
	pub(crate) visibility: Visibility,
	pub(crate) confine: Confinement,
	pub(crate) inlining: Inlining,
	pub(crate) params: Vec<Parameter>,
	pub(crate) return_type: SymPtr,
}

#[derive(Debug)]
pub(crate) struct Parameter {
	pub(crate) name: NameIx,
	pub(crate) type_spec: SymPtr,
}

bitflags::bitflags! {
	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	pub(crate) struct FunctionFlags: u8 {
		/// The script writer has hinted that the function is unlikely to be called.
		const COLD = 1 << 0;
	}
}

#[derive(Debug, Default)]
pub(crate) enum Inlining {
	/// Corresponds to the annotation `#[inline(never)]`.
	Never,
	#[default]
	Normal,
	/// Corresponds to the annotation `#[inline]`.
	More,
	/// Corresponds to the annotation `#[inline(extra)]`.
	Extra,
}

// SymConst ////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub(crate) struct SymConst {
	pub(crate) visibility: Visibility,
	pub(crate) type_spec: SymPtr,
}
