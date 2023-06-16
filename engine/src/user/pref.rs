//! A "preference" is a single engine/game/mod setting configurable by the user.

use std::{collections::HashMap, marker::PhantomData, mem::ManuallyDrop, ops::Deref, sync::Arc};

use crossbeam::atomic::AtomicCell;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use serde::{ser::SerializeStruct, Serialize};

use crate::RgbaF32;

use super::Error;

#[derive(Debug, Default)]
pub struct PrefPreset {
	/// Assigned to this preset by the user.
	/// Must be between 2 and 64 characters long, but is otherwise unrestricted.
	/// Stored so the [core](super::UserCore) knows which directory to write to.
	name: String,
	/// Each mount that declares any prefs (or CVars for that matter) gets one.
	/// Order is the same as the user's chosen load order.
	namespaces: Vec<PrefNamespace>,
	/// Keys take the format `namespace_name/pref_name`.
	/// In each value:
	/// - `::0` is an index into `prefs`.
	/// - `::1` is an index into [`PrefNamespace::prefs`].
	map: HashMap<String, (usize, usize)>,
}

impl PrefPreset {
	pub(super) fn get<P: PrefValue>(&self, id: &str) -> Result<Handle<P>, Error> {
		let ipair = match self.map.get(id) {
			Some(e) => e,
			None => return Err(Error::PrefNotFound(id.to_string())),
		};

		let pref = &self.namespaces[ipair.0].prefs[ipair.1];

		if P::KIND == pref.kind() {
			Ok(Handle::new(pref.clone()))
		} else {
			Err(Error::TypeMismatch {
				expected: pref.kind(),
				given: P::KIND,
			})
		}
	}

	#[must_use]
	pub(super) fn _get_namespace(&self, id: &str) -> Option<&PrefNamespace> {
		self.namespaces.iter().find(|&ns| ns._id == id)
	}

	#[must_use]
	pub(super) fn _all_namespaces(&self) -> &[PrefNamespace] {
		&self.namespaces
	}

	#[must_use]
	pub(super) fn name(&self) -> &str {
		&self.name
	}

	#[must_use]
	pub(super) fn new(name: String) -> Self {
		Self {
			name,
			namespaces: vec![],
			map: HashMap::default(),
		}
	}

	pub(super) fn _add_namespaces(&mut self, mut namespaces: Vec<PrefNamespace>) {
		self.namespaces.append(&mut namespaces);
		self._update();
	}

	pub(super) fn _truncate(&mut self, len: usize) {
		self.namespaces.truncate(len);
		self._update();
	}

	fn _update(&mut self) {
		self.map.clear();

		for (ns_i, namespace) in self.namespaces.iter().enumerate() {
			for (p_i, pref) in namespace.prefs.iter().enumerate() {
				self.map.insert(pref.id().to_string(), (ns_i, p_i));
			}
		}
	}
}

/// Each mount that declares any preferences (or CVars for that matter) gets one.
#[derive(Debug)]
pub struct PrefNamespace {
	/// Stored so the [core](super::UserCore) knows which .toml file to write to.
	/// Derived from the [mount's ID](vfs::MountInfo::id).
	_id: String,
	/// Keys are pref IDs; these are restricted to ASCII alphanumerics and
	/// underscores, must start with an ASCII letter or underscore, and can not
	/// be longer than 64 characters.
	///
	/// Values are alphabetically sorted.
	prefs: Vec<Arc<Pref>>,
}

impl PrefNamespace {
	#[must_use]
	pub(super) fn _id(&self) -> &str {
		&self._id
	}

	#[must_use]
	pub(super) fn _to_toml(&self) -> toml::value::Table {
		let mut ret = toml::value::Table::with_capacity(self.prefs.len());

		for pref in &self.prefs {
			ret.insert(pref.id.clone(), pref._to_toml());
		}

		ret
	}
}

/// A single engine/game/mod setting configurable by the user.
/// It is a direct counterpart to GZDoom's console variables, a.k.a. "CVars".
pub struct Pref {
	id: String,
	kind: PrefKind,
	data: PrefData,
	flags: PrefFlags,
	// TODO: On-change callback which can invoke Lith.
}

