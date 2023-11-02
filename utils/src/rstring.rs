//! A thin atomically reference-counted string type.

use std::{borrow::Borrow, ffi::c_void, ops::Deref};

/// Thin atomically reference-counted string using [`triomphe::ThinArc`].
///
/// Essentially a [`std::sync::Arc`], but occupies only one pointer-width and
/// has no support for weak pointers (since strings cannot have circular references),
/// so it makes non-trivial space efficiency gains.
pub struct RString(triomphe::ThinArc<(), u8>);

impl RString {
	#[must_use]
	pub fn new(string: impl AsRef<str>) -> Self {
		Self(triomphe::ThinArc::from_header_and_slice(
			(),
			string.as_ref().as_bytes(),
		))
	}

	#[must_use]
	pub fn from_strs(strings: &[&str]) -> Self {
		let iter = strings.iter().flat_map(|s| s.as_bytes()).copied();
		let total_len = strings.iter().fold(0, |acc, s| acc + s.len());

		Self(triomphe::ThinArc::from_header_and_iter(
			(),
			ByteIter(iter, total_len),
		))
	}

	#[must_use]
	pub fn from_str_iter<'s>(strings: impl Iterator<Item = &'s str> + Clone) -> Self {
		let iter = strings.clone().flat_map(|s| s.as_bytes()).copied();
		let total_len = strings.fold(0, |acc, s| acc + s.len());

		Self(triomphe::ThinArc::from_header_and_iter(
			(),
			ByteIter(iter, total_len),
		))
	}

	#[must_use]
	pub fn as_ptr(&self) -> *const str {
		unsafe {
			let s = std::str::from_utf8_unchecked(&self.0.slice);
			s as *const str
		}
	}

	#[must_use]
	pub fn as_thin_ptr(&self) -> *const c_void {
		self.0.as_ptr()
	}

	#[must_use]
	pub fn as_str(&self) -> &str {
		self.deref()
	}

	/// Checks if these are two equivalent pointers to the same string.
	#[must_use]
	pub fn ptr_eq(&self, other: &Self) -> bool {
		std::ptr::eq(self.0.as_ptr(), other.0.as_ptr())
	}
}

impl Deref for RString {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		// SAFETY: These can only be constructed with `impl AsRef<str>` and are
		// immutable. This byte slice is guaranteed to be valid UTF-8.
		unsafe { std::str::from_utf8_unchecked(&self.0.slice) }
	}
}

impl Clone for RString {
	/// Incurs only the cost of one `Arc` clone.
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl PartialEq for RString {
	/// Character-by-character string comparison.
	fn eq(&self, other: &Self) -> bool {
		self.deref() == other.deref()
	}
}

impl PartialEq<&str> for RString {
	/// Character-by-character string comparison.
	fn eq(&self, other: &&str) -> bool {
		Deref::deref(self) == *other
	}
}

impl Eq for RString {}

impl PartialOrd for RString {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for RString {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		let s0 = Deref::deref(self);
		let s1 = Deref::deref(other);
		s0.cmp(s1)
	}
}

impl std::hash::Hash for RString {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		Borrow::<str>::borrow(self).hash(state)
	}
}

impl Borrow<str> for RString {
	fn borrow(&self) -> &str {
		self.as_ref()
	}
}

impl std::fmt::Display for RString {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		std::fmt::Display::fmt(self.deref(), f)
	}
}

impl std::fmt::Debug for RString {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "\"{}\"", self.as_str())
	}
}

#[test]
#[cfg(test)]
fn soundness() {
	use std::collections::HashSet;

	let rstring = RString::new("atmospheric extinction");

	assert_eq!(rstring, "atmospheric extinction");
	assert!(rstring.eq_ignore_ascii_case("ATMOSPHERIC EXTINCTION"));

	unsafe {
		let ptr = rstring.as_ptr();
		let sref = &*ptr;
		assert_eq!(sref, "atmospheric extinction");
	}

	assert_eq!(rstring.len(), 22);

	let rstring = RString::from_str_iter(
		["devour", ".", "and"]
			.iter()
			.map(|s| *s)
			.chain([".", "saturate"]),
	);
	assert_eq!(rstring, "devour.and.saturate");

	let rstring = RString::from_strs(&["/patience/is", "::a::virtue"]);
	assert_eq!(rstring.len(), 23);
	assert_eq!(rstring, "/patience/is::a::virtue");
	assert!(rstring.eq_ignore_ascii_case("/PATIENCE/IS::A::VIRTUE"));

	let mut set = HashSet::new();
	set.insert(rstring.clone());
	assert!(set.contains(&rstring));
	assert!(set.contains("/patience/is::a::virtue"));
}

// Details /////////////////////////////////////////////////////////////////////

/// The type of `I` does not provide [`ExactSizeIterator`], but we
/// do in fact know the size, so just provide an impl ourselves.
///
/// Yes, it really does take this kind of "language lawyering" to iterate
/// over every byte in a slice of string slices.
struct ByteIter<I: Iterator<Item = u8>>(I, usize);

impl<I: Iterator<Item = u8>> Iterator for ByteIter<I> {
	type Item = u8;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(self.1, Some(self.1))
	}
}

impl<I: Iterator<Item = u8>> ExactSizeIterator for ByteIter<I> {
	fn len(&self) -> usize {
		self.1
	}
}
