//! Interface between [`kira`] and [`zmusic`].

/*

Copyright (C) 2022 ***REMOVED***

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <http://www.gnu.org/licenses/>.

*/

use std::{
	collections::VecDeque,
	sync::{
		atomic::{AtomicU8, Ordering as AtomicOrdering},
		Arc,
	},
};

use crossbeam::channel::{Receiver, Sender};
use kira::{
	clock::clock_info::ClockInfoProvider,
	dsp::Frame,
	sound::static_sound::PlaybackState,
	track::TrackId,
	tween::{Tween, Tweener},
	Volume,
};

use super::Error;

/// Intended to parallel the interface of [`StaticSoundData`].
///
/// [`StaticSoundData`]: kira::sound::static_sound::StaticSoundData
pub struct MidiData {
	song: zmusic::Song,
	pub settings: MidiSettings,
}

impl MidiData {
	#[must_use]
	pub fn new(song: zmusic::Song, settings: MidiSettings) -> Self {
		Self { song, settings }
	}
}

impl kira::sound::SoundData for MidiData {
	type Error = zmusic::Error;
	type Handle = MidiSoundHandle;

	fn into_sound(self) -> Result<(Box<dyn kira::sound::Sound>, Self::Handle), Self::Error> {
		debug_assert!(self.song.is_stopped());

		// Temporary destructure for `mut song`
		let MidiData { mut song, settings } = self;

		let stream_info = song.start_silent(settings.looping)?;
		let buffer_size = (stream_info.buffer_size / 10) as usize;
		let mut sample_buffer = Vec::with_capacity(buffer_size);
		sample_buffer.resize(buffer_size, 0.0);
		let shared = Arc::new(Shared {
			state: AtomicU8::new(STATE_PLAYING),
		});
		let (sender, receiver) = crossbeam::channel::unbounded();
		let shared_kept = shared.clone();

		let volume = Tweener::new(settings.volume);
		let volume_fade = match settings.fade_in {
			Some(fade_in) => {
				let mut tweenable = Tweener::new(Volume::Decibels(Volume::MIN_DECIBELS));
				tweenable.set(Volume::Decibels(0.0), fade_in);
				tweenable
			}
			None => Tweener::new(Volume::Decibels(0.0)),
		};
		let panning = Tweener::new(settings.panning);

		let sound = MidiSound {
			shared,
			state: PlaybackState::Playing,
			data: Self { song, settings },
			finished: false,
			stream_info,
			frames: Default::default(),
			sample_buffer,
			receiver,
			volume,
			volume_fade,
			panning,
		};

		Ok((
			Box::new(sound),
			MidiSoundHandle {
				shared: shared_kept,
				sender,
			},
		))
	}
}

/// Intended to parallel the interface of [`StaticSoundSettings`].
///
/// [`StaticSoundSettings`]: kira::sound::static_sound::StaticSoundSettings
#[derive(Debug, Clone, PartialEq)]
pub struct MidiSettings {
	pub track: TrackId,
	pub looping: bool,
	pub volume: Volume,
	/// The panning of the sound, where 0 is hard left and 1 is hard right.
	pub panning: f64,
	pub fade_in: Option<Tween>,
}

impl Default for MidiSettings {
	fn default() -> Self {
		Self {
			track: TrackId::Main,
			looping: false,
			volume: Volume::Amplitude(1.0),
			panning: 0.5,
			fade_in: None,
		}
	}
}

/// Intended to parallel the interface of [`StaticSoundHandle`].
///
/// [`StaticSoundHandle`]: kira::sound::static_sound::StaticSoundHandle
pub struct MidiSoundHandle {
	shared: Arc<Shared>,
	sender: Sender<MidiCommand>,
}

impl MidiSoundHandle {
	#[must_use]
	pub fn state(&self) -> PlaybackState {
		self.shared.playback_state()
	}

	#[must_use]
	pub fn is_playing(&self) -> bool {
		self.state() == PlaybackState::Playing
	}

	pub fn pause(&mut self, tween: Tween) -> Result<(), Error> {
		self.sender
			.send(MidiCommand::Pause(tween))
			.map_err(|_| Error::CommandMidi)
	}

	pub fn resume(&mut self, tween: Tween) -> Result<(), Error> {
		self.sender
			.send(MidiCommand::Resume(tween))
			.map_err(|_| Error::CommandMidi)
	}

	pub fn stop(&mut self, tween: Tween) -> Result<(), Error> {
		self.sender
			.send(MidiCommand::Stop(tween))
			.map_err(|_| Error::CommandMidi)
	}

	pub fn set_volume(&mut self, volume: impl Into<Volume>, tween: Tween) -> Result<(), Error> {
		self.sender
			.send(MidiCommand::SetVolume {
				volume: volume.into(),
				tween,
			})
			.map_err(|_| Error::CommandMidi)
	}

	pub fn set_panning(&mut self, panning: f64, tween: Tween) -> Result<(), Error> {
		self.sender
			.send(MidiCommand::SetPanning { panning, tween })
			.map_err(|_| Error::CommandMidi)
	}
}

// Internal details ////////////////////////////////////////////////////////////
#[derive(Debug)]
struct Shared {
	state: AtomicU8,
}

