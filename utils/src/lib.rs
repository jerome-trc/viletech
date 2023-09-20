//! # VileTech Utilities
//!
//! An assortment of small helper symbols used by multiple other VileTech crates.

#![doc(
	html_favicon_url = "https://media.githubusercontent.com/media/jerome-trc/viletech/master/assets/viletech/viletech.png",
	html_logo_url = "https://media.githubusercontent.com/media/jerome-trc/viletech/master/assets/viletech/viletech.png"
)]

#[cfg(feature = "archery")]
pub mod arck;
pub mod io;
#[macro_use]
pub mod macros;
pub mod math;
pub mod path;
pub mod rstring;
pub mod simd;
pub mod string;

pub type SmallString = smartstring::SmartString<smartstring::LazyCompact>;

/// See <https://zdoom.org/wiki/Editor_number>. Used when populating levels.
pub type EditorNum = u16;
/// See <https://zdoom.org/wiki/Spawn_number>. Used by ACS.
pub type SpawnNum = u16;

/// Note that minutes and seconds are both remainders, not totals.
#[must_use]
pub fn duration_to_hhmmss(duration: std::time::Duration) -> (u64, u64, u64) {
	let mins = duration.as_secs() / 60;
	let hours = mins / 60;
	(hours, mins % 60, duration.as_secs() % 60)
}

/// For representing all the possible endings
/// for operations that can be cancelled by the end user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub enum Outcome<T, E> {
	Cancelled,
	None,
	Err(E),
	Ok(T),
}

impl<T, E> Outcome<T, E> {
	pub fn map<U, F: FnOnce(T) -> U>(self, op: F) -> Outcome<U, E> {
		match self {
			Outcome::Ok(t) => Outcome::Ok(op(t)),
			Outcome::Err(e) => Outcome::Err(e),
			Outcome::None => Outcome::None,
			Outcome::Cancelled => todo!(),
		}
	}
}

impl<T, E> From<Result<T, E>> for Outcome<T, E> {
	fn from(value: Result<T, E>) -> Self {
		match value {
			Ok(t) => Outcome::Ok(t),
			Err(e) => Outcome::Err(e),
		}
	}
}

/// For sending cancellation and progress updates between threads.
/// Wrap in a [`std::sync::Arc`] and use to check how far along a load operation is.
///
/// For example, this is how game loading displays progress bars.
///
/// Uses atomics; all operations run on [`std::sync::atomic::Ordering::Relaxed`].
#[derive(Debug, Default)]
pub struct SendTracker {
	cancelled: std::sync::atomic::AtomicBool,
	progress: std::sync::atomic::AtomicUsize,
	target: std::sync::atomic::AtomicUsize,
}

impl SendTracker {
	#[must_use]
	pub fn new(target: usize) -> Self {
		Self {
			target: std::sync::atomic::AtomicUsize::new(target),
			..Default::default()
		}
	}

	#[must_use]
	pub fn progress(&self) -> usize {
		self.progress.load(std::sync::atomic::Ordering::Relaxed)
	}

	#[must_use]
	pub fn target(&self) -> usize {
		self.target.load(std::sync::atomic::Ordering::Relaxed)
	}

	/// 0.0 means just started; 1.0 means done.
	#[must_use]
	pub fn progress_percent(&self) -> f64 {
		let prog = self.progress.load(std::sync::atomic::Ordering::Relaxed);
		let tgt = self.target.load(std::sync::atomic::Ordering::Relaxed);

		if tgt == 0 {
			return 0.0;
		}

		prog as f64 / tgt as f64
	}

	#[must_use]
	pub fn is_done(&self) -> bool {
		self.progress.load(std::sync::atomic::Ordering::Relaxed)
			>= self.target.load(std::sync::atomic::Ordering::Relaxed)
	}

	pub fn add_to_target(&self, amount: usize) {
		self.target
			.fetch_add(amount, std::sync::atomic::Ordering::Relaxed);
	}

	pub fn set_target(&self, amount: usize) {
		self.target
			.store(amount, std::sync::atomic::Ordering::Relaxed);
	}

	pub fn add_to_progress(&self, amount: usize) {
		self.progress
			.fetch_add(amount, std::sync::atomic::Ordering::Relaxed);
	}

	/// Sets the progress counter to be equal to the target counter.
	///
	/// Mind that [`Self::is_done`] will go back to returning `false` if the target
	/// is incremented after this.
	pub fn finish(&self) {
		self.progress.store(
			self.target.load(std::sync::atomic::Ordering::Relaxed),
			std::sync::atomic::Ordering::Relaxed,
		);
	}

	pub fn cancel(&self) {
		self.cancelled
			.store(true, std::sync::atomic::Ordering::Relaxed);
	}

	#[must_use]
	pub fn is_cancelled(&self) -> bool {
		self.cancelled.load(std::sync::atomic::Ordering::Relaxed)
	}
}

/// WAD entries have a name length limit of 8 ASCII characters, and this limitation
/// has persisted through Doom's descendant source ports (for whatever reason).
/// For compatibility purposes, VileTech sometimes needs to pretend that there's
/// no game data namespacing and look up the last loaded thing with a certain name.
pub type Id8 = arrayvec::ArrayString<{ std::mem::size_of::<char>() * 8 }>;

/// Returns `None` if `id8` starts with a NUL.
/// Return values have no trailing NUL bytes.
#[must_use]
pub fn read_id8(id8: [u8; 8]) -> Option<Id8> {
	if id8.starts_with(&[b'\0']) {
		return None;
	}

	let mut ret = Id8::new();

	for byte in id8 {
		if byte == b'\0' {
			break;
		}

		ret.push(char::from(byte));
	}

	Some(ret)
}

/// Takes however much of `string` can fit into an `Id8` and returns that.
#[must_use]
pub fn id8_truncated(string: &str) -> Id8 {
	let mut ret = Id8::new();
	let end = ret.capacity().min(string.len());
	ret.push_str(&string[0..end]);
	ret
}

/// 4 ASCII characters rolled into one `u32`.
/// Byte ordering is **target-endianness dependent**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ascii4(u32);

impl Ascii4 {
	#[must_use]
	pub const fn from_bytes(a: u8, b: u8, c: u8, d: u8) -> Self {
		let a = a as u32;
		let b = b as u32;
		let c = c as u32;
		let d = d as u32;

		#[cfg(target_endian = "little")]
		{
			Self(a | (b << 8) | (c << 16) | (d << 24))
		}
		#[cfg(target_endian = "big")]
		{
			Self(d | (c << 8) | (b << 16) | (a << 24))
		}
	}

	#[must_use]
	pub const fn from_bstr(bstr: &'static [u8; 4]) -> Self {
		let a = bstr[0] as u32;
		let b = bstr[1] as u32;
		let c = bstr[2] as u32;
		let d = bstr[3] as u32;

		#[cfg(target_endian = "little")]
		{
			Self(a | (b << 8) | (c << 16) | (d << 24))
		}
		#[cfg(target_endian = "big")]
		{
			Self(d | (c << 8) | (b << 16) | (a << 24))
		}
	}
}

impl From<u32> for Ascii4 {
	fn from(value: u32) -> Self {
		Self(value)
	}
}
