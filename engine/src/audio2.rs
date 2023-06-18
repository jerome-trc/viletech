//! Sound- and music-related code.

mod gui;
mod midi;

use std::{io::Cursor, path::PathBuf};

use arrayvec::ArrayVec;
use bevy::prelude::Entity;
use bevy_egui::egui;
use crossbeam::channel::SendError;
use kira0_8_3::{
	manager::{
		backend::{cpal::CpalBackend, Backend},
		error::{AddSpatialSceneError, AddSubTrackError, PlaySoundError},
		AudioManager, AudioManagerSettings,
	},
	sound::{
		static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings},
		PlaybackState, SoundData,
	},
	spatial::{
		emitter::{EmitterHandle, EmitterSettings},
		listener::{ListenerHandle, ListenerSettings},
		scene::{AddListenerError, SpatialSceneHandle, SpatialSceneSettings},
	},
	track::{
		effect::{
			delay::{DelayBuilder, DelayHandle},
			reverb::{ReverbBuilder, ReverbHandle},
		},
		TrackBuilder, TrackHandle, TrackRoutes,
	},
	tween::Tween,
	CommandError, OutputDestination,
};
use nodi::midly;
use rayon::prelude::*;
use tracing::{debug, error, info, warn};

use crate::data::Catalog;

use self::gui::DevGui;

pub use self::midi::{
	render as render_midi, Data as MidiData, FileFormat as MidiFormat, Handle as MidiHandle,
	Settings as MidiSettings, SoundFont, SoundFontKind,
};

pub const MUSIC_LAYERS: usize = 8;

/// The centre for music and sound effect playback for any VileTech application.
pub struct AudioCore {
	manager: AudioManager,
	soundfonts: Vec<SoundFont>,
	scene: SpatialSceneHandle,
	listener: ListenerHandle,
	tracks: SubTracks,

	/// Unlike `sounds`, this behaves like a stack.
	/// Only the last element gets played at any given time.
	music: ArrayVec<MusicController, 3>,
	sounds: Box<[Option<SoundEffect>]>,

	gui: DevGui,
}

/// All played audio gets routed through one of these
/// to enable separation of volume control by the end user.
pub struct SubTracks {
	pub sfx: TrackController,
	pub music: TrackController,
	pub menu: TrackController,
}

impl AudioCore {
	/// If `None` is given, the defaults will be used.
	pub fn new(manager_settings: Option<AudioManagerSettings<CpalBackend>>) -> Result<Self, Error> {
		let m_settings = manager_settings.unwrap_or_default();

		let snd_cap = m_settings.capacities.sound_capacity;

		let s_settings = SpatialSceneSettings::new()
			.listener_capacity(1)
			.emitter_capacity(1024);

		fluidlite::Log::set(
			fluidlite::LogLevel::DEBUG,
			fluidlite::FnLogger::new(|level, msg| match level {
				fluidlite::LogLevel::Panic => error!(target: "fluidlite", "(FATAL) {msg}"),
				fluidlite::LogLevel::Error => error!(target: "fluidlite", "msg"),
				fluidlite::LogLevel::Warning => warn!(target: "fluidlite", "{msg}"),
				fluidlite::LogLevel::Info => info!(target: "fluidlite", "{msg}"),
				fluidlite::LogLevel::Debug => debug!(target: "fluidlite", "{msg}"),
			}),
		);

		let mut manager = AudioManager::new(m_settings).map_err(Error::KiraBackend)?;

		let t_sfx = TrackController::new(&mut manager, None)?;
		let t_music = TrackController::new(&mut manager, None)?;
		let t_menu = TrackController::new(&mut manager, None)?;

		let mut scene = manager
			.add_spatial_scene(s_settings)
			.map_err(Error::SpatialScene)?;

		let listener = scene
			.add_listener(
				glam::Vec3::ZERO,
				glam::Quat::IDENTITY,
				ListenerSettings::new().track(t_sfx.handle.id()),
			)
			.map_err(Error::Listener)?;

		let soundfonts = Self::collect_soundfonts()?;

		let mut ret = Self {
			manager,
			soundfonts,
			scene,
			listener,
			tracks: SubTracks {
				sfx: t_sfx,
				music: t_music,
				menu: t_menu,
			},

			music: ArrayVec::new(),
			sounds: {
				let mut v = vec![];
				v.resize_with(snd_cap, || None);
				v.into_boxed_slice()
			},

			gui: DevGui::default(),
		};

		if !ret.soundfonts.is_empty() {
			let cow = ret.soundfonts[0].path.to_string_lossy();
			ret.gui.soundfont_buf = cow.to_string();
		}

		Ok(ret)
	}

