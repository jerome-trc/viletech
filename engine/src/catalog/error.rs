//! Things that can go wrong during data management operations.

use data::level;
use image::ImageError;
use vfs::VPathBuf;

/// Things that can go wrong during (non-preparation) datum management operations,
/// like lookup and mutation. Also see [`PrepError`].
#[derive(Debug)]
pub enum DatumError {
	/// A data object ID didn't resolve to anything.
	NotFound(String),
}

impl std::error::Error for DatumError {}

impl std::fmt::Display for DatumError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::NotFound(id) => {
				write!(f, "no data object exists by the ID: {id}")
			}
		}
	}
}

#[derive(Debug)]
pub struct PrepError {
	pub path: VPathBuf,
	pub kind: PrepErrorKind,
}

impl PrepError {
	#[must_use]
	pub fn is_fatal(&self) -> bool {
		// TODO: Lith parse errors are fatal.
		matches!(
			self.kind,
			PrepErrorKind::Io(_) | PrepErrorKind::MissingLithRoot
		)
	}
}

/// Game loading is a two-step process; data preparation is the second step.
/// This covers the errors that can possibly happen during these operations.
#[derive(Debug)]
pub enum PrepErrorKind {
	/// Failed to read a [COLORMAP] WAD lump.
	///
	/// [COLORMAP]: https://doomwiki.org/wiki/COLORMAP
	ColorMap(data::Error),
	/// Failed to read an [ENDOOM] WAD lump.
	///
	/// [ENDOOM]: https://doomwiki.org/wiki/ENDOOM
	EnDoom(data::Error),
	/// A file between the `F_START` and `F_END` markers was not 4096 bytes in size.
	///
	/// See <https://doomwiki.org/wiki/WAD#Flats.2C_Sprites.2C_and_Patches>.
	Flat,
	Level(level::Error),
	/// Tried to decode a non-picture format image and failed.
	Image(ImageError),
	Io(std::io::Error),
	/// A mount declared a script root file that was not found in the VFS.
	MissingLithRoot,
	/// Failed to read a [PNAMES] WAD lump.
	///
	/// [PNAMES]: https://doomwiki.org/wiki/PNAMES
	PNames(data::Error),
	/// A file between the `S_START` and `S_END` markers is not in picture format,
	/// or any other recognized image format.
	///
	/// See <https://doomwiki.org/wiki/WAD#Flats.2C_Sprites.2C_and_Patches>.
	Sprite,
	/// Failed to read a [TEXTURE1 or TEXTURE2] WAD lump.
	///
	/// [TEXTURE1 or TEXTURE2]: https://doomwiki.org/wiki/TEXTURE1_and_TEXTURE2
	TextureX(data::Error),
	/// A [virtual file](vfs::File) was expected to have some byte or string content,
	/// but was instead empty or a directory.
	Unreadable(VPathBuf),
	/// Failure to decode a FLAC, MP3, Ogg, or WAV file.
	WaveformAudio(kira::sound::FromFileError),
}

impl std::error::Error for PrepError {}

impl std::fmt::Display for PrepError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.kind {
			PrepErrorKind::ColorMap(err) => {
				write!(f, "failed to read `{p}`: {err}", p = self.path)
			}
			PrepErrorKind::EnDoom(err) => {
				write!(f, "failed to read `{p}`: {err}", p = self.path)
			}
			PrepErrorKind::Flat => {
				write!(
					f,
					"lump {} is between `F_START` and `F_END` but is not a flat",
					self.path
				)
			}
			PrepErrorKind::Io(err) => err.fmt(f),
			PrepErrorKind::Level(err) => {
				write!(f, "level `{}` is invalid. Reason: {err}", self.path)
			}
			PrepErrorKind::Image(err) => {
				write!(
					f,
					"failed to decode image: `{p}` - details: {err}",
					p = self.path
				)
			}
			PrepErrorKind::MissingLithRoot => {
				write!(f, "Lithica root directory not found at path: {}", self.path)
			}
			PrepErrorKind::PNames(err) => {
				write!(f, "failed to read `{p}`: {err}", p = self.path)
			}
			PrepErrorKind::Sprite => {
				write!(
					f,
					"lump {} is between `S_START` and `S_END` \
					but is not a recognized sprite format",
					self.path
				)
			}
			PrepErrorKind::TextureX(err) => {
				write!(
					f,
					"TEXTURE1/TEXTURE2 lump `{p}` is malformed: {err}",
					p = self.path
				)
			}
			PrepErrorKind::Unreadable(path) => {
				write!(
					f,
					"virtual file {path} was expected to have bytes or text content, \
					but it is empty or a directory",
				)
			}
			PrepErrorKind::WaveformAudio(err) => write!(
				f,
				"failed to load audio file: `{p}` - details: {err}",
				p = self.path
			),
		}
	}
}
