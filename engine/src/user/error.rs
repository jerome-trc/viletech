//! Things that can go wrong when managing prefs, profiles, save games, et cetera.

use std::{path::PathBuf, str::Utf8Error};

use super::PrefKind;

/// Things that can go wrong when managing prefs, profiles, save games, et cetera.
#[derive(Debug)]
pub enum Error {
	/// A caller gave an ID that did not resolve to any known [`Pref`](super::Pref).
	PrefNotFound(String),
	/// A caller tried to get a [`PrefHandle`] and the ID resolved correctly,
	/// but the type requested was different to that of the backing [`Pref`].
	///
	/// [`PrefHandle`]: super::PrefHandle
	/// [`Pref`]: super::Pref
	TypeMismatch {
		expected: PrefKind,
		given: PrefKind,
	},
	Utf8 {
		source: Utf8Error,
		path: PathBuf,
	},
	/// See [`std::fs::read`].
	FileRead {
		source: std::io::Error,
		path: PathBuf,
	},
	TomlParse {
		source: toml::de::Error,
		path: PathBuf,
	},
	/// See [`std::fs::create_dir`].
	CreateDir {
		source: std::io::Error,
		path: PathBuf,
	},
	/// See [`std::fs::write`].
	FileWrite {
		source: std::io::Error,
		path: PathBuf,
	},
	/// Expected to find a directory at a path and found a plain file instead, or
	/// vice-versa. This is treated severely; the engine takes care not to delete
	/// parts of the FS that are not expected to be present, so as to not risk
	/// damaging the user's non-VileTech data.
	FileAbnormality {
		path: PathBuf,
		expected: &'static str,
		found: &'static str,
	},
	/// See [`std::fs::read_dir`].
	ReadDir {
		source: std::io::Error,
		path: PathBuf,
	},
	/// An attempt to create a profile or preference preset would have overwritten
	/// another by the same name.
	Preexisting {
		item: &'static str,
		path: PathBuf,
	},
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Self::Utf8 { source, path: _ } => Some(source),
			Self::FileRead { source, path: _ } => Some(source),
			Self::TomlParse { source, path: _ } => Some(source),
			Self::CreateDir { source, path: _ } => Some(source),
			Self::FileWrite { source, path: _ } => Some(source),
			Self::ReadDir { source, path: _ } => Some(source),
			_ => None,
		}
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::PrefNotFound(id) => {
				write!(f, "No pref exists by the ID: {id}")
			}
			Self::TypeMismatch { expected, given } => {
				write!(
					f,
					"Type mismatch during pref lookup. \
					Expected {expected:#?}, got {given:#?}.",
				)
			}
			Self::Utf8 { source, path } => {
				write!(
					f,
					"File is invalid text: {p}\r\n\tDetails: {s}",
					p = path.display(),
					s = source
				)
			}
			Self::FileRead { source, path } => {
				write!(
					f,
					"Failed to read contents of file: {p}\r\n\tDetails: {s}",
					p = path.display(),
					s = source,
				)
			}
			Self::TomlParse { source, path } => {
				write!(
					f,
					"Failed to parse contents of file: {p}\r\n\tDetails: {s}",
					p = path.display(),
					s = source,
				)
			}
			Self::CreateDir { source, path } => {
				write!(
					f,
					"Failed to create a folder: {p}\r\n\tDetails: {s}",
					p = path.display(),
					s = source,
				)
			}
			Self::FileWrite { source, path } => {
				write!(
					f,
					"Failed to write to file: {p}\r\n\tDetails: {s}",
					p = path.display(),
					s = source,
				)
			}
			Self::FileAbnormality {
				expected,
				found,
				path,
			} => {
				write!(
					f,
					"Abnormality at file path: {p}\r\n\t\
					Expected to find: {expected}\r\n\t\
					Found: {found}",
					p = path.display(),
				)
			}
			Self::ReadDir { source, path } => {
				write!(
					f,
					"Failed to read contents of directory: {p}\r\n\tDetails: {s}",
					p = path.display(),
					s = source,
				)
			}
			Self::Preexisting { item, path } => {
				write!(f, "{item} already exists: {}", path.display())
			}
		}
	}
}