	pub fn start_sfx_wave(
		&mut self,
		mut data: StaticSoundData,
		channel: Option<usize>,
		space: SoundSpace,
	) -> Result<(), Error> {
		let mut entity = None;

		let emitter = match space {
			SoundSpace::World {
				pos,
				entity: e,
				settings,
			} => {
				let e_h =
					self.spatial_setup(&mut data.settings.output_destination, pos, settings)?;
				entity = e;
				Some(e_h)
			}
			SoundSpace::Menu => {
				data.settings.output_destination =
					OutputDestination::Track(self.tracks.menu.handle.id());
				None
			}
			SoundSpace::Unsourced => {
				data.settings.output_destination =
					OutputDestination::Track(self.tracks.sfx.handle.id());
				None
			}
		};

		let sound = self.manager.play(data).map_err(Error::PlayWave)?;

		let sfx = SoundEffect {
			sound: Handle::Wave(sound),
			entity,
			emitter,
		};

		self.start_sfx_on_channel(sfx, channel)
	}

	pub fn start_sfx_midi(
		&mut self,
		mut data: MidiData,
		channel: Option<usize>,
		space: SoundSpace,
	) -> Result<(), Error> {
		let mut entity = None;

		let emitter = match space {
			SoundSpace::World {
				pos,
				entity: e,
				settings,
			} => {
				let e_h = self.spatial_setup(&mut data.settings.destination, pos, settings)?;
				entity = e;
				Some(e_h)
			}
			SoundSpace::Menu => {
				data.settings.destination = OutputDestination::Track(self.tracks.menu.handle.id());
				None
			}
			SoundSpace::Unsourced => {
				data.settings.destination = OutputDestination::Track(self.tracks.sfx.handle.id());
				None
			}
		};

		let sound = self.manager.play(data).map_err(Error::PlayMidi)?;

		let sfx = SoundEffect {
			sound: Handle::Midi(sound),
			entity,
			emitter,
		};

		self.start_sfx_on_channel(sfx, channel)
	}

	/// `dest` is only altered if returning `Ok`.
	fn spatial_setup(
		&mut self,
		dest: &mut OutputDestination,
		pos: glam::Vec3,
		settings: EmitterSettings,
	) -> Result<EmitterHandle, Error> {
		self.scene
			.add_emitter(pos, settings)
			.map_err(|err| Error::AddEmitter(Box::new(err)))
			.map(|e_h| {
				*dest = OutputDestination::Emitter(e_h.id());
				e_h
			})
	}

	fn start_sfx_on_channel(
		&mut self,
		sfx: SoundEffect,
		channel: Option<usize>,
	) -> Result<(), Error> {
		let channel = match channel {
			Some(c) => c,
			None => self
				.sounds
				.iter()
				.rev()
				.position(|snd| snd.is_none())
				.unwrap_or(self.sounds.len() - 1),
		};

		let chan_len = self.sounds.len();

		let snd = self
			.sounds
			.get_mut(channel)
			.ok_or(Error::InvalidChannel { channel, chan_len })?;
		let prev = std::mem::replace(snd, Some(sfx));

		if let Some(mut p) = prev {
			p.stop(Tween::default())?;
		}

		Ok(())
	}

	/// Instantly resumes every sound and music handle.
	pub fn resume_all(&mut self) -> Result<(), Error> {
		self.manager.resume(Tween::default()).map_err(Error::from)
	}

	/// Instantly pauses every sound and music handle.
	pub fn pause_all(&mut self) -> Result<(), Error> {
		self.manager.pause(Tween::default()).map_err(Error::from)
	}

	/// Instantly stops all ongoing audio.
	/// The sound array gets cleared along with the music stack.
	pub fn stop_all(&mut self) {
		self.stop_all_music();
		self.stop_all_sounds();
	}

	/// Instantly stops all handles in the music stack and clears it.
	pub fn stop_all_music(&mut self) {
		for mut group in self.music.drain(..) {
			group.stop_all_layers(None);
		}
	}

	/// Uses the [`rayon`] global thread pool.
	/// Logs an error if stopping a sound fails.
	pub fn stop_all_sounds(&mut self) {
		self.sounds.par_iter_mut().enumerate().for_each(|(i, sfx)| {
			if let Some(mut s) = sfx.take() {
				if let Err(err) = s.stop(Tween::default()) {
					error!("`stop_all_sounds` failed to stop sound {i}: {err}");
				}
			}
		});
	}

