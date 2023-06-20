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
		matches!(
			self.kind,
			PrepErrorKind::Io(_) | PrepErrorKind::MissingLithDir | PrepErrorKind::LithParse(_)
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
	MissingLithDir,
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
	LithParse(lith::parse::Error),
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
					"`COLORMAP` lump is the wrong size: `{p}` - expected 8704, but found: {size}",
					p = self.path.display()
				)
			}
			PrepErrorKind::EnDoom(size) => {
				write!(
					f,
					"`ENDOOM` lump is the wrong size: `{p}` - expected 4000, but found: {size}",
					p = self.path.display()
				)
			}
			PrepErrorKind::Flat => {
				write!(
					f,
					"lump {} is between `F_START` and `F_END` but is not a flat",
					self.path.display()
				)
			}
			PrepErrorKind::Io(err) => err.fmt(f),
			PrepErrorKind::Level(err) => {
				write!(
					f,
					"level `{}` is invalid. Reason: {err}",
					self.path.display()
				)
			}
			PrepErrorKind::Image(err) => {
				write!(
					f,
					"failed to decode image: `{p}` - details: {err}",
					p = self.path.display()
				)
			}
			PrepErrorKind::MissingLithDir => {
				write!(
					f,
					"LithScript root directory not found at path: {}",
					self.path.display()
				)
			}
			PrepErrorKind::PNames => {
				write!(f, "malformed PNAMES lump: {}", self.path.display())
			}
			PrepErrorKind::Sprite => {
				write!(
					f,
					"lump {} is between `S_START` and `S_END` \
					but is not a recognized sprite format",
					self.path.display()
				)
			}
			PrepErrorKind::TextureX => {
				write!(
					f,
					"malformed TEXTURE1 or TEXTURE2 lump: {}",
					self.path.display()
				)
			}
			PrepErrorKind::Unreadable(path) => {
				write!(
					f,
					"virtual file {p} was expected to have bytes or text content, \
					but it is empty or a directory",
					p = path.display()
				)
			}
			PrepErrorKind::LithParse(_) => unimplemented!(),
			PrepErrorKind::WaveformAudio(err) => write!(
				f,
				"failed to load audio file: `{p}` - details: {err}",
				p = self.path.display()
			),
		}
	}
}
