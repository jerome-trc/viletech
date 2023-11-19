//! [`VPathBuf`] and [`VPath`].

use std::hash::{Hash, Hasher};

use util::Id8;

/// Components are separated by `/` like in Unixes.
#[derive(Debug, Clone)]
pub struct VPathBuf(String);

impl VPathBuf {
	#[must_use]
	pub fn new(string: String) -> Self {
		Self(string)
	}
}

impl std::borrow::Borrow<VPath> for VPathBuf {
	fn borrow(&self) -> &VPath {
		VPath::new(self.0.as_str())
	}
}

impl std::ops::Deref for VPathBuf {
	type Target = VPath;

	fn deref(&self) -> &Self::Target {
		VPath::new(self.0.as_str())
	}
}

impl PartialEq for VPathBuf {
	fn eq(&self, other: &Self) -> bool {
		self.as_str() == other.as_str()
	}
}

impl PartialEq<VPath> for VPathBuf {
	fn eq(&self, other: &VPath) -> bool {
		self.as_str() == other.as_str()
	}
}

impl<'p> PartialEq<&'p VPath> for VPathBuf {
	fn eq(&self, other: &&'p VPath) -> bool {
		self.as_str() == other.as_str()
	}
}

impl Eq for VPathBuf {}

impl Hash for VPathBuf {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.as_str().hash(state);
	}
}

impl<S: AsRef<str>> From<S> for VPathBuf {
	fn from(value: S) -> Self {
		Self(value.as_ref().to_owned())
	}
}

impl std::fmt::Display for VPathBuf {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

impl<'s> FromIterator<&'s str> for VPathBuf {
	fn from_iter<T: IntoIterator<Item = &'s str>>(iter: T) -> Self {
		let mut buf = String::new();

		for comp in iter {
			buf.push('/');
			buf.push_str(comp);
		}

		Self(buf)
	}
}

impl<'s> FromIterator<&'s VPath> for VPathBuf {
	fn from_iter<T: IntoIterator<Item = &'s VPath>>(iter: T) -> Self {
		let mut buf = String::new();

		for comp in iter {
			buf.push('/');
			buf.push_str(comp.as_str());
		}

		Self(buf)
	}
}
/// [`VPathBuf`]'s counterpart to [`std::path::Path`].
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VPath(str);

impl VPath {
	#[must_use]
	pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &Self {
		// SAFETY: Same code as `std::path::Path::new`.
		unsafe { &*(s.as_ref() as *const str as *const Self) }
	}

	#[must_use]
	pub fn byte_len(&self) -> usize {
		self.0.len()
	}

	/// Whether this path starts with a `/` character.
	#[must_use]
	pub fn is_absolute(&self) -> bool {
		self.0.starts_with('/')
	}

	pub fn components(&self) -> impl Iterator<Item = &Self> {
		self.as_str()
			.split('/')
			.filter(|c| !c.is_empty())
			.map(Self::new)
	}

	/// The same functionality as [`std::path::Path::file_name`].
	#[must_use]
	pub fn file_name(&self) -> Option<&Self> {
		self.0.rsplit('/').next().map(Self::new)
	}

	/// The same functionality as [`std::path::Path::file_prefix`].
	#[must_use]
	pub fn file_prefix(&self) -> Option<&Self> {
		self.file_name()
			.and_then(|fname| fname.as_str().split('.').next().map(Self::new))
	}

	/// The same functionality as [`std::path::Path::file_stem`].
	#[must_use]
	pub fn file_stem(&self) -> Option<&Self> {
		let Some(name) = self.file_name() else {
			return None;
		};

		let Some((stem, _ext)) = name.as_str().rsplit_once('.') else {
			return Some(name);
		};

		Some(Self::new(stem))
	}

	/// The same functionality as [`std::path::Path::extension`].
	#[must_use]
	pub fn extension(&self) -> Option<&str> {
		let mut rsplit = self.0.rsplit('.');
		let ret = rsplit.next();

		match rsplit.next() {
			Some(_) => ret,
			None => None,
		}
	}

	/// The same functionality as [`std::path::Path::parent`].
	#[must_use]
	pub fn parent(&self) -> Option<&Self> {
		self.0.rsplitn(2, '/').last().map(Self::new)
	}

	/// Returns the ["stem"](std::path::Path::file_stem) of this path,
	/// truncated to the first 8 characters, converted to ASCII uppercase.
	#[must_use]
	pub fn lump_name(&self) -> Option<Id8> {
		let Some(stem) = self.file_stem() else {
			return None;
		};

		let mut ret = Id8::new();

		for c in stem.as_str().chars().take(8) {
			ret.push(c.to_ascii_uppercase());
		}

		Some(ret)
	}

	#[must_use]
	pub fn as_str(&self) -> &str {
		&self.0
	}

	#[must_use]
	pub fn eq_ignore_ascii_case(&self, s: impl AsRef<str>) -> bool {
		self.as_str().eq_ignore_ascii_case(s.as_ref())
	}
}

impl PartialEq<VPathBuf> for VPath {
	fn eq(&self, other: &VPathBuf) -> bool {
		self.as_str() == other.as_str()
	}
}

impl<'p> PartialEq<VPathBuf> for &'p VPath {
	fn eq(&self, other: &VPathBuf) -> bool {
		self.as_str() == other.as_str()
	}
}

impl ToOwned for VPath {
	type Owned = VPathBuf;

	fn to_owned(&self) -> Self::Owned {
		VPathBuf::new(self.0.to_owned())
	}
}

impl std::fmt::Display for VPath {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}
