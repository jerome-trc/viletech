//! Interface between [`kira`] and [`nodi`]/[`fluidlite`].
//!
//! Much of this code is a copy-paste of `kira`'s internal sound sampling code.

use std::{
	collections::VecDeque,
	io::{Read, Seek},
	path::{Path, PathBuf},
	sync::Arc,
	time::Duration,
};

use cpal::{
	traits::{DeviceTrait, HostTrait},
	SampleFormat,
};
use crossbeam::{
	atomic::AtomicCell,
	channel::{Receiver, Sender},
};
use fluidlite::Synth;
use kira::{
	clock::clock_info::ClockInfoProvider,
	dsp::Frame,
	modulator::value_provider::ModulatorValueProvider,
	sound::{
		static_sound::{StaticSoundData, StaticSoundSettings},
		PlaybackState, SoundData,
	},
	tween::{self, Parameter, Tween},
	OutputDestination, Volume,
};
use nodi::{
	midly::{Format, MetaMessage, Smf, Timing, TrackEvent, TrackEventKind},
	timers::{FixedTempo, Ticker},
	Connection, Moment, Sheet, Timer,
};
use tracing::{info, warn};

use super::{AudioCore, Error};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileFormat {
	Midi,
	Hmi,
	Xmi,
	DmxMus,
	Mids,
}

impl FileFormat {
	/// From ZMusic.
	#[must_use]
	pub fn deduce(bytes: &[u8]) -> Option<FileFormat> {
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
	pub(super) path: PathBuf,
	pub(super) kind: SoundFontKind,
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

/// Converts a MIDI into a waveform sound.
///
/// Note that the output is configured using `data.settings`, not `settings`.
pub fn render(data: Data, settings: StaticSoundSettings) -> Result<StaticSoundData, Error> {
	let mut timer = data.create_timer();
	let _ = timer.duration(&data.sheet); // Needed to initalize `timer`!
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
		frames: frames.into(),
		settings,
	})
}

/// Intended to parallel the interface of [`StaticSoundHandle`].
///
/// [`StaticSoundHandle`]: kira::sound::static_sound::StaticSoundHandle
#[derive(Debug)]
pub struct Handle {
	shared: Arc<AtomicCell<PlaybackState>>,
	sender: Sender<Command>,
}

impl Handle {
	/// Returns the current playback state of the sound.
	#[must_use]
	pub fn state(&self) -> PlaybackState {
		self.shared.load()
	}

	#[must_use]
	pub fn is_playing(&self) -> bool {
		self.state() == PlaybackState::Playing
	}

	pub fn pause(&self, tween: Tween) -> Result<(), Error> {
		self.sender.send(Command::Pause(tween)).map_err(Error::from)
	}

	pub fn resume(&self, tween: Tween) -> Result<(), Error> {
		self.sender
			.send(Command::Resume(tween))
			.map_err(Error::from)
	}

	pub fn stop(&self, tween: Tween) -> Result<(), Error> {
		self.sender.send(Command::Stop(tween)).map_err(Error::from)
	}

	pub fn set_volume(&self, volume: impl Into<Volume>, tween: Tween) -> Result<(), Error> {
		self.sender
			.send(Command::SetVolume(volume.into(), tween))
			.map_err(Error::from)
	}

	pub fn set_panning(&self, panning: f64, tween: Tween) -> Result<(), Error> {
		self.sender
			.send(Command::SetPanning(panning, tween))
			.map_err(Error::from)
	}
}

/// Intended to parallel the interface of [`StaticSoundData`].
///
/// [`StaticSoundData`]: kira::sound::static_sound::StaticSoundData
#[derive(Debug, Clone)]
pub struct Data {
	pub settings: Settings,
	soundfont: PathBuf,
	sheet: Arc<Sheet>,
	timing: Timing,
}

impl Data {
	#[must_use]
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

/// Intended to parallel the interface of [`StaticSoundSettings`].
///
/// [`StaticSoundSettings`]: kira::sound::static_sound::StaticSoundSettings
#[derive(Debug, Clone)]
pub struct Settings {
	pub destination: OutputDestination,
	pub volume: Volume,
	/// The panning of the sound, where 0 is hard left and 1 is hard right.
	pub panning: f64,
	pub fade_in: Option<Tween>,
}

impl Default for Settings {
	fn default() -> Self {
		Self {
			destination: OutputDestination::MAIN_TRACK,
			volume: Volume::Amplitude(1.0),
			panning: 0.5,
			fade_in: None,
		}
	}
}

impl SoundData for Data {
	type Error = Box<Error>; // Indirection prevents a recursive type.
	type Handle = Handle;