impl Pref {
	#[must_use]
	pub fn id(&self) -> &str {
		&self.id
	}

	#[must_use]
	pub fn flags(&self) -> PrefFlags {
		self.flags
	}

	#[must_use]
	pub fn kind(&self) -> PrefKind {
		self.kind
	}

	/// Returns `true` if this preference has the same storage type and value as
	/// another.
	#[must_use]
	pub fn eq_val(&self, other: &Self) -> bool {
		unsafe {
			match (self.kind, other.kind) {
				(PrefKind::Bool, PrefKind::Bool) => {
					self.data.boolean.value.load() == other.data.boolean.value.load()
				}
				(PrefKind::Int, PrefKind::Int) => {
					self.data.int.value.load() == other.data.int.value.load()
				}
				(PrefKind::Float, PrefKind::Float) => {
					self.data.float.value.load() == other.data.float.value.load()
				}
				(PrefKind::Color, PrefKind::Color) => {
					self.data.color.value.load() == other.data.color.value.load()
				}
				(PrefKind::String, PrefKind::String) => {
					self.data.string.value.read().deref() == other.data.string.value.read().deref()
				}
				_ => false,
			}
		}
	}

	/// Returns `true` if this preference's value is the same as its default.
	#[must_use]
	pub fn eq_default(&self) -> bool {
		unsafe {
			match self.kind {
				PrefKind::Bool => self.data.boolean.value.load() == self.data.boolean.default,
				PrefKind::Int => self.data.int.value.load() == self.data.int.default,
				PrefKind::Float => self.data.float.value.load() == self.data.float.default,
				PrefKind::Color => self.data.color.value.load() == self.data.color.default,
				PrefKind::String => {
					*self.data.string.value.read().deref() == self.data.string.default
				}
			}
		}
	}

	#[must_use]
	pub fn is_saved(&self) -> bool {
		self.flags.contains(PrefFlags::SAVED)
	}

	/// Both the current value and default are set to `default`. Reading the FS
	/// for the real current value (if any) is left to the caller.
	#[must_use]
	pub(super) fn _new_bool(id: String, default: bool, flags: PrefFlags) -> Arc<Self> {
		Arc::new(Self {
			id,
			kind: PrefKind::Bool,
			data: PrefData {
				boolean: ManuallyDrop::new(BoolStore {
					value: AtomicCell::new(default),
					default,
				}),
			},
			flags,
		})
	}

	/// Both the current value and default are set to `default`. Reading the FS
	/// for the real current value (if any) is left to the caller.
	#[must_use]
	pub(super) fn _new_int(id: String, default: i64, flags: PrefFlags) -> Arc<Self> {
		Arc::new(Self {
			id,
			kind: PrefKind::Int,
			data: PrefData {
				int: ManuallyDrop::new(I64Store {
					value: AtomicCell::new(default),
					default,
				}),
			},
			flags,
		})
	}

	/// Both the current value and default are set to `default`. Reading the FS
	/// for the real current value (if any) is left to the caller.
	#[must_use]
	pub(super) fn _new_float(id: String, default: f64, flags: PrefFlags) -> Arc<Self> {
		Arc::new(Self {
			id,
			kind: PrefKind::Float,
			data: PrefData {
				float: ManuallyDrop::new(F64Store {
					value: AtomicCell::new(default),
					default,
				}),
			},
			flags,
		})
	}

	/// Both the current value and default are set to `default`. Reading the FS
	/// for the real current value (if any) is left to the caller.
	#[must_use]
	pub(super) fn _new_color(id: String, default: RgbaF32, flags: PrefFlags) -> Arc<Self> {
		Arc::new(Self {
			id,
			kind: PrefKind::Color,
			data: PrefData {
				color: ManuallyDrop::new(ColorStore {
					value: AtomicCell::new(default),
					default,
				}),
			},
			flags,
		})
	}

	/// Both the current value and default are set to `default`. Reading the FS
	/// for the real current value (if any) is left to the caller.
	#[must_use]
	pub(super) fn _new_string(id: String, default: String, flags: PrefFlags) -> Arc<Self> {
		Arc::new(Self {
			id,
			kind: PrefKind::String,
			data: PrefData {
				string: ManuallyDrop::new(StringStore {
					value: RwLock::new(default.clone()),
					default,
				}),
			},
			flags,
		})
	}

