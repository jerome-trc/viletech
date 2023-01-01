//! Developer GUI for diagnosing and interacting with the audio subsystem.

use std::path::Path;

use indoc::formatdoc;
use kira::{
	sound::static_sound::{PlaybackState, StaticSoundSettings},
	tween::Tween,
};
use log::{info, warn};

use super::{AudioCore, MidiData, MidiSettings};

impl AudioCore {
	pub(super) fn ui_impl(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
		egui::ScrollArea::vertical().show(ui, |ui| {
			ui.heading("Music");

			ui.label(&formatdoc! {"
				Music 1: {:#?}
				Music 2: {:#?}",
				self.music1,
				self.music2,
			});

			ui.separator();
			ui.heading("Sounds");

			ui.label(&format!(
				"{} playing, {} paused, {} total",
				self.sounds
					.iter()
					.filter(|s| s.state() == PlaybackState::Playing)
					.count(),
				self.sounds
					.iter()
					.filter(|s| s.state() == PlaybackState::Paused)
					.count(),
				self.sounds.len(),
			));

			let mut i = 0;

			while i < self.sounds.len() {
				let state = self.sounds[i].state();

				ui.horizontal(|ui| {
					ui.label(format!("{} - {:#?}", i, state));

					ui.add_visible_ui(state != PlaybackState::Playing, |ui| {
						if ui.button("Resume").clicked() {
							if let Err(err) = self.sounds[i].resume(Tween::default()) {
								warn!("Failed to resume sound: {err}");
							}
						}
					});

					ui.add_visible_ui(state == PlaybackState::Playing, |ui| {
						if ui.button("Pause").clicked() {
							if let Err(err) = self.sounds[i].pause(Tween::default()) {
								warn!("Failed to pause sound: {err}");
							}
						}
					});

					if ui.button("Stop").clicked() {
						if let Err(err) = self.sounds[i].stop(Tween::default()) {
							warn!("Failed to stop sound: {err}");
						}

						self.sounds.swap_remove(i);
					} else {
						i += 1;
					}
				});
			}

			ui.separator();
			ui.heading("Quick Play");

			ui.horizontal(|ui| {
				ui.radio_value(&mut self.gui.slot_to_play, SELSLOT_MUS1, "Music 1");
				ui.radio_value(&mut self.gui.slot_to_play, SELSLOT_MUS2, "Music 2");
				ui.radio_value(&mut self.gui.slot_to_play, SELSLOT_SOUND, "Sound");
			});

			ui.add(
				egui::Slider::new(&mut self.gui.volume, 0.0..=4.0)
					.text("Volume")
					.custom_formatter(|val, _| format!("{:03.2}%", val * 100.0)),
			);

			ui.horizontal(|ui| {
				ui.label("MIDI Device: ");

				ui.menu_button(format!("{} | \u{23F7}", self.gui.midi_device), |ui| {
					ui.set_min_width(20.0);

					for device in zmusic::device::Index::all() {
						match *device {
							// Support for these two by zmusic-rs and VileTech are WIP
							zmusic::device::Index::Standard | zmusic::device::Index::Default => {
								continue;
							}
							_ => {}
						};

						if ui.button(format!("{}", device)).clicked() {
							self.gui.midi_device = *device;
							ui.close_menu();
						}
					}
				});
			});

			ui.label("(VFS Path/Asset ID/Asset Index)");

			ui.horizontal(|ui| {
				ui.text_edit_singleline(&mut self.gui.string_buf);

				let btn_play = egui::Button::new("Play");
				let btn_clear = egui::Button::new("Clear");

				if ui
					.add_enabled(!self.gui.string_buf.is_empty(), btn_play)
					.clicked()
				{
					self.ui_impl_try_play();
				}

				if ui
					.add_enabled(!self.gui.string_buf.is_empty(), btn_clear)
					.clicked()
				{
					self.gui.string_buf.clear();
				}
			});

			ui.separator();
			ui.heading("Diagnostics");

			ui.label(formatdoc! {"
			Sound capacity: {snd_cap}
			Mixer sub-track capacity: {track_cap}
			Clock capacity: {clock_cap}",
			snd_cap = self.manager.sound_capacity(),
			track_cap = self.manager.num_sub_tracks(),
			clock_cap = self.manager.clock_capacity(),
			});
		});
	}

	fn ui_impl_try_play(&mut self) {
		let path = Path::new(&self.gui.string_buf).to_path_buf();
		let vfs = self.vfs.read();

		let fref = match vfs.lookup(&path) {
			Some(f) => f,
			None => {
				info!("No file under virtual path: {}", path.display());
				return;
			}
		};

		if !fref.is_readable() {
			info!(
				"File can not be read (not binary or text): {}",
				path.display()
			);
			return;
		}

		let bytes = fref.read_unchecked();

		if zmusic::MidiKind::is_midi(bytes) {
			let midi = match self.zmusic.new_song(bytes, self.gui.midi_device) {
				Ok(m) => m,
				Err(err) => {
					info!(
						"Failed to create MIDI song from: {}\r\n\tError: {err}",
						path.display()
					);
					return;
				}
			};

			let mut midi = MidiData::new(midi, MidiSettings::default());
			midi.settings.volume = kira::Volume::Amplitude(self.gui.volume);
			drop(vfs);

			let res = match self.gui.slot_to_play {
				SELSLOT_MUS1 => self.start_music_midi::<false>(midi),
				SELSLOT_MUS2 => self.start_music_midi::<true>(midi),
				SELSLOT_SOUND => self.start_sound_midi(midi, None),
				_ => unreachable!(),
			};

			match res {
				Ok(()) => {
					info!(
						"Playing MIDI: {}\r\n\tWith device: {}\r\n\tAt volume: {}",
						path.display(),
						self.gui.midi_device,
						self.gui.volume
					);
				}
				Err(err) => {
					info!(
						"Failed to play MIDI from: {}\r\n\tError: {err}",
						path.display()
					);
				}
			};
		} else if let Ok(mut sdat) =
			super::sound_from_bytes(bytes.to_owned(), StaticSoundSettings::default())
		{
			sdat.settings.volume = kira::Volume::Amplitude(self.gui.volume);
			drop(vfs);

			let res = match self.gui.slot_to_play {
				SELSLOT_MUS1 => self.start_music_wave::<false>(sdat),
				SELSLOT_MUS2 => self.start_music_wave::<true>(sdat),
				SELSLOT_SOUND => self.start_sound_wave(sdat, None),
				_ => unreachable!(),
			};

			match res {
				Ok(()) => {
					info!(
						"Playing: {}\r\n\tAt volume: {}",
						path.display(),
						self.gui.volume
					);
				}
				Err(err) => {
					info!("Failed to play: {}\r\nError: {err}", path.display());
				}
			};
		} else {
			info!(
				"Given file is neither waveform nor MIDI audio: {}",
				path.display()
			);
		}
	}
}

/// State storage for the audio developer GUI.
pub(super) struct DeveloperGui {
	string_buf: String,
	volume: f64,
	midi_device: zmusic::device::Index,
	slot_to_play: u8,
}

const SELSLOT_MUS1: u8 = 0;
const SELSLOT_MUS2: u8 = 1;
const SELSLOT_SOUND: u8 = 2;

impl Default for DeveloperGui {
	fn default() -> Self {
		Self {
			string_buf: Default::default(),
			volume: 1.0,
			midi_device: zmusic::device::Index::FluidSynth,
			slot_to_play: SELSLOT_SOUND,
		}
	}
}
