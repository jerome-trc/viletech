//! Type aliases and thin wrappers used throughout this crate but not exported,
//! kept in once place for cleanliness without polluting lib.rs.

use std::hash::BuildHasherDefault;

use cranelift::codegen::ir;
use rustc_hash::FxHasher;
use smallvec::SmallVec;

use crate::{
	ast,
	compile::{
		intern::NameIx,
		mem::{APtr, NPtr, OPtr},
		LutSym,
	},
	front::{
		sema::{CEval, SemaContext},
		sym::Symbol,
		tsys::TypeDef,
	},
};

pub(crate) type FxHamt<K, V> = im::HashMap<K, V, BuildHasherDefault<FxHasher>>;
pub(crate) type FxIndexMap<K, V> = indexmap::IndexMap<K, V, BuildHasherDefault<FxHasher>>;
pub(crate) type FxDashMap<K, V> = dashmap::DashMap<K, V, BuildHasherDefault<FxHasher>>;
pub(crate) type FxDashSet<K> = dashmap::DashSet<K, BuildHasherDefault<FxHasher>>;

pub(crate) type AbiType = cranelift::codegen::ir::Type;
#[allow(unused)]
pub(crate) type AbiTypes = SmallVec<[AbiType; 1]>;

pub(crate) type SymPtr = APtr<Symbol>;
pub(crate) type SymOPtr = OPtr<Symbol>;
#[allow(unused)]
pub(crate) type SymNPtr = NPtr<Symbol>;

pub(crate) type TypePtr = APtr<TypeDef>;
pub(crate) type TypeOPtr = OPtr<TypeDef>;
pub(crate) type TypeNPtr = NPtr<TypeDef>;

pub(crate) type IrPtr = APtr<ir::Function>;
pub(crate) type IrOPtr = OPtr<ir::Function>;
pub(crate) type IrNPtr = NPtr<ir::Function>;

pub(crate) type CEvalIntrin = fn(&SemaContext, ast::ArgList) -> CEval;

pub(crate) type Scope = FxHamt<NameIx, LutSym>;