	#[must_use]
	pub(super) fn _to_toml(&self) -> toml::Value {
		unsafe {
			match self.kind {
				PrefKind::Bool => toml::Value::Boolean(self.data.boolean.value.load()),
				PrefKind::Int => toml::Value::Integer(self.data.int.value.load()),
				PrefKind::Float => toml::Value::Float(self.data.float.value.load()),
				PrefKind::Color => {
					let value = self.data.color.value.load();
					let mut table = toml::value::Table::with_capacity(3);
					table.insert("r".to_string(), value.red.into());
					table.insert("g".to_string(), value.green.into());
					table.insert("b".to_string(), value.blue.into());
					toml::Value::Table(table)
				}
				PrefKind::String => toml::Value::String(self.data.string.value.read().clone()),
			}
		}
	}
}

impl Serialize for Pref {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		debug_assert!(self.is_saved());

		unsafe {
			match self.kind {
				PrefKind::Bool => serializer.serialize_bool(self.data.boolean.value.load()),
				PrefKind::Int => serializer.serialize_i64(self.data.int.value.load()),
				PrefKind::Float => serializer.serialize_f64(self.data.float.value.load()),
				PrefKind::Color => {
					let value = self.data.color.value.load();
					let mut color = serializer.serialize_struct("color", 3)?;
					color.serialize_field("r", &value.red)?;
					color.serialize_field("g", &value.green)?;
					color.serialize_field("b", &value.blue)?;
					color.end()
				}
				PrefKind::String => serializer.serialize_str(self.data.string.value.read().deref()),
			}
		}
	}
}

