//! Sound- and music-related code.

mod gui;
mod kmidi;

use std::{
	io::{Cursor, Read, Seek},
	ops::{Deref, DerefMut},
	path::{Path, PathBuf},
	sync::Arc,
};

use bevy::prelude::{debug, error, info, warn};
use bevy_egui::egui;
use crossbeam::channel::SendError;
use kira::{
	manager::{
		backend::{cpal::CpalBackend, Backend},
		error::PlaySoundError,
		AudioManager, AudioManagerSettings,
	},
	sound::{
		static_sound::{PlaybackState, StaticSoundData, StaticSoundHandle, StaticSoundSettings},
		SoundData,
	},
	tween::Tween,
	CommandError,
};
use nodi::midly;
use once_cell::sync::Lazy;
use parking_lot::{Mutex, RwLock};

use crate::{data::Catalog, sim::actor::Actor, util, vfs::FileRef};

pub use kmidi::{
	render as render_midi, Data as MidiData, Handle as MidiHandle, Settings as MidiSettings,
};

use self::gui::DevGui;

pub struct AudioCore {
	/// The centre of waveform sound synthesis and playback.
	pub manager: AudioManager,
	pub soundfonts: Vec<SoundFont>,
	/// Unlike `Self::sounds`, this behaves like a stack.
	/// Only the last element gets played at any given time.
	pub music: Vec<Sound>,
	/// Sounds currently being played.
	pub sounds: Vec<WorldSound>,
	catalog: Arc<RwLock<Catalog>>,
	gui: DevGui,
}

// SAFETY: This is to deal with `AudioManager`, which uses sufficient internal
// synchronization to be safe but is missing a trait bound on a `PhantomData`.
unsafe impl Sync for AudioCore {}

impl AudioCore {
	/// If `None` is given, the defaults will be used.
	pub fn new(
		catalog: Arc<RwLock<Catalog>>,
		manager_settings: Option<AudioManagerSettings<CpalBackend>>,
	) -> Result<Self, Error> {
		let manager_settings = manager_settings.unwrap_or_default();
		let sound_cap = manager_settings.capacities.sound_capacity;

		fluidlite::Log::set(
			fluidlite::LogLevel::DEBUG,
			fluidlite::FnLogger::new(|level, msg| match level {
				fluidlite::LogLevel::Panic => error!(target: "fluidlite", "(FATAL): {msg}"),
				fluidlite::LogLevel::Error => error!(target: "fluidlite", "msg"),
				fluidlite::LogLevel::Warning => warn!(target: "fluidlite", "{msg}"),
				fluidlite::LogLevel::Info => info!(target: "fluidlite", "{msg}"),
				fluidlite::LogLevel::Debug => debug!(target: "fluidlite", "{msg}"),
			}),
		);

		let mut ret = Self {
			catalog,
			manager: AudioManager::new(manager_settings).map_err(Error::KiraBackend)?,
			soundfonts: Self::collect_soundfonts()?,
			music: vec![],
			sounds: Vec::with_capacity(sound_cap),
			gui: DevGui::default(),
		};

		if !ret.soundfonts.is_empty() {
			let cow = ret.soundfonts[0].path.to_string_lossy();
			ret.gui.soundfont_buf = cow.to_string();
		}

		Ok(ret)
	}

	/// Clears sound and music handles that have stopped.
	/// Returns an error if the removal of a music track causes another song
	/// lower in the stack to start, and it fails to do so.
	pub fn update(&mut self) -> Result<(), Error> {
		self.sounds
			.retain(|snd| snd.state() != PlaybackState::Stopped);

		self.music
			.retain(|mus| mus.state() != PlaybackState::Stopped);

		// If the previously-currently-playing music track at the top of the
		// stack was stopped and removed, whichever track was below it was paused,
		// and must now be resumed.
		if let Some(mus) = self.music.last_mut() {
			if !mus.is_playing() {
				return mus.resume(Tween::default());
			}
		}

		Ok(())
	}

	/// This assumes that `data` has already been completely configured.
	///
	/// Returns an error if:
	/// - An already-playing song fails to pause.
	/// - The given song fails to start playback.
	///
	/// `fade_out` only gets used if the new music is getting pushed over an
	/// already-playing track, which gets paused. If `None` is given, the pause
	/// happens practically instantaneously.
	pub fn start_music_wave(
		&mut self,
		data: StaticSoundData,
		fade_out: Option<Tween>,
	) -> Result<(), Error> {
		if let Some(mus) = self.music.last_mut() {
			mus.pause(fade_out.unwrap_or_default())?;
		}

		let handle = self.manager.play(data).map_err(Error::PlayWave)?;

		self.music.push(Sound::Wave(handle));

		Ok(())
	}