	/// Clears sound handles which have finished playing.
	/// Uses the [`rayon`] global thread pool.
	pub fn update(&mut self) {
		self.sounds.par_iter_mut().for_each(|snd| {
			if snd
				.as_ref()
				.is_some_and(|s| s.sound.state() == PlaybackState::Stopped)
			{
				*snd = None;
			}
		});
	}

	#[must_use]
	pub fn listener(&self) -> &ListenerHandle {
		&self.listener
	}

	#[must_use]
	pub fn listener_mut(&mut self) -> &mut ListenerHandle {
		&mut self.listener
	}

	#[must_use]
	pub fn sfx_channels(&self) -> &[Option<SoundEffect>] {
		&self.sounds
	}

	#[must_use]
	pub fn sfx_channels_mut(&mut self) -> &mut [Option<SoundEffect>] {
		&mut self.sounds
	}

	#[must_use]
	pub fn music(&self, index: usize) -> Option<&MusicController> {
		self.music.get(index)
	}

	#[must_use]
	pub fn music_mut(&mut self, index: usize) -> Option<&mut MusicController> {
		self.music.get_mut(index)
	}

	#[must_use]
	pub fn music_stack(&self) -> &ArrayVec<MusicController, 3> {
		&self.music
	}

	#[must_use]
	pub fn music_stack_mut(&mut self) -> &mut ArrayVec<MusicController, 3> {
		&mut self.music
	}

	#[must_use]
	pub fn subtracks(&self) -> &SubTracks {
		&self.tracks
	}

	#[must_use]
	pub fn subtracks_mut(&mut self) -> &mut SubTracks {
		&mut self.tracks
	}

	/// Draw the egui-based developer/debug/diagnosis menu, and perform any
	/// state mutations requested through it by the user.
	pub fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, catalog: &Catalog) {
		self.ui_impl(ctx, ui, catalog);
	}
}

impl Drop for AudioCore {
	fn drop(&mut self) {
		self.stop_all();
	}
}

impl std::fmt::Debug for AudioCore {
	fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		unimplemented!("proper `Debug` impl. pending internal stabilization")
	}
}

#[derive(Clone)]
pub enum SoundSpace {
	/// The sound will be played to the "sound effects" track and use its
	/// volume control and effects, but not be spatial.
	Unsourced,
	/// The sound will be played to the "menu" track, and use its volume control
	/// and effects.
	Menu,
	/// The sound will go through a spatial emitter to the "sound effects" track
	/// and use its volume control and effects.
	World {
		pos: glam::Vec3,
		entity: Option<Entity>,
		settings: EmitterSettings,
	},
}

// Handles to playing sounds ///////////////////////////////////////////////////

/// Enables inspection and control of a currently-playing sound or musical track,
/// whether it's waveform or MIDI.
pub enum Handle {
	Wave(StaticSoundHandle),
	Midi(MidiHandle),
}

impl Handle {
	#[must_use]
	pub fn state(&self) -> PlaybackState {
		match self {
			Self::Wave(wave) => wave.state(),
			Self::Midi(midi) => midi.state(),
		}
	}

	pub fn pause(&mut self, tween: Tween) -> Result<(), Error> {
		match self {
			Handle::Wave(wave) => wave.pause(tween).map_err(Error::from),
			Handle::Midi(midi) => midi.pause(tween),
		}
	}

	pub fn resume(&mut self, tween: Tween) -> Result<(), Error> {
		match self {
			Handle::Wave(wave) => wave.resume(tween).map_err(Error::from),
			Handle::Midi(midi) => midi.resume(tween),
		}
	}

	pub fn stop(&mut self, tween: Tween) -> Result<(), Error> {
		match self {
			Handle::Wave(wave) => wave.stop(tween).map_err(Error::from),
			Handle::Midi(midi) => midi.stop(tween),
		}
	}

	#[must_use]
	pub fn is_playing(&self) -> bool {
		match self {
			Handle::Wave(wave) => wave.state() == PlaybackState::Playing,
			Handle::Midi(midi) => midi.is_playing(),
		}
	}
}

impl std::fmt::Debug for Handle {
	fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		unimplemented!("proper `Debug` impl. pending internal stabilization")
	}
}

/// An audio track that may be playing spatially.
pub struct SoundEffect {
	sound: Handle,
	entity: Option<Entity>,
	emitter: Option<EmitterHandle>,
}

impl SoundEffect {
	#[must_use]
	pub fn entity(&self) -> Option<Entity> {
		self.entity
	}

