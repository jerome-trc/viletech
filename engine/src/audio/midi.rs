//! Interface between [`kira`] and [`nodi`]/[`fluidlite`].
//!
//! Much of this code is a copy-paste of `kira`'s internal sound sampling code.

use std::{
	collections::VecDeque,
	path::{Path, PathBuf},
	sync::{
		atomic::{AtomicU8, Ordering as AtomicOrdering},
		Arc,
	},
	time::Duration,
};

use cpal::{
	traits::{DeviceTrait, HostTrait},
	SampleFormat,
};
use crossbeam::channel::{Receiver, Sender};
use fluidlite::Synth;
use kira::{
	clock::clock_info::ClockInfoProvider,
	dsp::Frame,
	sound::{
		static_sound::{PlaybackState, StaticSoundData, StaticSoundSettings},
		SoundData,
	},
	track::TrackId,
	tween::{Tween, Tweener},
	LoopBehavior, Volume,
};
use nodi::{
	midly::{Format, MetaMessage, Smf, Timing, TrackEvent, TrackEventKind},
	timers::{FixedTempo, Ticker},
	Connection, Moment, Sheet, Timer,
};

use super::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Device {
	FluidSynth,
}

impl std::fmt::Display for Device {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::FluidSynth => write!(f, "FluidSynth"),
		}
	}
}

/// Intended to parallel the interface of [`StaticSoundData`].
///
/// [`StaticSoundData`]: kira::sound::static_sound::StaticSoundData
#[derive(Clone, Debug)]
pub struct Data {
	pub settings: Settings,
	pub soundfont: PathBuf,
	sheet: Arc<Sheet>,
	timing: Timing,
}

impl Data {
	pub fn new(midi: Smf, soundfont: PathBuf, settings: Settings) -> Self {
		let Smf { header, tracks } = midi;

		let sheet = match header.format {
			Format::SingleTrack | Format::Sequential => Sheet::sequential(&tracks),
			Format::Parallel => Sheet::parallel(&tracks),
		};

		Self {
			settings,
			soundfont,
			sheet: Arc::new(sheet),
			timing: header.timing,
		}
	}

	#[must_use]
	fn create_timer(&self) -> Box<dyn Timer + Send + Sync> {
		match &self.timing {
			Timing::Metrical(n) => {
				let mut t = Ticker::new(n.as_int());
				t.speed = 1.0;
				Box::new(t)
			}
			Timing::Timecode(fps, subframe) => {
				let micros_per_tick = 1_000_000.0 / fps.as_f32() / *subframe as f32;
				let t = FixedTempo(micros_per_tick as u64);
				Box::new(t)
			}
		}
	}

	/// Note; these cases are allegedly extremely rare.
	#[allow(unused)]
	fn timecode_duration(
		moments: &[Moment],
		tracks: &[Vec<TrackEvent>],
		fps: f32,
		subframe: f32,
	) -> Duration {
		let tempo = tracks.iter().find_map(|track| {
			track.iter().find_map(|&tevent| {
				if let TrackEventKind::Meta(MetaMessage::Tempo(t)) = tevent.kind {
					Some(t)
				} else {
					None
				}
			})
		});

		let secs_per_tick = 1.0 / fps / subframe;

		let tempo = match tempo {
			// If the MIDI is malformed and has no tempo meta-message,
			// we just have to make a best guess.
			None => return Duration::from_secs_f32(secs_per_tick * moments.len() as f32),
			Some(t) => t.as_int(),
		};

		let mut ret = Duration::default();
		let mut micros_per_tick = secs_per_tick * 1_000_000.0;
		let micros_per_beat = tempo as f32;
		let ticks_per_beat = micros_per_beat / micros_per_tick;

		for moment in moments {
			for event in &moment.events {
				if let nodi::Event::Tempo(val) = event {
					micros_per_tick = (*val as f32) / ticks_per_beat;
				}
			}

			ret += Duration::from_micros(micros_per_tick as u64);
		}

		ret
	}