	/// This assumes that `data` has already been completely configured.
	///
	/// Returns an error if:
	/// - An already-playing song fails to pause.
	/// - The given song fails to start playback.
	///
	/// `fade_out` only gets used if the new music is getting pushed over an
	/// already-playing track, which gets paused. If `None` is given, the pause
	/// happens practically instantaneously.
	pub fn start_music_midi(
		&mut self,
		data: MidiData,
		fade_out: Option<Tween>,
	) -> Result<(), Error> {
		if let Some(mus) = self.music.last_mut() {
			mus.pause(fade_out.unwrap_or_default())?;
		}

		let handle = self.manager.play(data).map_err(Error::PlayMidi)?;

		self.music.push(Sound::Midi(handle));

		Ok(())
	}

	/// Stops the music slot at the top of the stack.
	/// If the music stack is empty, this is a valid no-op.
	pub fn stop_music(&mut self, fade_out: Option<Tween>) -> Result<(), Error> {
		let Some(mut mus) = self.music.pop() else { return Ok(()); };
		mus.stop(fade_out.unwrap_or_default())
	}

	/// If no `source` is given, the sound will always audible to all clients
	/// and not be subjected to any panning or attenuation.
	pub fn start_sound_wave(
		&mut self,
		data: StaticSoundData,
		source: Option<Actor>,
	) -> Result<(), Error> {
		self.sounds.push(WorldSound {
			sound: Sound::Wave(self.manager.play(data).map_err(Error::PlayWave)?),
			_source: source,
		});

		Ok(())
	}

	/// If no `source` is given, the sound will always audible to all clients
	/// and not be subjected to any panning or attenuation.
	pub fn start_sound_midi(&mut self, data: MidiData, source: Option<Actor>) -> Result<(), Error> {
		self.sounds.push(WorldSound {
			sound: Sound::Midi(self.manager.play(data).map_err(Error::PlayMidi)?),
			_source: source,
		});

		Ok(())
	}

	/// Instantly pauses every sound and music handle.
	pub fn pause_all(&mut self) -> Result<(), Error> {
		self.manager.pause(Tween::default()).map_err(Error::from)
	}

	pub fn pause_all_sounds(&mut self) {
		for (i, sound) in self.sounds.iter_mut().enumerate() {
			if let Err(err) = sound.pause(Tween::default()) {
				error!("Failed to pause sound {i}: {err}");
			}
		}
	}

	/// Instantly resumes every sound and music handle.
	pub fn resume_all(&mut self) -> Result<(), Error> {
		self.manager.resume(Tween::default()).map_err(Error::from)
	}

	pub fn resume_all_sounds(&mut self) {
		for (i, sound) in self.sounds.iter_mut().enumerate() {
			if let Err(err) = sound.resume(Tween::default()) {
				error!("Failed to resume sound {i}: {err}");
			}
		}
	}

	/// Instantly stops every ongoing sound and music track. The sound array
	/// gets cleared along with both music slots.
	pub fn stop_all(&mut self) {
		self.stop_all_sounds();
		self.stop_all_music();
	}

	/// An error gets logged if stopping a sound fails.
	pub fn stop_all_sounds(&mut self) {
		for (i, mut sound) in self.sounds.drain(..).enumerate() {
			if let Err(err) = sound.stop(Tween::default()) {
				error!("`stop_all_sounds` failed to stop sound {i}: {err}");
			}
		}
	}

	/// The entire music stack gets stopped instantaneously and cleared.
	/// An error gets logged if stopping a slot fails.
	pub fn stop_all_music(&mut self) {
		for (i, mut mus) in self.music.drain(..).enumerate() {
			if let Err(err) = mus.stop(Tween::default()) {
				error!("`stop_all_music` failed to stop song {i}: {err}");
			}
		}
	}

	/// A fundamental part of engine initialization. Recursively read the contents of
	/// `<executable_directory>/soundfonts`, determine their types, and store their
	/// paths. Note that in the debug build, `<working_directory>/data/soundfonts`
	/// will be walked instead.
	///
	/// If no SoundFont files whatsoever could be found, `Ok(())` still gets
	/// returned, but a log warning gets emitted.
	fn collect_soundfonts() -> Result<Vec<SoundFont>, Error> {
		let mut ret = vec![];

		let walker = walkdir::WalkDir::new::<&Path>(SOUNDFONT_DIR.as_ref())
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
				SOUNDFONT_DIR.display(),
			);
		}

		Ok(ret)
	}

	/// Draw the egui-based developer/debug/diagnosis menu, and perform any
	/// state mutations requested through it by the user.
	pub fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
		self.ui_impl(ctx, ui);
	}
}

impl std::fmt::Debug for AudioCore {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("AudioCore")
			.field("soundfonts", &self.soundfonts)
			.field("music", &self.music)
			.field("sounds", &self.sounds)
			.field("catalog", &self.catalog)
			.field("gui", &self.gui)
			.finish()
	}
}

/// A type alias for convenience and to reduce line noise.
pub type AudioCoreAM = Arc<Mutex<AudioCore>>;
/// A type alias for convenience and to reduce line noise.
pub type AudioCoreAL = Arc<RwLock<AudioCore>>;

