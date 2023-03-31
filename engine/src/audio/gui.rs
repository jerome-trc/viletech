//! Developer GUI for diagnosing and interacting with the audio subsystem.

use std::path::PathBuf;

use bevy::prelude::{info, warn};
use bevy_egui::egui;
use indoc::formatdoc;
use kira::{
	sound::static_sound::{PlaybackState, StaticSoundSettings},
	tween::Tween,
};
use nodi::midly;

use crate::VPath;

use super::{midi, AudioCore, MidiData, MidiSettings};

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
					ui.label(format!("{i} - {state:#?}"));

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
				ui.radio_value(
					&mut self.gui.slot_to_play,
					DeveloperGui::SELSLOT_MUS1,
					"Music 1",
				);
				ui.radio_value(
					&mut self.gui.slot_to_play,
					DeveloperGui::SELSLOT_MUS2,
					"Music 2",
				);
				ui.radio_value(
					&mut self.gui.slot_to_play,
					DeveloperGui::SELSLOT_SOUND,
					"Sound",
				);
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

			ui.label("(VFS Path/Asset ID)");

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

		let fref = match catalog.get_file(&path) {
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

			let res = match self.gui.slot_to_play {
				DeveloperGui::SELSLOT_MUS1 => self.start_music_midi::<false>(mdat),
				DeveloperGui::SELSLOT_MUS2 => self.start_music_midi::<true>(mdat),
				DeveloperGui::SELSLOT_SOUND => self.start_sound_midi(mdat, None),
				_ => unreachable!(),
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

			let res = match self.gui.slot_to_play {
				DeveloperGui::SELSLOT_MUS1 => self.start_music_wave::<false>(sdat),
				DeveloperGui::SELSLOT_MUS2 => self.start_music_wave::<true>(sdat),
				DeveloperGui::SELSLOT_SOUND => self.start_sound_wave(sdat, None),
				_ => unreachable!(),
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
pub(super) struct DeveloperGui {
	/// Let the user write a VFS path or asset ID.
	pub(super) id_buf: String,
	/// For allowing the user to enter a custom SoundFont path.
	///
	/// Set by default from the first SoundFont collected during audio core
	/// initialization. If none were found, this will be empty.
	pub(super) soundfont_buf: String,
	/// Amplitude. Slider runs from 0.0 to 4.0.
	pub(super) volume: f64,
	pub(super) midi_device: midi::Device,
	pub(super) slot_to_play: u8,
}

impl DeveloperGui {
	const SELSLOT_MUS1: u8 = 0;
	const SELSLOT_MUS2: u8 = 1;
	const SELSLOT_SOUND: u8 = 2;
}

impl Default for DeveloperGui {
	fn default() -> Self {
		Self {
			id_buf: String::default(),
			soundfont_buf: String::default(),
			volume: 1.0,
			midi_device: midi::Device::FluidSynth,
			slot_to_play: DeveloperGui::SELSLOT_SOUND,
		}
	}
}