	#[must_use]
	pub fn duration(&self) -> Duration {
		match &self.timing {
			Timing::Metrical(n) => {
				let mut t = Ticker::new(n.as_int());
				t.duration(&self.sheet)
			}
			Timing::Timecode(fps, subframe) => {
				let micros_per_tick = 1_000_000.0 / fps.as_f32() / *subframe as f32;
				let mut t = FixedTempo(micros_per_tick as u64);
				t.duration(&self.sheet)
			}
		}
	}
}

impl SoundData for Data {
	type Error = Box<Error>; // Indirection prevents a recursive type.
	type Handle = Handle;

	fn into_sound(self) -> Result<(Box<dyn kira::sound::Sound>, Self::Handle), Self::Error> {
		// `sleep_duration` demands `mut` but it never does any real mutation.
		let mut timer = self.create_timer();

		let _ = timer.duration(&self.sheet);
		let tick_len = timer.sleep_duration(1);

		let renderer = Renderer::new(tick_len, &self.soundfont)?;
		let (sender, receiver) = crossbeam::channel::unbounded();

		let shared = Arc::new(Shared {
			state: AtomicU8::new(STATE_PLAYING),
		});

		let shared_kept = shared.clone();

		let volume = Tweener::new(self.settings.volume);
		let volume_fade = match self.settings.fade_in {
			Some(fade_in) => {
				let mut tweenable = Tweener::new(Volume::Decibels(Volume::MIN_DECIBELS));
				tweenable.set(Volume::Decibels(0.0), fade_in);
				tweenable
			}
			None => Tweener::new(Volume::Decibels(0.0)),
		};
		let panning = Tweener::new(self.settings.panning);

		let sound = Sound {
			shared,
			state: PlaybackState::Playing,
			data: self,
			frames: VecDeque::default(),
			renderer,
			position: 0,
			to_wait: tick_len,
			timer,
			receiver,

			volume,
			volume_fade,
			panning,
		};

		Ok((
			Box::new(sound),
			Handle {
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
pub struct Settings {
	pub track: TrackId,
	pub looping: Option<LoopBehavior>,
	pub reverse: bool,
	pub volume: Volume,
	/// The panning of the sound, where 0 is hard left and 1 is hard right.
	pub panning: f64,
	pub fade_in: Option<Tween>,
}

impl Default for Settings {
	fn default() -> Self {
		Self {
			track: TrackId::Main,
			looping: None,
			reverse: false,
			volume: Volume::Amplitude(1.0),
			panning: 0.5,
			fade_in: None,
		}
	}
}

/// Intended to parallel the interface of [`StaticSoundHandle`].
///
/// [`StaticSoundHandle`]: kira::sound::static_sound::StaticSoundHandle
#[derive(Debug)]
pub struct Handle {
	shared: Arc<Shared>,
	sender: Sender<Command>,
}

impl Handle {
	/// Returns the current playback state of the sound.
	#[must_use]
	pub fn state(&self) -> PlaybackState {
		self.shared.playback_state()
	}

	#[must_use]
	pub fn is_playing(&self) -> bool {
		self.state() == PlaybackState::Playing
	}

	pub fn pause(&mut self, tween: Tween) -> Result<(), Error> {
		self.sender.send(Command::Pause(tween)).map_err(Error::from)
	}

	pub fn resume(&mut self, tween: Tween) -> Result<(), Error> {
		self.sender
			.send(Command::Resume(tween))
			.map_err(Error::from)
	}

	pub fn stop(&mut self, tween: Tween) -> Result<(), Error> {
		self.sender.send(Command::Stop(tween)).map_err(Error::from)
	}

	pub fn set_volume(&mut self, volume: impl Into<Volume>, tween: Tween) -> Result<(), Error> {
		self.sender
			.send(Command::SetVolume(volume.into(), tween))
			.map_err(Error::from)
	}

	pub fn set_panning(&mut self, panning: f64, tween: Tween) -> Result<(), Error> {
		self.sender
			.send(Command::SetPanning(panning, tween))
			.map_err(Error::from)
	}
}

/// Converts a MIDI into a waveform sound.
///
/// Note that the output is configured using `data.settings`, not `settings`.
pub fn render(data: Data, settings: StaticSoundSettings) -> Result<StaticSoundData, Error> {
	let mut timer = data.create_timer();
	let _ = timer.duration(&data.sheet);
	let mut renderer = Renderer::new(timer.sleep_duration(1), &data.soundfont)?;
	let mut frames = Vec::default();

	for moment in data.sheet.iter() {
		for mut frame in renderer.render(moment) {
			frame *= data.settings.volume.as_amplitude() as f32;
			frame = frame.panned(data.settings.panning as f32);
			frames.push(frame);
		}
	}

	Ok(StaticSoundData {
		sample_rate: renderer.sample_rate,
		frames: Arc::new(frames),
		settings,
	})
}

// Internal: Sound /////////////////////////////////////////////////////////////

struct Sound {
	shared: Arc<Shared>,
	state: PlaybackState,
	data: Data,
	renderer: Renderer,
	frames: VecDeque<Frame>,
	/// Which [`Moment`] in `data`'s sheet are we at?
	position: usize,
	timer: Box<dyn Timer + Send + Sync>,
	to_wait: Duration,
	receiver: Receiver<Command>,

	volume: Tweener<Volume>,
	volume_fade: Tweener<Volume>,
	panning: Tweener,
}

impl Sound {
	#[must_use]
	fn sheet_finished(&self) -> bool {
		self.position >= self.data.sheet.len()
	}

	fn render_moment(&mut self) {
		let moment = &self.data.sheet[self.position];

		for mut frame in self.renderer.render_with_timer(moment, self.timer.as_mut()) {
			frame *= self.volume_fade.value().as_amplitude() as f32;
			frame *= self.volume.value().as_amplitude() as f32;
			frame = frame.panned(self.panning.value() as f32);
			self.frames.push_back(frame);
		}
	}
}

impl kira::sound::Sound for Sound {
	fn track(&mut self) -> TrackId {
		self.data.settings.track
	}

	fn process(&mut self, dt: f64, clock_info_provider: &ClockInfoProvider) -> Frame {
		self.volume.update(dt, clock_info_provider);
		self.panning.update(dt, clock_info_provider);

		if self.volume_fade.update(dt, clock_info_provider) {
			match self.state {
				PlaybackState::Pausing => {
					self.state = PlaybackState::Paused;
					self.shared.set_state(PlaybackState::Paused);
					self.renderer.all_notes_off();
				}
				PlaybackState::Stopping => {
					self.state = PlaybackState::Stopped;
					self.shared.set_state(PlaybackState::Stopped);
					self.renderer.all_notes_off();
				}
				_ => {}
			}
		}

		let ret = if let Some(frame) = self.frames.pop_front() {
			frame
		} else {
			self.render_moment();
			self.frames.pop_front().unwrap()
		};

		self.to_wait = self.to_wait.saturating_sub(Duration::from_secs_f64(dt));

		if self.to_wait.is_zero() {
			self.position += 1;
			self.to_wait = self.timer.sleep_duration(1);
		}

		ret
	}

	fn finished(&self) -> bool {
		self.sheet_finished() || self.state == PlaybackState::Stopped
	}

	fn on_start_processing(&mut self) {
		while let Ok(cmd) = self.receiver.try_recv() {
			match cmd {
				Command::Pause(tween) => {
					self.state = PlaybackState::Pausing;
					self.shared.set_state(PlaybackState::Pausing);
					self.volume_fade
						.set(Volume::Decibels(Volume::MIN_DECIBELS), tween);
				}
				Command::Resume(tween) => {
					self.state = PlaybackState::Playing;
					self.shared.set_state(PlaybackState::Playing);
					self.volume_fade.set(Volume::Decibels(0.0), tween);
				}
				Command::Stop(tween) => {
					self.state = PlaybackState::Stopping;
					self.shared.set_state(PlaybackState::Stopping);
					self.volume_fade
						.set(Volume::Decibels(Volume::MIN_DECIBELS), tween);
				}
				Command::SetVolume(volume, tween) => self.volume.set(volume, tween),
				Command::SetPanning(panning, tween) => self.panning.set(panning, tween),
			}
		}
	}
}

impl std::fmt::Debug for Sound {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Sound")
			.field("shared", &self.shared)
			.field("state", &self.state)
			.field("data", &self.data)
			.field("renderer", &self.renderer)
			.field("frames", &self.frames)
			.field("position", &self.position)
			.field("timer", &"unknown")
			.field("receiver", &self.receiver)
			.field("volume", &self.volume)
			.field("volume_fade", &self.volume_fade)
			.field("panning", &self.panning)
			.finish()
	}
}

// Internal: Shared ////////////////////////////////////////////////////////////

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

// Internal: Command ///////////////////////////////////////////////////////////

#[derive(Debug)]
pub(super) enum Command {
	SetVolume(Volume, Tween),
	SetPanning(f64, Tween),
	Pause(Tween),
	Resume(Tween),
	Stop(Tween),
}

// Internal: Renderer //////////////////////////////////////////////////////////

struct Renderer {
	synth: Synth,
	fbuf: Vec<Frame>,
	mode: RenderMode,
	sample_rate: u32,
}

impl Renderer {
	fn new(tick_len: Duration, soundfont: impl AsRef<Path>) -> Result<Self, Error> {
		use kira::manager::backend::cpal::Error as CpalError;

		let synth = Synth::new(
			fluidlite::Settings::new().expect("Failed to create a FluidSynth settings object."),
		)
		.map_err(Error::MidiSynth)?;

		let _ = synth
			.sfload(soundfont.as_ref(), true)
			.map_err(|err| Error::SoundFontRead(soundfont.as_ref().to_path_buf(), err))?;

		// FluidSynth's docs recommend setting the sample rate to the audio
		// driver's native output rate, so retrieve it via `cpal`.

		let host = cpal::default_host();
		let device = host
			.default_output_device()
			.ok_or(Error::KiraBackend(CpalError::NoDefaultOutputDevice))?;
		let dev_config = device
			.default_output_config()
			.map_err(|err| Error::KiraBackend(CpalError::DefaultStreamConfigError(err)))?;

		let sample_rate = dev_config.sample_rate().0;
		synth.set_sample_rate(sample_rate as f32);
		synth.set_gain(0.5);
		synth.set_reverb_on(false); // `kira` reverb effects can be applied later.
		synth.set_chorus_on(false);
		synth.system_reset().map_err(Error::MidiSynth)?;

		let channels = synth.count_audio_channels();

		// (RAT) Hack that makes *most* MIDIs sound *almost* correct.
		// Next step is to study prior art to properly allocate and use buffer.
		let bufsize = tick_len.as_micros() as usize / 25;
		let sample_format = dev_config.sample_format();

		let mode = match sample_format {
			SampleFormat::F32 => {
				let mut v = Vec::default();
				v.resize(bufsize, 0.0);

				match channels {
					1 => RenderMode::F32X1(v),
					2 => RenderMode::F32X2(v),
					_ => unreachable!(),
				}
			}
			SampleFormat::I16 | SampleFormat::U16 => {
				let mut v = Vec::default();
				v.resize(bufsize, 0);

				match channels {
					1 => RenderMode::I16X1(v),
					2 => RenderMode::I16X2(v),
					_ => unreachable!(),
				}
			}
			other => unimplemented!("Unexpected sample format: {other:#?}"),
		};

		let mut ret = Self {
			synth,
			mode,
			fbuf: Vec::with_capacity(bufsize / channels as usize),
			sample_rate,
		};

		ret.all_notes_off();

		Ok(ret)
	}

	fn render(&mut self, moment: &Moment) -> impl Iterator<Item = Frame> + '_ {
		for event in &moment.events {
			if let nodi::Event::Midi(midi) = event {
				let _ = self.play(*midi);
			}
		}

		self.generate_frames();
		self.fbuf.drain(..)
	}

	fn render_with_timer(
		&mut self,
		moment: &Moment,
		timer: &mut dyn Timer,
	) -> impl Iterator<Item = Frame> + '_ {
		for event in &moment.events {
			match event {
				nodi::Event::Midi(midi) => {
					let _ = self.play(*midi);
				}
				nodi::Event::Tempo(tempo) => {
					timer.change_tempo(*tempo);
				}
				_ => {}
			}
		}

		self.generate_frames();
		self.fbuf.drain(..)
	}

	fn generate_frames(&mut self) {
		match &mut self.mode {
			// (RAT) All I seem to have is sample data using 1 channel of floats,
			// so all the other cases are untested.
			RenderMode::F32X1(sbuf) | RenderMode::F32X2(sbuf) => {
				let _ = self.synth.write::<&mut [f32]>(sbuf.as_mut());

				for chunk in sbuf.chunks_exact_mut(2) {
					self.fbuf.push(Frame {
						left: chunk[0],
						right: chunk[1],
					});
				}
			}
			RenderMode::I16X1(sbuf) | RenderMode::I16X2(sbuf) => {
				let _ = self.synth.write::<&mut [i16]>(sbuf.as_mut());

				for chunk in sbuf.chunks_exact_mut(2) {
					self.fbuf.push(Frame {
						left: <i16 as cpal::Sample>::to_float_sample(chunk[0]),
						right: <i16 as cpal::Sample>::to_float_sample(chunk[1]),
					});
				}
			}
		}
	}
}

impl nodi::Connection for Renderer {
	fn play(&mut self, event: nodi::MidiEvent) -> bool {
		use nodi::midly::MidiMessage;
		let c = event.channel.as_int() as u32;

		let _ = match event.message {
			MidiMessage::NoteOff { key, vel: _ } => self.synth.note_off(c, key.as_int() as u32),
			MidiMessage::NoteOn { key, vel } => {
				self.synth
					.note_on(c, key.as_int() as u32, vel.as_int() as u32)
			}
			MidiMessage::Aftertouch { key, vel } => {
				self.synth
					.key_pressure(c, key.as_int() as u32, vel.as_int() as u32)
			}
			MidiMessage::Controller { controller, value } => {
				self.synth
					.cc(c, controller.as_int() as u32, value.as_int() as u32)
			}
			MidiMessage::ProgramChange { program } => {
				self.synth.program_change(c, program.as_int() as u32)
			}
			MidiMessage::ChannelAftertouch { vel } => {
				self.synth.channel_pressure(c, vel.as_int() as u32)
			}
			MidiMessage::PitchBend { bend } => self.synth.pitch_bend(c, bend.0.as_int() as u32),
		};

		true
	}
}

impl std::fmt::Debug for Renderer {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Renderer")
			.field("synth", &())
			.field("fbuf", &self.fbuf)
			.field("sample_rate", &self.sample_rate)
			.finish()
	}
}

#[derive(Debug)]
enum RenderMode {
	/// 32-bit floats in 1 channel.
	F32X1(Vec<f32>),
	/// 32-bit floats in 2 channels.
	F32X2(Vec<f32>),
	/// 16-bit signed or unsigned integers in 1 channel.
	I16X1(Vec<i16>),
	/// 16-bit signed or unsigned integers in 2 channels.
	I16X2(Vec<i16>),
}
