//! A thin atomically reference-counted string type.

/// Thin atomically reference-counted string using [`triomphe::ThingArc`].
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
	pub fn as_ptr(&self) -> *const str {
		unsafe {
			let s = std::str::from_utf8_unchecked(&self.0.slice);
			s as *const str
		}
	}
}

impl std::ops::Deref for RString {
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
	/// Checks if these are two equivalent pointers to the same string.
	fn eq(&self, other: &Self) -> bool {
		std::ptr::eq(self.0.as_ptr(), other.0.as_ptr())
	}
}

impl PartialEq<str> for RString {
	/// Character-by-character string comparison.
	fn eq(&self, other: &str) -> bool {
		std::ops::Deref::deref(self) == other
	}
}

impl Eq for RString {}

impl std::hash::Hash for RString {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.0.as_ptr().hash(state)
	}
}

#[test]
#[cfg(test)]
fn soundness() {
	let rstring = RString::new("hello world");

	assert_eq!(rstring, *"hello world");
	assert!(rstring.eq_ignore_ascii_case("HELLO WORLD"));

	unsafe {
		let ptr = rstring.as_ptr();
		let sref = &*ptr;
		assert_eq!(sref, "hello world");
	}

	assert_eq!(rstring.len(), 11);
}