/// Enables inspection and control of a currently-playing sound or musical track,
/// whether it's waveform or MIDI.
pub enum Sound {
	Wave(StaticSoundHandle),
	Midi(MidiHandle),
}

impl Sound {
	#[must_use]
	pub fn state(&self) -> PlaybackState {
		match self {
			Self::Wave(wave) => wave.state(),
			Self::Midi(midi) => midi.state(),
		}
	}

	pub fn pause(&mut self, tween: Tween) -> Result<(), Error> {
		match self {
			Sound::Wave(wave) => wave.pause(tween).map_err(Error::from),
			Sound::Midi(midi) => midi.pause(tween),
		}
	}

	pub fn resume(&mut self, tween: Tween) -> Result<(), Error> {
		match self {
			Sound::Wave(wave) => wave.resume(tween).map_err(Error::from),
			Sound::Midi(midi) => midi.resume(tween),
		}
	}

	pub fn stop(&mut self, tween: Tween) -> Result<(), Error> {
		match self {
			Sound::Wave(wave) => wave.stop(tween).map_err(Error::from),
			Sound::Midi(midi) => midi.stop(tween),
		}
	}

	#[must_use]
	pub fn is_playing(&self) -> bool {
		match self {
			Sound::Wave(wave) => wave.state() == PlaybackState::Playing,
			Sound::Midi(midi) => midi.is_playing(),
		}
	}
}

impl std::fmt::Debug for Sound {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Wave(_) => f.debug_tuple("Wave").finish(),
			Self::Midi(_) => f.debug_tuple("Midi").finish(),
		}
	}
}

/// [`Sound`] with a source actor attached.
#[derive(Debug)]
pub struct WorldSound {
	sound: Sound,
	_source: Option<Actor>,
}

impl Deref for WorldSound {
	type Target = Sound;

	fn deref(&self) -> &Self::Target {
		&self.sound
	}
}

impl DerefMut for WorldSound {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.sound
	}
}

pub fn sound_from_file(
	file: FileRef,
	settings: StaticSoundSettings,
) -> Result<StaticSoundData, Box<dyn std::error::Error>> {
	let bytes = file.try_read_bytes()?.to_owned();
	let cursor = Cursor::new(bytes);

	match StaticSoundData::from_cursor(cursor, settings) {
		Ok(ssd) => Ok(ssd),
		Err(err) => Err(Box::new(err)),
	}
}

pub fn sound_from_bytes(
	bytes: impl Into<Vec<u8>>,
	settings: StaticSoundSettings,
) -> Result<StaticSoundData, kira::sound::FromFileError> {
	let cursor = Cursor::new(bytes.into());
	StaticSoundData::from_cursor(cursor, settings)
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

static SOUNDFONT_DIR: Lazy<PathBuf> = Lazy::new(|| {
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
});

#[must_use]
pub fn soundfont_dir() -> &'static Path {
	&SOUNDFONT_DIR
}

#[derive(Debug)]
pub enum Error {
	SoundFontRead(PathBuf, fluidlite::Error),
	KiraBackend(<CpalBackend as Backend>::Error),
	ParseMidi(midly::Error),
	MidiSynth(fluidlite::Error),
	PlayMidi(PlayMidiError),
	PlayWave(PlayWaveError),
	CommandOverflow,
	ThreadPanic,
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Self::SoundFontRead(_, err) => Some(err),
			Self::KiraBackend(err) => Some(err),
			Self::ParseMidi(err) => Some(err),
			Self::MidiSynth(err) => Some(err),
			Self::PlayMidi(err) => Some(err),
			Self::PlayWave(err) => Some(err),
			Self::CommandOverflow | Self::ThreadPanic => None,
		}
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::SoundFontRead(path, err) => write!(
				f,
				"failed to read SoundFont at path: `{p}` - details: {err}",
				p = path.display()
			),
			Self::KiraBackend(err) => err.fmt(f),
			Self::ParseMidi(err) => err.fmt(f),
			Self::MidiSynth(err) => err.fmt(f),
			Self::PlayMidi(err) => err.fmt(f),
			Self::PlayWave(err) => err.fmt(f),
			Self::CommandOverflow => {
				write!(f, "tried to send too many commands to a sound at once")
			}
			Self::ThreadPanic => write!(f, "audio thread has panicked"),
		}
	}
}

impl From<CommandError> for Error {
	fn from(value: CommandError) -> Self {
		match value {
			CommandError::CommandQueueFull => Self::CommandOverflow,
			CommandError::MutexPoisoned => Self::ThreadPanic,
			_ => unreachable!(),
		}
	}
}

impl From<SendError<kmidi::Command>> for Error {
	fn from(_: SendError<kmidi::Command>) -> Self {
		Self::ThreadPanic
	}
}

pub type PlayWaveError = PlaySoundError<<StaticSoundData as SoundData>::Error>;
pub type PlayMidiError = PlaySoundError<<MidiData as SoundData>::Error>;
