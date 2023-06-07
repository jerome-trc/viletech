//! Things that can go wrong during data management operations.

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
				write!(f, "No data object exists by the ID: {id}")
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
		matches!(
			self.kind,
			PrepErrorKind::Io(_) | PrepErrorKind::MissingVzsDir | PrepErrorKind::VzsParse(_)
		)
	}
}

/// Game loading is a two-step process; data preparation is the second step.
/// This covers the errors that can possibly happen during these operations.
#[derive(Debug)]
pub enum PrepErrorKind {
	/// A [COLORMAP] WAD lump is the wrong size.
	///
	/// [COLORMAP]: https://doomwiki.org/wiki/COLORMAP
	ColorMap(usize),
	/// An [ENDOOM] WAD lump is the wrong size.
	///
	/// [ENDOOM]: https://doomwiki.org/wiki/ENDOOM
	EnDoom(usize),
	/// A file between the `F_START` and `F_END` markers was not 4096 bytes in size.
	///
	/// See <https://doomwiki.org/wiki/WAD#Flats.2C_Sprites.2C_and_Patches>.
	Flat,
	Level(level::Error),
	/// Tried to decode a non-picture format image and failed.
	Image(ImageError),
	Io(std::io::Error),
	/// A mount declared a script root file that was not found in the VFS.
	MissingVzsDir,
	/// A [PNAMES] WAD lump is too short or an incorrect size.
	///
	/// [PNAMES]: https://doomwiki.org/wiki/PNAMES
	PNames,
	/// A file between the `S_START` and `S_END` markers is not in picture format,
	/// or any other recognized image format.
	///
	/// See <https://doomwiki.org/wiki/WAD#Flats.2C_Sprites.2C_and_Patches>.
	Sprite,
	/// A [TEXTURE1 or TEXTURE2] WAD lump is too short or an incorrect size.
	///
	/// [TEXTURE1 or TEXTURE2]: https://doomwiki.org/wiki/TEXTURE1_and_TEXTURE2
	TextureX,
	/// A [virtual file](vfs::File) was expected to have some byte or string content,
	/// but was instead empty or a directory.
	Unreadable(VPathBuf),
	VzsParse(vzs::parse::Error<'static>),
	/// Failure to decode a FLAC, MP3, Ogg, or WAV file.
	WaveformAudio(kira::sound::FromFileError),
}

impl std::error::Error for PrepError {}

impl std::fmt::Display for PrepError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.kind {
			PrepErrorKind::ColorMap(size) => {
				write!(
					f,
					"`COLORMAP` lump is the wrong size: {p}\r\n\t\
					Expected 8704, but found: {size}",
					p = self.path.display()
				)
			}
			PrepErrorKind::EnDoom(size) => {
				write!(
					f,
					"`ENDOOM` lump is the wrong size: {p}\r\n\t\
					Expected 4000, but found: {size}",
					p = self.path.display()
				)
			}
			PrepErrorKind::Flat => {
				write!(
					f,
					"Lump {} is between `F_START` and `F_END` but is not a flat.",
					self.path.display()
				)
			}
			PrepErrorKind::Io(err) => err.fmt(f),
			PrepErrorKind::Level(err) => {
				write!(f, "Map `{}` is invalid. Reason: {err}", self.path.display())
			}
			PrepErrorKind::Image(err) => {
				write!(
					f,
					"Failed to decode image: {p}\r\n\tDetails: {err}",
					p = self.path.display()
				)
			}
			PrepErrorKind::MissingVzsDir => {
				write!(
					f,
					"VZScript root directory not found at path: {}",
					self.path.display()
				)
			}
			PrepErrorKind::PNames => {
				write!(f, "Malformed PNAMES lump: {}", self.path.display())
			}
			PrepErrorKind::Sprite => {
				write!(
					f,
					"Lump {} is between `S_START` and `S_END` \
					but is not a recognized sprite format.",
					self.path.display()
				)
			}
			PrepErrorKind::TextureX => {
				write!(
					f,
					"Malformed TEXTURE1 or TEXTURE2 lump: {}",
					self.path.display()
				)
			}
			PrepErrorKind::Unreadable(path) => {
				write!(
					f,
					"Virtual file {p} was expected to have bytes or text content, \
					but it is empty or a directory.",
					p = path.display()
				)
			}
			PrepErrorKind::VzsParse(_) => todo!(),
			PrepErrorKind::WaveformAudio(err) => write!(
				f,
				"Failed to load audio file: {p}\r\n\t\
				Details: {err}",
				p = self.path.display()
			),
		}
	}
}
