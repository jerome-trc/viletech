//! Various virtual path types serving different needs using structural sharing.

use std::{
	borrow::Borrow,
	hash::{Hash, Hasher},
};

use util::rstring::RString;

/// Always uses `/` as a separator, and always in lowercase.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VPathBuf(pub(crate) RString);

impl Borrow<VPath> for VPathBuf {
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

/// Shares content with a [`VPath`], but "pretends" to lack a file name.
/// As an example, `/my_mod/music/subfolder/song.mid` becomes `/my_mod/music/subfolder`.
#[derive(Debug, Clone)]
pub struct FolderPath(pub(crate) RString);

impl Borrow<VPath> for FolderPath {
	fn borrow(&self) -> &VPath {
		VPath::new(self.0.as_str()).parent().unwrap()
	}
}

impl PartialEq for FolderPath {
	fn eq(&self, other: &Self) -> bool {
		Borrow::<VPath>::borrow(self) == Borrow::<VPath>::borrow(other)
	}
}

impl Eq for FolderPath {}

impl PartialOrd for FolderPath {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Borrow::<VPath>::borrow(self).partial_cmp(Borrow::<VPath>::borrow(other))
	}
}

impl Ord for FolderPath {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		Borrow::<VPath>::borrow(self).cmp(Borrow::<VPath>::borrow(other))
	}
}

impl Hash for FolderPath {
	fn hash<H: Hasher>(&self, state: &mut H) {
		Borrow::<VPath>::borrow(self).hash(state);
	}
}

impl From<VPathBuf> for FolderPath {
	fn from(value: VPathBuf) -> Self {
		Self(value.0)
	}
}

impl std::ops::Deref for FolderPath {
	type Target = VPath;

	fn deref(&self) -> &Self::Target {
		VPath::new(self.0.as_str())
	}
}

/// Shares content with a [`VPath`], but "pretends" to lack the first two components.
/// As an example, `/my_mod/music/subfolder/song.mid` becomes `subfolder/song.mid`.
#[derive(Debug, Clone)]
pub struct SpacedPath(pub(crate) RString);

impl Borrow<VPath> for SpacedPath {
	fn borrow(&self) -> &VPath {
		VPath::new(self.0.as_str().splitn(4, '/').last().unwrap())
	}
}

impl PartialEq for SpacedPath {
	fn eq(&self, other: &Self) -> bool {
		Borrow::<VPath>::borrow(self) == Borrow::<VPath>::borrow(other)
	}
}

impl Eq for SpacedPath {}

impl PartialOrd for SpacedPath {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Borrow::<VPath>::borrow(self).partial_cmp(Borrow::<VPath>::borrow(other))
	}
}

impl Ord for SpacedPath {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		Borrow::<VPath>::borrow(self).cmp(Borrow::<VPath>::borrow(other))
	}
}

impl Hash for SpacedPath {
	fn hash<H: Hasher>(&self, state: &mut H) {
		Borrow::<VPath>::borrow(self).hash(state);
	}
}

impl From<VPathBuf> for SpacedPath {
	fn from(value: VPathBuf) -> Self {
		Self(value.0)
	}
}

impl std::ops::Deref for SpacedPath {
	type Target = VPath;

	fn deref(&self) -> &Self::Target {
		VPath::new(self.0.as_str())
	}
}

/// Shares content with a [`VPath`], but "pretends" to be only a file name.
/// As an example, `/my_mod/music/subfolder/song.mid` becomes `song.mid`.
#[derive(Debug, Clone)]
pub struct ShortPath(pub(crate) RString);

impl Borrow<VPath> for ShortPath {
	fn borrow(&self) -> &VPath {
		VPath::new(self.0.as_str()).name().unwrap()
	}
}

impl PartialEq for ShortPath {
	fn eq(&self, other: &Self) -> bool {
		Borrow::<VPath>::borrow(self) == Borrow::<VPath>::borrow(other)
	}
}

impl Eq for ShortPath {}

impl PartialOrd for ShortPath {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Borrow::<VPath>::borrow(self).partial_cmp(Borrow::<VPath>::borrow(other))
	}
}

impl Ord for ShortPath {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		Borrow::<VPath>::borrow(self).cmp(Borrow::<VPath>::borrow(other))
	}
}

impl Hash for ShortPath {
	fn hash<H: Hasher>(&self, state: &mut H) {
		Borrow::<VPath>::borrow(self).hash(state);
	}
}

impl From<VPathBuf> for ShortPath {
	fn from(value: VPathBuf) -> Self {
		Self(value.0)
	}
}

impl std::ops::Deref for ShortPath {
	type Target = VPath;

	fn deref(&self) -> &Self::Target {
		VPath::new(self.0.as_str())
	}
}

/// [`VPath`]'s counterpart to [`std::path::Path`].
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VPath(str);

impl VPath {
	#[must_use]
	pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &Self {
		// SAFETY: Same code as `std::path::Path::new`.
		unsafe { &*(s.as_ref() as *const str as *const Self) }
	}

	pub fn components(&self) -> impl Iterator<Item = &Self> {
		let text = if self.is_absolute() {
			&self.0[1..]
		} else {
			&self.0
		};

		text.split('/').map(Self::new)
	}

	#[must_use]
	pub fn name(&self) -> Option<&Self> {
		self.0.rsplit('/').next().map(Self::new)
	}

	#[must_use]
	pub fn is_absolute(&self) -> bool {
		self.0.starts_with('/')
	}

	#[must_use]
	pub fn parent(&self) -> Option<&Self> {
		self.0.rsplitn(2, '/').last().map(Self::new)
	}
}

impl PartialEq<str> for VPath {
	fn eq(&self, other: &str) -> bool {
		&self.0 == other
	}
}
