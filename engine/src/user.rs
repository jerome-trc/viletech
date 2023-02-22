//! User information (preferences, storage) structures and functions.
//!
//! The engine will recognize one "user info" directory on the end user's machine.
//! This may be a [home directory](home::home_dir) or `<executable directory>/user`,
//! if the user would prefer their entire installation be portable.

mod dirs;
mod error;
mod pref;
mod profile;

use std::{
	fs::ReadDir,
	path::{Path, PathBuf},
};

use log::warn;
use serde::{Deserialize, Serialize};

pub use dirs::*;
pub use error::*;
pub use pref::*;
pub use profile::*;

/// Hub for preferences, persistent storage (p-storage),
/// saved games, demos, and screenshot management.
#[derive(Debug)]
pub struct UserCore {
	user_dir: PathBuf,
	global_cfg: GlobalConfig,
	profile: Profile,
	prefs: PrefPreset,
}

// Public interface.
impl UserCore {
	/// Creates an empty core for when the user has no information on the FS and
	/// needs to decide where it should be stored. Once they have chosen, call
	/// [`new`](Self::uninit).
	///
	/// Almost all of the core's methods will panic if trying to use it while
	/// in this state.
	#[must_use]
	pub fn uninit() -> Self {
		Self {
			global_cfg: GlobalConfig::default(),
			user_dir: PathBuf::default(),
			profile: Profile::default(),
			prefs: PrefPreset::default(),
		}
	}

	/// If `user_dir` does not exist or is not actually a directory, this will panic.
	pub fn new(user_dir: PathBuf) -> Result<Self, Error> {
		assert!(user_dir.exists() && user_dir.is_dir());

		let mut ret = Self {
			global_cfg: GlobalConfig::default(),
			user_dir,
			profile: Profile::default(),
			prefs: PrefPreset::default(),
		};

		if let Some(gcfg) = ret.read_global_cfg()? {
			if !gcfg.last_profile.is_empty() {
				let path = ret.profile_path(&gcfg.last_profile);
				let p = ret.find_profile(&path)?;

				ret.profile = match p {
					Some(p) => p,
					None => ret.find_any_profile()?,
				};
			} else {
				ret.profile = ret.find_any_profile()?;
			}

			if !gcfg.last_preset.is_empty() {
				let path = ret.pref_preset_path(&gcfg.last_profile);
				let p = ret.find_pref_preset(&path)?;

				ret.prefs = match p {
					Some(p) => p,
					None => ret.find_any_pref_preset()?,
				};
			} else {
				ret.prefs = ret.find_any_pref_preset()?;
			}

			ret.global_cfg = gcfg;
		} else {
			ret.profile = ret.find_any_profile()?;
			ret.prefs = ret.find_any_pref_preset()?;
			let gcfg = ret.init_global_cfg()?;
			ret.global_cfg = gcfg;
		}

		// TODO:
		// - Define prefs from manifests as they get mounted to the VFS.
		// - Fill pref values from .toml files.

		Ok(ret)
	}

	pub fn get_pref<P: PrefValue>(&self, id: &str) -> Result<PrefHandle<P>, Error> {
		self.prefs.get::<P>(id)
	}
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct GlobalConfig {
	last_profile: String,
	last_preset: String,
}

/// Lives directly under the user info directory.
pub(self) const GLOBALCFG_FILENAME: &str = "global.toml";

// Internal implementation details.
impl UserCore {
	/// Returns `Ok(None)` if the global config file does not exist.
	/// Mind that its fields may still be empty even if it does exist.
	fn read_global_cfg(&self) -> Result<Option<GlobalConfig>, Error> {
		let gcfg_path = self.user_dir.join(GLOBALCFG_FILENAME);

		if !gcfg_path.exists() {
			return Ok(None);
		}

		let bytes = std::fs::read(&gcfg_path).map_err(|err| Error::FileRead {
			source: err,
			path: gcfg_path.clone(),
		})?;

		let text = String::from_utf8(bytes).map_err(|err| Error::Utf8 {
			source: err.utf8_error(),
			path: gcfg_path.clone(),
		})?;

		let ret: GlobalConfig = toml::from_str(&text).map_err(|err| Error::TomlParse {
			source: err,
			path: gcfg_path.clone(),
		})?;

		Ok(Some(ret))
	}

	/// Called if the global config file did not exist, and needs to be written.
	fn init_global_cfg(&self) -> Result<GlobalConfig, Error> {
		let ret = GlobalConfig {
			last_profile: self.profile.name().to_string(),
			last_preset: self.prefs.name().to_string(),
		};

		let text = toml::ser::to_string_pretty(&ret).expect("Failed to serialize global config.");

		fwrite(self.user_dir.join(GLOBALCFG_FILENAME), text)?;

		Ok(ret)
	}

