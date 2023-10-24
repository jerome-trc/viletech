//! Type aliases and thin wrappers used throughout this crate but not exported,
//! kept in once place for cleanliness without polluting lib.rs.

use std::hash::BuildHasherDefault;

use cranelift::codegen::ir;
use rustc_hash::FxHasher;
use smallvec::SmallVec;

use crate::{
	ast,
	compile::LutSym,
	front::{
		sema::{CEval, SemaContext},
		sym::Symbol,
		tsys::TypeDatum,
	},
	intern::NameIx,
	mem::{APtr, NPtr},
};

pub(crate) type FxHamt<K, V> = im::HashMap<K, V, BuildHasherDefault<FxHasher>>;
pub(crate) type FxIndexMap<K, V> = indexmap::IndexMap<K, V, BuildHasherDefault<FxHasher>>;
pub(crate) type FxDashMap<K, V> = dashmap::DashMap<K, V, BuildHasherDefault<FxHasher>>;
pub(crate) type FxDashSet<K> = dashmap::DashSet<K, BuildHasherDefault<FxHasher>>;

pub(crate) type AbiType = cranelift::codegen::ir::Type;
#[allow(unused)]
pub(crate) type AbiTypes = SmallVec<[AbiType; 1]>;

pub(crate) type SymPtr = APtr<Symbol>;
#[allow(unused)]
pub(crate) type SymNPtr = NPtr<Symbol>;

pub(crate) type TypePtr = APtr<TypeDatum>;
pub(crate) type TypeNPtr = NPtr<TypeDatum>;

pub(crate) type IrPtr = APtr<ir::Function>;
pub(crate) type IrNPtr = NPtr<ir::Function>;

pub(crate) type CEvalIntrin = fn(&SemaContext, ast::ArgList) -> CEval;

pub(crate) type Scope = FxHamt<NameIx, LutSym>;