bitflags::bitflags! {
	/// Special treatment rules for preferences.
	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	pub struct PrefFlags: u8 {
		/// Only scripts from the mount that declared this pref may read it.
		const PRIVATE_READ = 1 << 0;
		/// Only scripts from the mount that declared this pref may mutate it.
		const PRIVATE_WRITE = 1 << 1;
		/// Combines `PRIVATE_READ` and `PRIVATE_WRITE`.
		const PRIVATE = Self::PRIVATE_READ.bits() | Self::PRIVATE_WRITE.bits();
		/// If unset, this pref only applies client-side.
		const SIM = 1 << 2;
		/// If unset, this pref is never written to a file.
		const SAVED = 1 << 3;
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefKind {
	Bool,
	Int,
	Float,
	Color,
	String,
}

impl std::fmt::Debug for Pref {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Pref")
			.field("kind", &self.kind)
			.field("data", unsafe {
				match self.kind {
					PrefKind::Bool => &self.data.boolean,
					PrefKind::Int => &self.data.int,
					PrefKind::Float => &self.data.float,
					PrefKind::Color => &self.data.color,
					PrefKind::String => &self.data.string,
				}
			})
			.field("flags", &self.flags)
			.finish()
	}
}

impl Drop for Pref {
	fn drop(&mut self) {
		// `AtomicCell` implements `Drop`, but it and the primitive within
		// are both trivially destructible, so don't bother.
		if let PrefKind::String = self.kind {
			unsafe {
				ManuallyDrop::drop(&mut self.data.string);
			}
		}
	}
}

/// Thin wrapper around an [`Arc`] leveraging generic typing to provide fast access.
#[derive(Debug)]
pub struct Handle<P: PrefValue>(Arc<Pref>, PhantomData<P>);

impl<P: PrefValue> Handle<P> {
	#[must_use]
	pub(super) fn new(p: Arc<Pref>) -> Self {
		Self(p, PhantomData)
	}

	/// See [`Pref::eq_val`].
	#[must_use]
	pub fn eq_val(&self, other: &Self) -> bool {
		self.0.eq_val(&other.0)
	}

	/// See [`Pref::eq_default`].
	#[must_use]
	pub fn eq_default(&self) -> bool {
		self.0.eq_default()
	}
}

// SAFETY: Handles can access raw union fields because their creation
// demands that a type check takes place first.

impl Handle<bool> {
	#[must_use]
	pub fn get(&self) -> bool {
		debug_assert!(self.0.kind == PrefKind::Bool);
		unsafe { self.0.data.boolean.value.load() }
	}

	pub fn set(&self, value: bool) {
		debug_assert!(self.0.kind == PrefKind::Bool);
		unsafe {
			self.0.data.boolean.value.store(value);
		}
	}
}

impl Handle<i64> {
	#[must_use]
	pub fn get(&self) -> i64 {
		debug_assert!(self.0.kind == PrefKind::Int);
		unsafe { self.0.data.int.value.load() }
	}

	pub fn set(&self, value: i64) {
		debug_assert!(self.0.kind == PrefKind::Int);
		unsafe {
			self.0.data.int.value.store(value);
		}
	}
}

impl Handle<f64> {
	#[must_use]
	pub fn get(&self) -> f64 {
		debug_assert!(self.0.kind == PrefKind::Float);
		unsafe { self.0.data.float.value.load() }
	}

	pub fn set(&self, value: f64) {
		debug_assert!(self.0.kind == PrefKind::Float);
		unsafe {
			self.0.data.float.value.store(value);
		}
	}
}

impl Handle<RgbaF32> {
	#[must_use]
	pub fn get(&self) -> RgbaF32 {
		debug_assert!(self.0.kind == PrefKind::Color);
		unsafe { self.0.data.color.value.load() }
	}

	pub fn set(&self, value: RgbaF32) {
		debug_assert!(self.0.kind == PrefKind::Color);
		unsafe {
			self.0.data.color.value.store(value);
		}
	}
}

impl<'p> Handle<String> {
	pub fn get(&'p self) -> RwLockReadGuard<'p, String> {
		debug_assert!(self.0.kind == PrefKind::String);
		unsafe { self.0.data.string.value.read() }
	}

	pub fn get_mut(&'p self) -> RwLockWriteGuard<'p, String> {
		debug_assert!(self.0.kind == PrefKind::String);
		unsafe { self.0.data.string.value.write() }
	}
}

impl<P: PrefValue> PartialEq for Handle<P> {
	/// Checks if these are two pointers to the same preference object.
	/// To check if they have the same storage type and value, use [`eq_val`].
	///
	/// [`eq_val`]: Self::eq_val
	fn eq(&self, other: &Self) -> bool {
		Arc::ptr_eq(&self.0, &other.0)
	}
}

impl<P: PrefValue> Eq for Handle<P> {}

pub trait PrefValue: private::Sealed {
	const KIND: PrefKind;
}

impl PrefValue for bool {
	const KIND: PrefKind = PrefKind::Bool;
}

impl PrefValue for i64 {
	const KIND: PrefKind = PrefKind::Int;
}

impl PrefValue for f64 {
	const KIND: PrefKind = PrefKind::Float;
}

impl PrefValue for RgbaF32 {
	const KIND: PrefKind = PrefKind::Color;
}

impl PrefValue for String {
	const KIND: PrefKind = PrefKind::String;
}

// Internal ////////////////////////////////////////////////////////////////////

union PrefData {
	boolean: ManuallyDrop<BoolStore>,
	int: ManuallyDrop<I64Store>,
	float: ManuallyDrop<F64Store>,
	color: ManuallyDrop<ColorStore>,
	string: ManuallyDrop<StringStore>,
}

#[derive(Debug)]
struct BoolStore {
	value: AtomicCell<bool>,
	default: bool,
}

#[derive(Debug)]
struct I64Store {
	value: AtomicCell<i64>,
	default: i64,
}

#[derive(Debug)]
struct F64Store {
	value: AtomicCell<f64>,
	default: f64,
}

#[derive(Debug)]
struct ColorStore {
	value: AtomicCell<RgbaF32>,
	default: RgbaF32,
}

#[derive(Debug)]
struct StringStore {
	value: RwLock<String>,
	default: String,
}

mod private {
	use crate::RgbaF32;

	pub trait Sealed {}

	impl Sealed for bool {}
	impl Sealed for i64 {}
	impl Sealed for f64 {}
	impl Sealed for RgbaF32 {}
	impl Sealed for String {}
}
