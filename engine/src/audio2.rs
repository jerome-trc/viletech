//! Sound- and music-related code.

mod gui;
mod midi;

use std::path::PathBuf;

use arrayvec::ArrayVec;
use bevy_egui::egui;
use kira0_8_3::{
	manager::{
		backend::{cpal::CpalBackend, Backend},
		error::{AddSpatialSceneError, AddSubTrackError},
		AudioManager, AudioManagerSettings,
	},
	sound::static_sound::StaticSoundHandle,
	spatial::{
		emitter::EmitterHandle,
		listener::{ListenerHandle, ListenerSettings},
		scene::{AddListenerError, SpatialSceneHandle, SpatialSceneSettings},
	},
	track::{
		effect::reverb::{ReverbBuilder, ReverbHandle},
		TrackBuilder, TrackHandle,
	},
};
use nodi::midly;
use tracing::{debug, error, info, warn};

use crate::data::Catalog;

pub use self::midi::{Format as MidiFormat, SoundFont, SoundFontKind};

#[derive(bevy::ecs::system::Resource)]
pub struct AudioCore {
	manager: AudioManager,
	soundfonts: Vec<SoundFont>,
	scene: SpatialSceneHandle,
	listener: ListenerHandle,

	/// Unlike `Self::sounds`, this behaves like a stack.
	/// Only the last element gets played at any given time.
	music: ArrayVec<MusicController, 3>,
	sounds: Vec<WorldSound>,

	t_sfx: TrackController,
	t_music: TrackController,
	t_menu: TrackController,
}

// SAFETY: This is for `kira::AudioManager`, which uses sufficient internal
// synchronization to be safe but is missing a trait bound on a `PhantomData`.
unsafe impl Sync for AudioCore {}

impl AudioCore {
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

		let t_sfx = {
			let mut builder = TrackBuilder::new();
			let reverb = builder.add_effect(ReverbBuilder::new());

			TrackController {
				handle: manager.add_sub_track(builder).map_err(Error::SubTrack)?,
				reverb,
			}
		};

		let t_music = {
			let mut builder = TrackBuilder::new();
			let reverb = builder.add_effect(ReverbBuilder::new());

			TrackController {
				handle: manager
					.add_sub_track(TrackBuilder::new())
					.map_err(Error::SubTrack)?,
				reverb,
			}
		};

		let t_menu = {
			let mut builder = TrackBuilder::new();
			let reverb = builder.add_effect(ReverbBuilder::new());

			TrackController {
				handle: manager
					.add_sub_track(TrackBuilder::new())
					.map_err(Error::SubTrack)?,
				reverb,
			}
		};

		let mut scene = manager
			.add_spatial_scene(s_settings)
			.map_err(Error::Spatial)?;

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

			music: ArrayVec::new(),
			sounds: vec![],

			t_sfx,
			t_music,
			t_menu,
		};

		Ok(ret)
	}

	/// Draw the egui-based developer/debug/diagnosis menu, and perform any
	/// state mutations requested through it by the user.
	pub fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, catalog: &Catalog) {
		todo!()
	}
}

impl std::fmt::Debug for AudioCore {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		unimplemented!("proper `Debug` impl. pending internal stabilization")
	}
}

/// Enables inspection and control of a currently-playing sound or musical track,
/// whether it's waveform or MIDI.
enum Sound {
	Wave(StaticSoundHandle),
}

impl Sound {
	// Soon!
}

impl std::fmt::Debug for Sound {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		unimplemented!("proper `Debug` impl. pending internal stabilization")
	}
}

/// A [`TrackHandle`] with effect handles attached.
///
/// Currently only offers reverb; if use cases arise for other effects, they
/// can be added later.
struct TrackController {
	handle: TrackHandle,
	reverb: ReverbHandle,
}

struct MusicController {
	layers: [Option<Sound>; 8],
}

impl std::fmt::Debug for MusicController {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		unimplemented!("proper `Debug` impl. pending internal stabilization")
	}
}

struct WorldSound {
	sound: Sound,
	emitter: EmitterHandle,
}

impl std::ops::Deref for WorldSound {
	type Target = Sound;

	fn deref(&self) -> &Self::Target {
		&self.sound
	}
}

impl std::fmt::Debug for WorldSound {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		unimplemented!("proper `Debug` impl. pending internal stabilization")
	}
}

// Error ///////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum Error {
	CommandOverflow,
	KiraBackend(<CpalBackend as Backend>::Error),
	Listener(AddListenerError),
	ParseMidi(midly::Error),
	PlayWave(PlayWaveError),
	SoundFontRead(PathBuf, fluidlite::Error),
	Spatial(AddSpatialSceneError),
	SubTrack(AddSubTrackError),
	ThreadPanic,
}

impl std::error::Error for Error {}