	/// This is a valid no-op if this sound effect has no attached emitter;
	/// `Ok` is always returned in these cases.
	pub fn reposition(&mut self, pos: glam::Vec3, tween: Option<Tween>) -> Result<(), Error> {
		if let Some(e) = &mut self.emitter {
			e.set_position(pos, tween.unwrap_or_default())
				.map_err(Error::from)
		} else {
			Ok(())
		}
	}
}

impl std::ops::Deref for SoundEffect {
	type Target = Handle;

	fn deref(&self) -> &Self::Target {
		&self.sound
	}
}

impl std::ops::DerefMut for SoundEffect {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.sound
	}
}

impl std::fmt::Debug for SoundEffect {
	fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		unimplemented!("proper `Debug` impl. pending internal stabilization")
	}
}

/// A [`TrackHandle`] with effect handles attached.
pub struct TrackController {
	pub handle: TrackHandle,
	pub reverb: ReverbHandle,
	pub delay: DelayHandle,
}

impl TrackController {
	fn new(manager: &mut AudioManager, routes: Option<TrackRoutes>) -> Result<Self, Error> {
		let mut builder = match routes {
			Some(r) => TrackBuilder::new().routes(r),
			None => TrackBuilder::new(),
		};

		let reverb = builder.add_effect(ReverbBuilder::new());
		let delay = builder.add_effect(DelayBuilder::new());

		Ok(TrackController {
			handle: manager.add_sub_track(builder).map_err(Error::SubTrack)?,
			reverb,
			delay,
		})
	}
}

impl std::fmt::Debug for TrackController {
	fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		unimplemented!("proper `Debug` impl. pending internal stabilization")
	}
}

pub struct MusicController {
	pub layers: [Option<Handle>; MUSIC_LAYERS],
}

impl MusicController {
	/// If `None` is given, all layers are stopped instantly.
	/// Logs an error if a layer fails to stop.
	pub fn stop_all_layers(&mut self, tween: Option<Tween>) {
		for (i, layer) in self.layers.iter_mut().enumerate() {
			let Some(handle) = layer else { continue; };

			if let Err(err) = handle.stop(tween.unwrap_or_default()) {
				error!("Failed to stop music layer: {i} ({err})");
			}

			*layer = None;
		}
	}

	/// If `None` is given, all layers are paused instantly.
	/// Logs an error if a layer fails to pause.
	pub fn pause_all_layers(&mut self, tween: Option<Tween>) {
		for (i, layer) in self.layers.iter_mut().enumerate() {
			let Some(handle) = layer else { continue; };

			if let Err(err) = handle.pause(tween.unwrap_or_default()) {
				error!("Failed to pause music layer: {i} ({err})");
			}

			*layer = None;
		}
	}
}

impl std::fmt::Debug for MusicController {
	fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		unimplemented!("proper `Debug` impl. pending internal stabilization")
	}
}

// Helpers /////////////////////////////////////////////////////////////////////

pub fn sound_from_bytes(
	bytes: impl Into<Vec<u8>>,
	settings: StaticSoundSettings,
) -> Result<StaticSoundData, kira0_8_3::sound::FromFileError> {
	let cursor = Cursor::new(bytes.into());
	StaticSoundData::from_cursor(cursor, settings)
}

// Error ///////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum Error {
	// TODO: `kira::AddEmitterError` isn't exposed, likely mistakenly.
	AddEmitter(Box<dyn std::error::Error>),
	CommandOverflow,
	InvalidChannel {
		channel: usize,
		chan_len: usize,
	},
	InvalidMusicLayer {
		song: usize,
		layer: usize,
	},
	KiraBackend(<CpalBackend as Backend>::Error),
	Listener(AddListenerError),
	MidiSynth(fluidlite::Error),
	/// Tried to push a new music layer group onto the stack, but it was full.
	MusicFull,
	ParseMidi(midly::Error),
	PlayMidi(PlayMidiError),
	PlayWave(PlayWaveError),
	SoundFontRead(PathBuf, fluidlite::Error),
	SpatialScene(AddSpatialSceneError),
	SubTrack(AddSubTrackError),
	ThreadPanic,
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

impl From<SendError<midi::Command>> for Error {
	fn from(_: SendError<midi::Command>) -> Self {
		Self::ThreadPanic
	}
}

