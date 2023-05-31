//! Developer GUI for diagnosing and interacting with the audio subsystem.

use std::path::PathBuf;

use bevy::prelude::{error, info};
use bevy_egui::egui;
use indoc::formatdoc;
use kira::{
	sound::static_sound::{PlaybackState, StaticSoundSettings},
	tween::Tween,
};
use nodi::midly;
use vfs::VPath;

use super::{midi, AudioCore, MidiData, MidiSettings};

impl AudioCore {
	pub(super) fn ui_impl(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
		egui::ScrollArea::vertical().show(ui, |ui| {
			ui.heading("Music");

			for (i, mus) in self.music.iter().enumerate() {
				let state = mus.state();
				ui.label(format!("{i} - {state:#?}"));
			}

			ui.separator();
			ui.heading("Sounds");

			ui.label(&{
				let mut playing = 0;
				let mut paused = 0;

				for snd in &self.sounds {
					if snd.state() == PlaybackState::Playing {
						playing += 1;
					}

					if snd.state() == PlaybackState::Paused {
						paused += 1;
					}
				}

				format!(
					"{playing} playing, {paused} paused, {changing} changing, {total} total",
					changing = self.sounds.len() - (playing + paused),
					total = self.sounds.len(),
				)
			});

			let mut i = 0;

			while i < self.sounds.len() {
				let state = self.sounds[i].state();

				ui.horizontal(|ui| {
					ui.label(format!("{i} - {state:#?}"));

					ui.add_visible_ui(state != PlaybackState::Playing, |ui| {
						if ui.button("Resume").clicked() {
							if let Err(err) = self.sounds[i].resume(Tween::default()) {
								error!("Failed to resume sound: {err}");
							}
						}
					});

					ui.add_visible_ui(state == PlaybackState::Playing, |ui| {
						if ui.button("Pause").clicked() {
							if let Err(err) = self.sounds[i].pause(Tween::default()) {
								error!("Failed to pause sound: {err}");
							}
						}
					});

					if ui.button("Stop").clicked() {
						if let Err(err) = self.sounds[i].stop(Tween::default()) {
							error!("Failed to stop sound: {err}");
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
				ui.radio_value(&mut self.gui.play_music, true, "Music");
				ui.radio_value(&mut self.gui.play_music, false, "Sound");
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

					if ui.button(format!("{}", midi::Device::FluidSynth)).clicked() {
						self.gui.midi_device = midi::Device::FluidSynth;
						ui.close_menu();
					}
				});
			});

			ui.horizontal(|ui| {
				ui.label("MIDI SoundFont File: ");
				ui.text_edit_singleline(&mut self.gui.soundfont_buf);
			});

			ui.label("(VFS Path/Data ID)");

			ui.horizontal(|ui| {
				ui.text_edit_singleline(&mut self.gui.id_buf);

				let btn_play = egui::Button::new("Play");
				let btn_clear = egui::Button::new("Clear");

				if ui
					.add_enabled(!self.gui.id_buf.is_empty(), btn_play)
					.clicked()
				{
					self.ui_impl_try_play();
				}

				if ui
					.add_enabled(!self.gui.id_buf.is_empty(), btn_clear)
					.clicked()
				{
					self.gui.id_buf.clear();
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
		let path = VPath::new(&self.gui.id_buf).to_path_buf();
		let catalog = self.catalog.read();

		let fref = match catalog.vfs().get(&path) {
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

		let bytes = fref.read_bytes();

		if let Ok(midi) = midly::Smf::parse(bytes) {
			let sf_path = PathBuf::from(self.gui.soundfont_buf.clone());

			if !sf_path.exists() {
				info!("The requested SoundFont was not found on the disk.");
				return;
			}

			let mut mdat = MidiData::new(midi, sf_path.clone(), MidiSettings::default());

			drop(catalog);

			mdat.settings.volume = kira::Volume::Amplitude(self.gui.volume);

			let res = if self.gui.play_music {
				self.start_music_midi(mdat, None)
			} else {
				self.start_sound_midi(mdat, None)
			};

			match res {
				Ok(()) => {
					info!(
						"Playing: {p}\r\n\tAt volume: {vol}\r\n\tWith device: {dev}",
						p = path.display(),
						vol = self.gui.volume,
						dev = self.gui.midi_device
					);
				}
				Err(err) => {
					info!(
						"Failed to play MIDI from: {}\r\n\tError: {err}",
						sf_path.display()
					);
				}
			}
		} else if let Ok(mut sdat) =
			super::sound_from_bytes(bytes.to_owned(), StaticSoundSettings::default())
		{
			drop(catalog);

			sdat.settings.volume = kira::Volume::Amplitude(self.gui.volume);

			let res = if self.gui.play_music {
				self.start_music_wave(sdat, None)
			} else {
				self.start_sound_wave(sdat, None)
			};

			match res {
				Ok(()) => {
					info!(
						"Playing: {p}\r\n\tAt volume: {vol}",
						p = path.display(),
						vol = self.gui.volume
					);
				}
				Err(err) => {
					info!("Failed to play: {}\r\n\tError: {err}", path.display());
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
#[derive(Debug)]
pub(super) struct DevGui {
	/// Let the user write a VFS path or data object ID.
	pub(super) id_buf: String,
	/// For allowing the user to enter a custom SoundFont path.
	///
	/// Set by default from the first SoundFont collected during audio core
	/// initialization. If none were found, this will be empty.
	pub(super) soundfont_buf: String,
	/// Amplitude. Slider runs from 0.0 to 4.0.
	pub(super) volume: f64,
	pub(super) midi_device: midi::Device,
	pub(super) play_music: bool,
}

impl Default for DevGui {
	fn default() -> Self {
		Self {
			id_buf: String::default(),
			soundfont_buf: String::default(),
			volume: 1.0,
			midi_device: midi::Device::FluidSynth,
			play_music: false,
		}
	}
}
