//! "Dynamic value", used across HiLith.

use gc_arena::Gc;
use util::SmallString;

use crate::table::Table;

/// "Dynamic value", used across HiLith.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum DynVal<'rt> {
	Null,
	Bool(bool),
	I64(i64),
	F64(f64),
	String(Gc<'rt, SmallString>),
	Table(Table<'rt>),
}

impl DynVal<'_> {
	#[must_use]
	pub fn type_name(&self) -> &'static str {
		match self {
			Self::Null => "null",
			Self::Bool(_) => "bool",
			Self::I64(_) => "i64",
			Self::F64(_) => "f64",
			Self::String(_) => "string",
			Self::Table(_) => "table",
		}
	}
}

unsafe impl gc_arena::Collect for DynVal<'_> {
	fn trace(&self, cc: &gc_arena::Collection) {
		match self {
			Self::Null | Self::Bool(_) | Self::I64(_) | Self::F64(_) | Self::String(_) => {}
			Self::Table(table) => {
				table.trace(cc);
			}
		}
	}
}

impl PartialEq for DynVal<'_> {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Null, _) | (_, Self::Null) => false,
			(Self::Bool(l0), Self::Bool(r0)) => *l0 == *r0,
			(Self::I64(l0), Self::I64(r0)) => *l0 == *r0,
			(Self::F64(l0), Self::F64(r0)) => *l0 == *r0,
			(Self::String(l0), Self::String(r0)) => Gc::ptr_eq(*l0, *r0),
			(Self::Table(l0), Self::Table(r0)) => Gc::ptr_eq(l0.0, r0.0),
			_ => false,
		}
	}
}

impl std::hash::Hash for DynVal<'_> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		match self {
			Self::Null => {}
			Self::Bool(b) => b.hash(state),
			Self::I64(i) => i.hash(state),
			Self::F64(f) => (*f as i64).hash(state),
			Self::String(s) => s.hash(state),
			Self::Table(t) => t.0.as_ptr().hash(state),
		}
	}
}

const _ASSERT_DYNVAL_SIZE: () = {
	if std::mem::size_of::<DynVal>() != 16 {
		panic!();
	}
};
