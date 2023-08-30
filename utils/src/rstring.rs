//! A thin atomically reference-counted string type.

use std::{borrow::Borrow, ops::Deref};

/// Thin atomically reference-counted string using [`triomphe::ThinArc`].
///
/// Essentially a [`std::sync::Arc`], but occupies only one pointer-width and
/// has no support for weak pointers (since strings cannot have circular references),
/// so it makes non-trivial space efficiency gains.
#[derive(Debug)]
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
		/// The type of `0` does not provide [`ExactSizeIterator`], but we
		/// do in fact know the size, so just provide an impl ourselves.
		struct Esi<I: Iterator<Item = u8>>(I, usize);

		impl<I: Iterator<Item = u8>> Iterator for Esi<I> {
			type Item = u8;

			fn next(&mut self) -> Option<Self::Item> {
				self.0.next()
			}

			fn size_hint(&self) -> (usize, Option<usize>) {
				(self.1, Some(self.1))
			}
		}

		impl<I: Iterator<Item = u8>> ExactSizeIterator for Esi<I> {
			fn len(&self) -> usize {
				self.1
			}
		}

		// Yes, it really does take this kind of "language lawyering" to iterate
		// over every byte in a slice of string slices.
		let iter = strings.iter().flat_map(|s| s.as_bytes()).copied();
		let total_len = strings.iter().fold(0, |acc, s| acc + s.len());

		Self(triomphe::ThinArc::from_header_and_iter(
			(),
			Esi(iter, total_len),
		))
	}

	#[must_use]
	pub fn as_ptr(&self) -> *const str {
		unsafe {
			let s = std::str::from_utf8_unchecked(&self.0.slice);
			s as *const str
		}
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
		let s0 = Deref::deref(self);
		let s1 = Deref::deref(other);
		s0.partial_cmp(s1)
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

#[test]
#[cfg(test)]
fn soundness() {
	use std::collections::HashSet;

	let rstring = RString::new("hello world");

	assert_eq!(rstring, "hello world");
	assert!(rstring.eq_ignore_ascii_case("HELLO WORLD"));

	unsafe {
		let ptr = rstring.as_ptr();
		let sref = &*ptr;
		assert_eq!(sref, "hello world");
	}

	assert_eq!(rstring.len(), 11);

	let rstring = RString::from_strs(&["/vzs/collect", "::TArray::element"]);
	assert_eq!(rstring.len(), 30);
	assert_eq!(rstring, "/vzs/collect::TArray::element");
	assert!(rstring.eq_ignore_ascii_case("/VZS/COLLECT::TARRAY::ELEMENT"));

	let mut set = HashSet::new();
	set.insert(rstring.clone());
	assert!(set.contains(&rstring));
	assert!(set.contains("/vzs/collect::TArray::element"));
}
