//! Trait for the LithScript ABI and its two kinds of "words".

use std::{
	any::TypeId,
	hash::{Hash, Hasher},
	ops::Range,
};

/// The principal unit of LithScript's ABI.
/// Only exposed to enable type conversions.
///
/// This structure is used to hold anything that can pass between the langauge
/// boundary; if it can't fit into 64 bits, it gets boxed or referenced.
///
/// For the rationale as to the decision to only use one underlying integer size,
/// see the reasons given for daScript to use only 128-bit variables in its
/// [reference manual](https://dascript.org/doc/dascript.pdf), section 3.1.
#[derive(Clone, Copy)]
pub union QWord {
	pub(super) boolean: bool,
	pub(super) character: char,
	pub(super) i_8: i8,
	pub(super) u_8: u8,
	pub(super) i_16: i16,
	pub(super) u_16: u16,
	pub(super) i_32: i32,
	pub(super) u_32: u32,
	pub(super) i_64: i64,
	pub(super) u_64: u64,
	pub(super) i_size: isize,
	pub(super) u_size: usize,
	pub(super) f_32: f32,
	pub(super) f_64: f64,
}

impl Default for QWord {
	/// Returns a zeroed quad-word.
	fn default() -> Self {
		unsafe { std::mem::zeroed() }
	}
}

impl std::fmt::Debug for QWord {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		unsafe { f.debug_tuple("QWord").field(&self.i_64).finish() }
	}
}

impl PartialEq for QWord {
	fn eq(&self, other: &Self) -> bool {
		unsafe { self.i_64 == other.i_64 }
	}
}

macro_rules! num_converters {
	($($int_t:ty, $field:ident);+) => {
		$(
			impl From<QWord> for $int_t {
				fn from(value: QWord) -> Self {
					unsafe { value.$field }
				}
			}

			impl From<$int_t> for QWord {
				fn from(value: $int_t) -> Self {
					Self { $field: value }
				}
			}
		)+
	}
}

num_converters! {
	bool, boolean;
	char, character;
	i8, i_8;
	u8, u_8;
	i16, i_16;
	u16, u_16;
	i32, i_32;
	u32, u_32;
	i64, i_64;
	u64, u_64;
	isize, i_size;
	usize, u_size;
	f32, f_32;
	f64, f_64
}

impl<T> From<QWord> for *const T {
	#[inline(always)]
	fn from(value: QWord) -> Self {
		unsafe { value.u_size as *const T }
	}
}

impl<T> From<*const T> for QWord {
	#[inline(always)]
	fn from(value: *const T) -> Self {
		Self {
			u_size: value as usize,
		}
	}
}

impl<T> From<QWord> for *mut T {
	#[inline(always)]
	fn from(value: QWord) -> Self {
		unsafe { value.u_size as *mut T }
	}
}

impl<T> From<*mut T> for QWord {
	#[inline(always)]
	fn from(value: *mut T) -> Self {
		Self {
			u_size: value as usize,
		}
	}
}

pub trait Abi: 'static {
	/// This will always be a tuple of [`QWord`]s.
	type Repr: 'static;

	#[must_use]
	fn to_words(self) -> Self::Repr;

	#[must_use]
	fn from_words(words: Self::Repr) -> Self;

	fn type_id_hash<H: Hasher>(state: &mut H);
}

impl Abi for () {
	type Repr = ();

	fn to_words(self) -> Self::Repr {}

	fn from_words(_: Self::Repr) -> Self {}

	fn type_id_hash<H: Hasher>(state: &mut H) {
		TypeId::of::<()>().hash(state)
	}
}

impl<T> Abi for T
where
	T: 'static + Into<QWord> + From<QWord>,
{
	type Repr = QWord;

	fn to_words(self) -> Self::Repr {
		self.into()
	}

	fn from_words(words: Self::Repr) -> Self {
		Self::from(words)
	}

	fn type_id_hash<H: Hasher>(state: &mut H) {
		TypeId::of::<T>().hash(state)
	}
}

impl<T> Abi for Range<T>
where
	T: 'static + Into<QWord> + From<QWord>,
{
	type Repr = (QWord, QWord);

	fn to_words(self) -> Self::Repr {
		(self.start.into(), self.end.into())
	}

	fn from_words(words: Self::Repr) -> Self {
		Self {
			start: words.0.into(),
			end: words.1.into(),
		}
	}

	fn type_id_hash<H: Hasher>(state: &mut H) {
		TypeId::of::<T>().hash(state)
	}
}

// RAT: This might be the biggest bottleneck on compile speed in the workspace

#[impl_trait_for_tuples::impl_for_tuples(1, 2)]
impl Abi for Tuple {
	for_tuples!(type Repr = (#(Tuple::Repr),*););

	fn to_words(self) -> Self::Repr {
		for_tuples!((#(Tuple::to_words(self.Tuple)),*))
	}

	fn from_words(words: Self::Repr) -> Self {
		for_tuples!((#(Tuple::from_words(words.Tuple)),*))
	}

	fn type_id_hash<H: Hasher>(state: &mut H) {
		let _ = for_tuples!((#(Tuple::type_id_hash(state)),*));
	}
}