	fn into_sound(self) -> Result<(Box<dyn kira::sound::Sound>, Self::Handle), Self::Error> {
		// `sleep_duration` demands `mut` but it never does any real mutation.
		let mut timer = self.create_timer();

		let _ = timer.duration(&self.sheet); // Needed to initalize `timer`!
		let tick_len = timer.sleep_duration(1);

		let renderer = Renderer::new(tick_len, &self.soundfont)?;
		let (sender, receiver) = crossbeam::channel::unbounded();

		let shared = Arc::new(AtomicCell::new(PlaybackState::Playing));

		let shared_kept = shared.clone();

		let volume = Parameter::new(
			tween::Value::Fixed(self.settings.volume),
			self.settings.volume,
		);

		let volume_fade = match self.settings.fade_in {
			Some(fade_in) => {
				let mut tweenable = Parameter::new(
					tween::Value::Fixed(Volume::Decibels(Volume::MIN_DECIBELS)),
					Volume::Decibels(Volume::MIN_DECIBELS),
				);

				tweenable.set(tween::Value::Fixed(Volume::Decibels(0.0)), fade_in);

				tweenable
			}
			None => Parameter::new(
				tween::Value::Fixed(Volume::Decibels(0.0)),
				Volume::Decibels(0.0),
			),
		};

		let panning = Parameter::new(
			tween::Value::Fixed(self.settings.panning),
			self.settings.panning,
		);

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

// Kira sound //////////////////////////////////////////////////////////////////

struct Sound {
	shared: Arc<AtomicCell<PlaybackState>>,
	state: PlaybackState,
	data: Data,
	renderer: Renderer,
	frames: VecDeque<Frame>,
	/// Which [`Moment`] in `data`'s sheet are we at?
	position: usize,
	timer: Box<dyn Timer + Send + Sync>,
	to_wait: Duration,
	receiver: Receiver<Command>,

	volume: Parameter<Volume>,
	volume_fade: Parameter<Volume>,
	panning: Parameter,
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
	fn output_destination(&mut self) -> OutputDestination {
		self.data.settings.destination
	}

	fn process(
		&mut self,
		dt: f64,
		clock: &ClockInfoProvider,
		modulator: &ModulatorValueProvider,
	) -> Frame {
		self.volume.update(dt, clock, modulator);
		self.panning.update(dt, clock, modulator);

		if self.volume_fade.update(dt, clock, modulator) {
			match self.state {
				PlaybackState::Pausing => {
					self.state = PlaybackState::Paused;
					self.shared.store(PlaybackState::Paused);
					self.renderer.all_notes_off();
				}
				PlaybackState::Stopping => {
					self.state = PlaybackState::Stopped;
					self.shared.store(PlaybackState::Stopped);
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
			if (self.position + 1) == self.data.sheet.len() {
				self.state = PlaybackState::Stopped;
				self.shared.store(PlaybackState::Stopped);
			} else {
				self.position += 1;
			}

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
					self.shared.store(PlaybackState::Pausing);
					self.volume_fade.set(
						tween::Value::Fixed(Volume::Decibels(Volume::MIN_DECIBELS)),
						tween,
					);
				}
				Command::Resume(tween) => {
					self.state = PlaybackState::Playing;
					self.shared.store(PlaybackState::Playing);
					self.volume_fade
						.set(tween::Value::Fixed(Volume::Decibels(0.0)), tween);
				}
				Command::Stop(tween) => {
					self.state = PlaybackState::Stopping;
					self.shared.store(PlaybackState::Stopping);
					self.volume_fade.set(
						tween::Value::Fixed(Volume::Decibels(Volume::MIN_DECIBELS)),
						tween,
					);
				}
				Command::SetVolume(volume, tween) => {
					self.volume.set(tween::Value::Fixed(volume), tween)
				}
				Command::SetPanning(panning, tween) => {
					self.panning.set(tween::Value::Fixed(panning), tween)
				}
			}
		}
	}
}

// AudioCore's SoundFont code //////////////////////////////////////////////////

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

// Renderer ////////////////////////////////////////////////////////////////////

struct Renderer {
	synth: Synth,
	fbuf: Vec<Frame>,
	mode: RenderMode,
	sample_rate: u32,
}

impl Renderer {
	/// `tick_len` needs to come from [`nodi::Timer::sleep_duration`],
	/// which should be passed `1`.
	fn new(tick_len: Duration, soundfont: impl AsRef<Path>) -> Result<Self, Error> {
		use kira::manager::backend::cpal::Error as CpalError;

		let synth = Synth::new(
			fluidlite::Settings::new().expect("failed to create a FluidSynth settings object"),
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

		// (RAT) Magic hack here. Don't know why this works, but all the MIDIs I
		// try sound indistinguishable from VLC and GZDoom's FluidSynth, so I don't
		// regard it as worth the effort to investigate more deeply right now.
		// To the reader: if you understand this, please explain to me and help me fix it.
		let sample_format = dev_config.sample_format();
		let bufsize = (tick_len.mul_f64(1.59369747899).as_micros() as usize) / 18;

		let mode = match sample_format {
			SampleFormat::F32 => {
				let mut v = vec![];
				v.resize(bufsize, 0.0);

				match channels {
					1 => RenderMode::F32X1(v),
					2 => RenderMode::F32X2(v),
					_ => unreachable!(),
				}
			}
			SampleFormat::I16 | SampleFormat::U16 => {
				let mut v = vec![];
				v.resize(bufsize, 0);

				match channels {
					1 => RenderMode::I16X1(v),
					2 => RenderMode::I16X2(v),
					_ => unreachable!(),
				}
			}
			other => unimplemented!("unexpected sample format: {other:#?}"),
		};

		let mut ret = Self {
			synth,
			mode,
			fbuf: Vec::with_capacity((sample_rate / 5) as usize),
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
				let _ = self.synth.write::<&mut [f32]>(&mut sbuf[..]);

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

// MIDI control commands ///////////////////////////////////////////////////////

#[derive(Debug)]
pub(super) enum Command {
	SetVolume(Volume, Tween),
	SetPanning(f64, Tween),
	Pause(Tween),
	Resume(Tween),
	Stop(Tween),
}
