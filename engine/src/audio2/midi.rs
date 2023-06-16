//! Interface between [`kira`] and [`nodi`]/[`fluidlite`].
//!
//! Much of this code is a copy-paste of `kira`'s internal sound sampling code.

use std::{path::{Path, PathBuf}, io::{Read, Seek}};

use tracing::{warn, info};

use super::{AudioCore, Error};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
	Midi,
	Hmi,
	Xmi,
	DmxMus,
	Mids,
}

impl Format {
	/// From ZMusic.
	#[must_use]
	pub fn deduce(bytes: &[u8]) -> Option<Format> {
		use util::Ascii4;

		if bytes.len() < 12 {
			return None;
		}

		if bytes[0] == b'M' && bytes[1] == b'U' && bytes[2] == b'S' && bytes[3] == 0x1A {
			return Some(Self::DmxMus);
		}

		let m0 = Ascii4::from_bytes(bytes[0], bytes[1], bytes[2], bytes[3]);
		let m1 = Ascii4::from_bytes(bytes[4], bytes[5], bytes[6], bytes[7]);
		let m2 = Ascii4::from_bytes(bytes[8], bytes[9], bytes[10], bytes[11]);

		if m0 == Ascii4::from_bstr(b"HMI-")
			&& m1 == Ascii4::from_bstr(b"MIDI")
			&& m2 == Ascii4::from_bstr(b"SONG")
		{
			return Some(Self::Hmi);
		}

		if m0 == Ascii4::from_bstr(b"HMIM") && m1 == Ascii4::from_bstr(b"IDIP") {
			return Some(Self::Hmi);
		}

		if m0 == Ascii4::from_bstr(b"FORM") && m2 == Ascii4::from_bstr(b"XDIR") {
			return Some(Self::Xmi);
		}

		if (m0 == Ascii4::from_bstr(b"CAT ") || m0 == Ascii4::from_bstr(b"FORM"))
			&& m2 == Ascii4::from_bstr(b"XMID")
		{
			return Some(Self::Xmi);
		}

		if m0 == Ascii4::from_bstr(b"RIFF") && m2 == Ascii4::from_bstr(b"MIDS") {
			return Some(Self::Mids);
		}

		if m0 == Ascii4::from_bstr(b"MThd") {
			return Some(Self::Midi);
		}

		None
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoundFont {
	/// The canonicalized path to this SoundFont's file.
	/// Needed by the FluidSynth backend.
	path: PathBuf,
	kind: SoundFontKind,
}

impl SoundFont {
	#[must_use]
	pub fn new(path: PathBuf, kind: SoundFontKind) -> Self {
		Self { path, kind }
	}

	/// The name of the SoundFont file, without the extension (i.e. the file stem).
	#[must_use]
	pub fn name(&self) -> &Path {
		Path::new(self.path.file_stem().unwrap_or_default())
	}

	/// The name of the SoundFont file, along with the extension.
	#[must_use]
	pub fn name_ext(&self) -> &Path {
		Path::new(self.path.file_name().unwrap_or_default())
	}

	/// The canonicalized path to this SoundFont's file.
	#[must_use]
	pub fn full_path(&self) -> &Path {
		&self.path
	}

	#[must_use]
	pub fn kind(&self) -> SoundFontKind {
		self.kind
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundFontKind {
	Sf2,
	Gus,
	Wopl,
	Wopn,
}

impl AudioCore {
	/// A fundamental part of engine initialization. Recursively read the contents of
	/// `<executable_directory>/soundfonts`, determine their types, and store their
	/// paths. Note that in the debug build, `<working_directory>/data/soundfonts`
	/// will be walked instead.
	///
	/// If no SoundFont files whatsoever could be found, `Ok(())` still gets
	/// returned, but a log warning gets emitted.
	pub(super) fn collect_soundfonts() -> Result<Vec<SoundFont>, Error> {
		let sfdir = Self::soundfont_dir();
		let mut ret = vec![];

		let walker = walkdir::WalkDir::new::<&Path>(sfdir.as_ref())
			.follow_links(false)
			.max_depth(8)
			.same_file_system(true)
			.sort_by_file_name()
			.into_iter()
			.filter_map(|res| res.ok());

		for dir_entry in walker {
			let path = dir_entry.path();

			let metadata = match dir_entry.metadata() {
				Ok(m) => m,
				Err(err) => {
					warn!(
						"Failed to retrieve metadata for file: {}\r\n\tError: {err}",
						path.display(),
					);
					continue;
				}
			};

			if metadata.is_dir() || metadata.is_symlink() || metadata.len() == 0 {
				continue;
			}

			// Check if another SoundFont by this name has already been collected.
			if ret
				.iter()
				.any(|sf: &SoundFont| sf.name().as_os_str().eq_ignore_ascii_case(path.as_os_str()))
			{
				continue;
			}

			let mut file = match std::fs::File::open(path) {
				Ok(f) => f,
				Err(err) => {
					warn!("Failed to open file: {}\r\nError: {}", path.display(), err);
					continue;
				}
			};

			let mut header = [0_u8; 16];

			match file.read_exact(&mut header) {
				Ok(()) => {}
				Err(err) => {
					warn!("Failed to read file: {}\r\nError: {}", path.display(), err);
				}
			};

			let sf_kind = if &header[0..4] == b"RIFF" && &header[8..16] == b"sfbkLIST" {
				SoundFontKind::Sf2
			} else if &header[..11] == b"WOPL3-BANK\0" {
				SoundFontKind::Wopl
			} else if &header[..11] == b"WOPN2-BANK\0" {
				SoundFontKind::Wopn
			} else if util::io::is_zip(&header) {
				SoundFontKind::Gus
			} else {
				info!(
					"Failed to determine SoundFont type of file: {}\r\nSkipping it.",
					path.display()
				);
				continue;
			};

			if sf_kind == SoundFontKind::Gus {
				match file.rewind() {
					Ok(()) => {}
					Err(err) => {
						warn!(
							"Failed to rewind file stream for zip read: {}\r\nError: {}",
							path.display(),
							err
						);
						continue;
					}
				};

				let mut archive = match zip::ZipArchive::new(&mut file) {
					Ok(zf) => zf,
					Err(err) => {
						warn!("Failed to unzip file: {}\r\nError: {}", path.display(), err);
						continue;
					}
				};

				// (GZ)
				// A SoundFont archive with only one file can't be a packed GUS patch.
				// Just skip this entirely.
				if archive.len() <= 1 {
					continue;
				}

				let timidity = match archive.by_name("timidity.cfg") {
					Ok(timid) => timid,
					Err(err) => {
						warn!(
							"Failed to find `timidity.cfg` file in: {}\r\nError: {}",
							path.display(),
							err
						);
						continue;
					}
				};

				if !timidity.is_file() || timidity.size() < 1 {
					warn!(
						"Found `timidity.cfg` in a zip SoundFont but it's malformed. ({})",
						path.display()
					);
					continue;
				}

				// This GUS SoundFont has been validated. Now it can be pushed.
			}

			ret.push(SoundFont {
				path: path.to_owned(),
				kind: sf_kind,
			});
		}

		if ret.is_empty() {
			warn!(
				"No SoundFont files were found under path: {}\r\n\t\
				The engine will be unable to render MIDI sound.",
				sfdir.display(),
			);
		}

		Ok(ret)
	}

	#[must_use]
	fn soundfont_dir() -> PathBuf {
		#[cfg(not(debug_assertions))]
		{
			let ret = util::path::exe_dir().join("soundfonts");

			if !ret.exists() {
				let res = std::fs::create_dir(&ret);

				if let Err(err) = res {
					panic!(
						"failed to create directory: {}\r\n\tError: {}",
						ret.display(),
						err
					)
				}
			}

			ret
		}

		#[cfg(debug_assertions)]
		{
			[
				std::env::current_dir().expect("failed to get working directory"),
				PathBuf::from("data/soundfonts"),
			]
			.iter()
			.collect()
		}
	}
}