	/// Returns:
	/// - `Ok(None)` if the profile's directory does not exist (or the `profiles`
	/// directory itself does not exist, for that matter).
	/// - `Err` if a file system operation fails, or a non-directory exists
	/// under the path that the profile's directory should have existed in.
	/// - `Ok(Some)` otherwise, even if the profile's directory has no contents.
	fn find_profile(&self, path: &Path) -> Result<Option<Profile>, Error> {
		if !path.exists() {
			return Ok(None);
		}

		Self::profile_valid(path)?;

		let name = path.file_name().unwrap().to_string_lossy();

		Ok(Some(Profile::new(name.to_string())))
	}

	/// The global config needed to be (re)-created or did not specify which
	/// profile should be used. Find an existing one, or create a new one.
	/// This may also need to create the whole `profiles` directory along the way.
	fn find_any_profile(&self) -> Result<Profile, Error> {
		const DEFAULT_NAME: &str = "Player";

		let profiles_dir = self.profiles_dir();

		if !profiles_dir.exists() {
			mkdir(&profiles_dir)?;
		}

		let dir_iter = read_dir(&profiles_dir)?.filter_map(|res| match res {
			Ok(dir_entry) => {
				let ftype = match dir_entry.file_type() {
					Ok(ft) => ft,
					Err(err) => {
						warn!(
							"Failed to get file type of possible user profile at path: {p}\
							\r\n\tDetails: {err}",
							p = dir_entry.path().display()
						);

						return None;
					}
				};

				if !ftype.is_dir() {
					return None;
				}

				Some(dir_entry)
			}
			Err(err) => {
				warn!(
					"Failed to read possible user profile under path: {p}\
					\r\n\tDetails: {err}",
					p = profiles_dir.display()
				);
				None
			}
		});

		for dir_entry in dir_iter {
			let de_path = dir_entry.path();

			match Self::profile_valid(&de_path) {
				Ok(()) => {
					let fname = dir_entry.file_name();
					let name = fname.to_string_lossy();
					return Ok(Profile::new(name.to_string()));
				}
				Err(err) => warn!(
					"Malformed user profile at path: {p}\r\n\tDetails: {d}",
					p = de_path.display(),
					d = err,
				),
			}
		}

		self.create_profile(DEFAULT_NAME.to_string())
	}

	fn create_profile(&self, name: String) -> Result<Profile, Error> {
		assert!(
			(2..=64).contains(&name.chars().count()),
			"Tried to create a profile with an illegally sized name ({c}).",
			c = name.chars().count(),
		);

		let path = self.profiles_dir().join(&name);

		if path.exists() {
			return Err(Error::Preexisting {
				item: "User profile",
				path,
			});
		}

		mkdir(self.profile_path(&name))?;
		mkdir(self.demos_dir(&name))?;
		mkdir(self.saves_dir(&name))?;
		mkdir(self.screenshots_dir(&name))?;
		mkdir(self.storage_dir(&name))?;

		Ok(Profile::new(name))
	}

	fn profile_valid(dir: &Path) -> Result<(), Error> {
		if !dir.is_dir() {
			return Err(Error::FileAbnormality {
				path: dir.to_path_buf(),
				expected: "a directory",
				found: "a file",
			});
		}

		Ok(())
	}

	/// Returns:
	/// - `Ok(None)` if the preset's directory does not exist (or the `prefs`
	/// directory itself does not exist, for that matter).
	/// - `Err` if a file system operation fails, or a non-directory exists
	/// under the path that the preset's directory should have existed in.
	/// - `Ok(Some)` otherwise, even if the preset's directory has no contents.
	fn find_pref_preset(&self, path: &Path) -> Result<Option<PrefPreset>, Error> {
		if !path.exists() {
			return Ok(None);
		}

		Self::pref_preset_valid(path)?;

		let name = path.file_name().unwrap().to_string_lossy();

		Ok(Some(PrefPreset::new(name.to_string())))
	}

	/// The global config needed to be (re)-created or did not specify which
	/// pref. preset should be used. Find an existing one, or create a new one.
	/// This may also need to create the whole `prefs` directory along the way.
	fn find_any_pref_preset(&self) -> Result<PrefPreset, Error> {
		const DEFAULT_NAME: &str = "Default";

		let prefs_dir = self.prefs_dir();

		if !prefs_dir.exists() {
			mkdir(&prefs_dir)?;
			return self.create_pref_preset(DEFAULT_NAME.to_string());
		}

		let dir_iter = read_dir(&prefs_dir)?.filter_map(|res| match res {
			Ok(dir_entry) => {
				let ftype = match dir_entry.file_type() {
					Ok(ft) => ft,
					Err(err) => {
						warn!(
							"Failed to get file type of possible pref. preset at path: {p}\
							\r\n\tDetails: {err}",
							p = dir_entry.path().display()
						);

						return None;
					}
				};

				if !ftype.is_dir() {
					return None;
				}

				Some(dir_entry)
			}
			Err(err) => {
				warn!(
					"Failed to read possible pref. preset under path: {p}\
					\r\n\tDetails: {err}",
					p = prefs_dir.display()
				);
				None
			}
		});

		for dir_entry in dir_iter {
			let de_path = dir_entry.path();

			match Self::pref_preset_valid(&de_path) {
				Ok(()) => {
					let fname = dir_entry.file_name();
					let name = fname.to_string_lossy();
					return Ok(PrefPreset::new(name.to_string()));
				}
				Err(err) => warn!(
					"Malformed pref. preset at path: {p}\r\n\tDetails: {d}",
					p = de_path.display(),
					d = err,
				),
			}
		}

		self.create_pref_preset(DEFAULT_NAME.to_string())
	}