const STATE_PLAYING: u8 = 0;
const STATE_PAUSING: u8 = 1;
const STATE_PAUSED: u8 = 2;
const STATE_STOPPING: u8 = 3;
const STATE_STOPPED: u8 = 4;

impl Shared {
	#[must_use]
	fn playback_state(&self) -> PlaybackState {
		match self.state.load(AtomicOrdering::SeqCst) {
			STATE_PLAYING => PlaybackState::Playing,
			STATE_PAUSING => PlaybackState::Pausing,
			STATE_PAUSED => PlaybackState::Paused,
			STATE_STOPPING => PlaybackState::Stopping,
			STATE_STOPPED => PlaybackState::Stopped,
			_ => unreachable!(),
		}
	}

	fn set_state(&self, state: PlaybackState) {
		self.state.store(
			match state {
				PlaybackState::Playing => STATE_PLAYING,
				PlaybackState::Pausing => STATE_PAUSING,
				PlaybackState::Paused => STATE_PAUSED,
				PlaybackState::Stopping => STATE_STOPPING,
				PlaybackState::Stopped => STATE_STOPPED,
			},
			AtomicOrdering::SeqCst,
		);
	}
}

struct MidiSound {
	shared: Arc<Shared>,
	state: PlaybackState,
	data: MidiData,
	/// Indicates that the ZMusic synth has no more samples to give, even if the
	/// song is not necessarily finished its entire runtime.
	finished: bool,
	stream_info: zmusic::song::StreamInfo,
	frames: VecDeque<Frame>,
	sample_buffer: Vec<f32>,
	receiver: Receiver<MidiCommand>,
	volume: Tweener<Volume>,
	volume_fade: Tweener<Volume>,
	panning: Tweener,
}

impl kira::sound::Sound for MidiSound {
	fn track(&mut self) -> kira::track::TrackId {
		self.data.settings.track
	}

	fn process(&mut self, dt: f64, clock_info_provider: &ClockInfoProvider) -> Frame {
		if self.shared.state.load(AtomicOrdering::SeqCst) == STATE_PAUSED {
			return Frame::ZERO;
		}

		self.volume.update(dt, clock_info_provider);
		self.panning.update(dt, clock_info_provider);

		if self.volume_fade.update(dt, clock_info_provider) {
			match self.state {
				PlaybackState::Pausing => {
					self.state = PlaybackState::Paused;
					self.shared.set_state(PlaybackState::Paused);
				}
				PlaybackState::Stopping => {
					self.state = PlaybackState::Stopped;
					self.shared.set_state(PlaybackState::Stopped);
				}
				_ => {}
			}
		}

		if let Some(frame) = self.frames.pop_front() {
			frame
		} else {
			self.on_start_processing();
			self.frames.pop_front().unwrap()
		}
	}

	fn finished(&self) -> bool {
		(self.finished && self.frames.is_empty()) || self.state == PlaybackState::Stopped
	}

	fn on_start_processing(&mut self) {
		self.finished = !self.data.song.fill_stream(&mut self.sample_buffer);
		self.data.song.update();

		if self.stream_info.num_channels == 1 {
			for frame in self.sample_buffer.chunks_exact_mut(1) {
				let mut frame = Frame {
					left: frame[0],
					right: frame[0],
				};

				frame *= self.volume.value().as_amplitude() as f32;
				frame = frame.panned(self.panning.value() as f32);

				self.frames.push_back(frame);
			}
		} else {
			for frame in self.sample_buffer.chunks_exact_mut(2) {
				let mut frame = Frame {
					left: frame[0],
					right: frame[1],
				};

				frame *= self.volume_fade.value().as_amplitude() as f32;
				frame *= self.volume.value().as_amplitude() as f32;
				frame = frame.panned(self.panning.value() as f32);

				self.frames.push_back(frame);
			}
		}

		while let Ok(cmd) = self.receiver.try_recv() {
			match cmd {
				MidiCommand::SetVolume { volume, tween } => self.volume.set(volume, tween),
				MidiCommand::SetPanning { panning, tween } => self.panning.set(panning, tween),
				MidiCommand::Pause(tween) => {
					self.state = PlaybackState::Pausing;
					self.shared.set_state(PlaybackState::Pausing);
					self.volume_fade
						.set(Volume::Decibels(Volume::MIN_DECIBELS), tween);
				}
				MidiCommand::Resume(tween) => {
					self.state = PlaybackState::Playing;
					self.shared.set_state(PlaybackState::Playing);
					self.volume_fade.set(Volume::Decibels(0.0), tween);
				}
				MidiCommand::Stop(tween) => {
					self.state = PlaybackState::Stopping;
					self.shared.set_state(PlaybackState::Stopping);
					self.volume_fade
						.set(Volume::Decibels(Volume::MIN_DECIBELS), tween);
				}
			}
		}
	}
}

#[derive(Debug)]
enum MidiCommand {
	SetVolume { volume: Volume, tween: Tween },
	SetPanning { panning: f64, tween: Tween },
	Pause(Tween),
	Resume(Tween),
	Stop(Tween),
}