pub type PlayWaveError = PlaySoundError<<StaticSoundData as SoundData>::Error>;
pub type PlayMidiError = PlaySoundError<<MidiData as SoundData>::Error>;

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Self::AddEmitter(err) => Some(err.as_ref()),
			Self::KiraBackend(err) => Some(err),
			Self::Listener(err) => Some(err),
			Self::MidiSynth(err) => Some(err),
			Self::ParseMidi(err) => Some(err),
			Self::PlayMidi(err) => Some(err),
			Self::PlayWave(err) => Some(err),
			Self::SoundFontRead(_, err) => Some(err),
			Self::SpatialScene(err) => Some(err),
			Self::SubTrack(err) => Some(err),
			Self::CommandOverflow
			| Self::InvalidChannel { .. }
			| Self::InvalidMusicLayer { .. }
			| Self::MusicFull
			| Self::ThreadPanic => None,
		}
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::AddEmitter(err) => {
				write!(f, "failed to create a spatial sound's emitter: {err}")
			}
			Self::CommandOverflow => {
				write!(f, "tried to send too many commands to a sound at once")
			}
			Self::InvalidChannel {
				channel: chan,
				chan_len,
			} => {
				write!(
					f,
					"channel is out-of-bounds: {chan} (only {chan_len} exist)"
				)
			}
			Self::InvalidMusicLayer { song, layer } => {
				write!(f, "tried to access layer {layer} of song {song} (only {MUSIC_LAYERS} layers exist)")
			}
			Self::KiraBackend(err) => write!(f, "audio backend error: {err}"),
			Self::Listener(err) => write!(f, "failed to create spatial audio listener: {err}"),
			Self::MidiSynth(err) => write!(f, "MIDI synthesis error: {err}"),
			Self::MusicFull => write!(
				f,
				"tried to push a new music layer group, but the music stack is full"
			),
			Self::ParseMidi(err) => write!(f, "failed to parse MIDI file: {err}"),
			Self::PlayMidi(err) => write!(f, "failed to play MIDI audio: {err}"),
			Self::PlayWave(err) => write!(f, "failed to play non-MIDI audio: {err}"),
			Self::SoundFontRead(path, err) => write!(
				f,
				"failed to read SoundFont at path: `{}` - details: {err}",
				path.display()
			),
			Self::SpatialScene(err) => write!(f, "failed to create spatial scene: {err}"),
			Self::SubTrack(err) => write!(f, "failed to create an audio sub-track: {err}"),
			Self::ThreadPanic => write!(f, "audio thread has panicked"),
		}
	}
}

#[cfg(test)]
mod test {
	use std::path::Path;

	use kira0_8_3::sound::static_sound::StaticSoundSettings;
	use nodi::midly::Smf;

	use super::*;

	fn read_sample_data(env_var_name: &'static str) -> Result<(PathBuf, Vec<u8>), String> {
		let path = match std::env::var(env_var_name) {
			Ok(p) => PathBuf::from(p),
			Err(err) => {
				return Err(format!(
					"failed to get environment variable `{env_var_name}` ({err})"
				))
			}
		};

		if !path.exists() {
			return Err(format!("file `{}` does not exist", path.display()));
		}

		let bytes = match std::fs::read(&path) {
			Ok(b) => b,
			Err(err) => return Err(format!("{err}")),
		};

		Ok((path, bytes))
	}

	#[test]
	fn waveform_with_sample_data() {
		let path = match read_sample_data("VILETECH_WAVESND_SAMPLE") {
			Ok(b) => b.0,
			Err(err) => {
				eprintln!("Skipping sample data-based waveform audio unit test. Reason: {err}");
				return;
			}
		};

		let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())
			.expect("manager construction failed");
		let w_data = StaticSoundData::from_file(path, StaticSoundSettings::default())
			.expect("file read or decode failed");
		let mut handle = manager.play(w_data).expect("playback start failed");
		std::thread::sleep(std::time::Duration::from_secs(30));
		handle.stop(Tween::default()).expect("sound stop failed");
	}

	#[test]
	fn midi_with_sample_data() {
		let sfpath: PathBuf = [
			Path::new(env!("CARGO_WORKSPACE_DIR")),
			Path::new("data/soundfonts/viletech.sf2"),
		]
		.iter()
		.collect();

		let bytes = match read_sample_data("VILETECH_MIDI_SAMPLE") {
			Ok(b) => b.1,
			Err(err) => {
				eprintln!("Skipping sample data-based MIDI unit test. Reason: {err}");
				return;
			}
		};

		let smf = Smf::parse(&bytes).expect("SMF parsing failed");
		let m_data = MidiData::new(smf, sfpath, MidiSettings::default());
		let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())
			.expect("manager construction failed");
		let handle = manager.play(m_data).expect("playback start failed");
		std::thread::sleep(std::time::Duration::from_secs(30));
		handle.stop(Tween::default()).expect("sound stop failed");
	}
}