	/// This creates the representative object and named directory, but no .toml
	/// files are written or read, and no pref structures are instantiated.
	/// Those responsibilities fall to the caller.
	fn create_pref_preset(&self, name: String) -> Result<PrefPreset, Error> {
		assert!(
			(2..=64).contains(&name.chars().count()),
			"Tried to create a pref. preset with an illegally sized name ({c}).",
			c = name.chars().count(),
		);

		let path = self.pref_preset_path(&name);

		if path.exists() {
			return Err(Error::Preexisting {
				item: "Preference preset",
				path,
			});
		}

		mkdir(path)?;

		Ok(PrefPreset::new(name))
	}

	fn pref_preset_valid(dir: &Path) -> Result<(), Error> {
		if !dir.is_dir() {
			return Err(Error::FileAbnormality {
				path: dir.to_path_buf(),
				expected: "a directory",
				found: "a file",
			});
		}

		Ok(())
	}

	#[must_use]
	fn globalcfg_path(&self) -> PathBuf {
		self.user_dir.join(GLOBALCFG_FILENAME)
	}

	/// Shortcut for `self.user_dir.join("profiles")`.
	#[must_use]
	fn profiles_dir(&self) -> PathBuf {
		self.user_dir.join("profiles")
	}

	#[must_use]
	fn profile_path(&self, name: &str) -> PathBuf {
		[&self.user_dir, Path::new("profiles"), Path::new(name)]
			.iter()
			.collect()
	}

	#[must_use]
	fn demos_dir(&self, profile_name: &str) -> PathBuf {
		debug_assert!(!profile_name.is_empty());

		[
			&self.user_dir,
			Path::new("profiles"),
			Path::new(profile_name),
			Path::new("demos"),
		]
		.iter()
		.collect()
	}

	#[must_use]
	fn saves_dir(&self, profile_name: &str) -> PathBuf {
		debug_assert!(!profile_name.is_empty());

		[
			&self.user_dir,
			Path::new("profiles"),
			Path::new(profile_name),
			Path::new("saves"),
		]
		.iter()
		.collect()
	}

	#[must_use]
	fn screenshots_dir(&self, profile_name: &str) -> PathBuf {
		debug_assert!(!profile_name.is_empty());

		[
			&self.user_dir,
			Path::new("profiles"),
			Path::new(profile_name),
			Path::new("screenshots"),
		]
		.iter()
		.collect()
	}

	#[must_use]
	fn storage_dir(&self, profile_name: &str) -> PathBuf {
		debug_assert!(!profile_name.is_empty());

		[
			&self.user_dir,
			Path::new("profiles"),
			Path::new(profile_name),
			Path::new("persist"),
		]
		.iter()
		.collect()
	}

	/// Shortcut for `self.user_dir.join("prefs")`.
	#[must_use]
	fn prefs_dir(&self) -> PathBuf {
		self.user_dir.join("prefs")
	}

	#[must_use]
	fn pref_preset_path(&self, name: &str) -> PathBuf {
		debug_assert!(!name.is_empty());

		[&self.user_dir, Path::new("prefs"), Path::new(name)]
			.iter()
			.collect()
	}
}

/// An error-mapping helper for brevity.
fn mkdir(path: impl AsRef<Path>) -> Result<(), Error> {
	debug_assert!(!path.as_ref().exists());

	std::fs::create_dir(&path).map_err(|err| Error::CreateDir {
		source: err,
		path: path.as_ref().to_path_buf(),
	})
}

/// An error-mapping helper for brevity.
fn fread(path: impl AsRef<Path>) -> Result<Vec<u8>, Error> {
	debug_assert!(path.as_ref().exists());

	std::fs::read(&path).map_err(|err| Error::FileRead {
		source: err,
		path: path.as_ref().to_path_buf(),
	})
}

/// An error-mapping helper for brevity.
fn fwrite(path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> Result<(), Error> {
	debug_assert!(!path.as_ref().exists());

	std::fs::write(&path, content).map_err(|err| Error::FileWrite {
		source: err,
		path: path.as_ref().to_path_buf(),
	})
}

/// An error-mapping helper for brevity.
fn read_dir(path: impl AsRef<Path>) -> Result<ReadDir, Error> {
	debug_assert!(path.as_ref().exists());

	std::fs::read_dir(&path).map_err(|err| Error::ReadDir {
		source: err,
		path: path.as_ref().to_path_buf(),
	})
}
