//! Type aliases and thin wrappers used throughout this crate but not exported,
//! kept in once place for cleanliness without polluting lib.rs.

use std::hash::BuildHasherDefault;

use cranelift::codegen::ir;
use rustc_hash::FxHasher;

use crate::{
	compile::{
		intern::NameIx,
		mem::{APtr, NPtr, OPtr},
		LutSym,
	},
	front::{sym::Symbol, tsys::TypeDef},
};

pub(crate) type FxHamt<K, V> = im::HashMap<K, V, BuildHasherDefault<FxHasher>>;
pub(crate) type FxIndexMap<K, V> = indexmap::IndexMap<K, V, BuildHasherDefault<FxHasher>>;
pub(crate) type FxDashMap<K, V> = dashmap::DashMap<K, V, BuildHasherDefault<FxHasher>>;
pub(crate) type FxDashSet<K> = dashmap::DashSet<K, BuildHasherDefault<FxHasher>>;

pub(crate) type AbiType = cranelift::codegen::ir::Type;

pub(crate) type SymPtr = APtr<Symbol>;
pub(crate) type SymOPtr = OPtr<Symbol>;

pub(crate) type TypePtr = APtr<TypeDef>;
pub(crate) type TypeOPtr = OPtr<TypeDef>;
pub(crate) type TypeNPtr = NPtr<TypeDef>;

pub(crate) type IrPtr = APtr<ir::Function>;
pub(crate) type IrOPtr = OPtr<ir::Function>;

pub(crate) type Scope = FxHamt<NameIx, LutSym>;
