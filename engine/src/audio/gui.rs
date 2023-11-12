use std::path::PathBuf;

use bevy_egui::egui;
use indoc::formatdoc;
use kira::{
	sound::{static_sound::StaticSoundSettings, PlaybackState},
	tween::{self, Tween},
	Volume,
};
use nodi::midly::Smf;
use tracing::{error, info};
use vfs::{VPath, VirtualFs};

use super::{AudioCore, MidiData, MidiSettings, SoundSpace};

impl AudioCore {
	pub(super) fn ui_impl(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, vfs: &VirtualFs) {
		egui::ScrollArea::vertical().show(ui, |ui| {
			// Music ///////////////////////////////////////////////////////////

			ui.heading("Music");

			for (i, group) in self.music.iter_mut().enumerate() {
				ui.group(|ui| {
					for slot in &mut group.layers {
						let Some(layer) = slot else {
							ui.label(format!("Layer {i} - <none>"));
							continue;
						};

						let state = layer.state();
						ui.label(format!("Layer {i} - {state:#?}"));
					}
				});
			}

			ui.separator();

			// Sounds //////////////////////////////////////////////////////////

			ui.heading("Sounds");

			ui.label(&self.ui_label_song_counters());

			for (i, channel) in self.sounds.iter_mut().enumerate() {
				let Some(sfx) = channel else {
					ui.label(format!("Channel {i} - <none>"));
					continue;
				};

				let state = sfx.state();
				ui.label(format!("Channel {i} - {state:#?}"));

				ui.add_visible_ui(state != PlaybackState::Playing, |ui| {
					if ui.button("Resume").clicked() {
						if let Err(err) = sfx.resume(Tween::default()) {
							error!("Failed to resume sound in channel {i}: {err}");
						}
					}
				});

				ui.add_visible_ui(state == PlaybackState::Playing, |ui| {
					if ui.button("Pause").clicked() {
						if let Err(err) = sfx.pause(Tween::default()) {
							error!("Failed to pause sound in channel {i}: {err}");
						}
					}
				});

				if ui.button("Stop").clicked() {
					if let Err(err) = sfx.stop(Tween::default()) {
						error!("Failed to stop sound in channel {i}: {err}");
					}

					*channel = None;
				}
			}

			ui.separator();

			// Quick play //////////////////////////////////////////////////////

			ui.heading("Quick Play");

			ui.add(
				egui::Slider::new(&mut self.gui.volume, 0.0..=4.0)
					.text("Volume")
					.custom_formatter(|val, _| format!("{:03.2}%", val * 100.0)),
			);

			ui.horizontal(|ui| {
				ui.label("MIDI SoundFont File: ");
				ui.text_edit_singleline(&mut self.gui.soundfont_buf);
			});

			ui.label("(VFS Path/Data ID)");

			ui.horizontal(|ui| {
				ui.text_edit_singleline(&mut self.gui.id_buf);

				let btn_clear = egui::Button::new("\u{1F5D9}");
				let btn_play = egui::Button::new("Play");

				if ui
					.add_enabled(!self.gui.id_buf.is_empty(), btn_play)
					.clicked()
				{
					self.ui_impl_try_play(vfs);
				}

				if ui
					.add_enabled(!self.gui.id_buf.is_empty(), btn_clear)
					.clicked()
				{
					self.gui.id_buf.clear();
				}
			});

			ui.separator();

			// Diagnostics /////////////////////////////////////////////////////

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

	fn ui_impl_try_play(&mut self, vfs: &VirtualFs) {
		let path = VPath::new(&self.gui.id_buf).to_owned();

		let fref = match vfs.lookup(&path) {
			Some(vfs::Ref::File(r)) => r,
			Some(vfs::Ref::Folder(_)) => {
				info!("`{path}` is a folder and can not be played.");
				return;
			}
			None => {
				info!("No file under virtual path: {path}");
				return;
			}
		};

		let mut guard = fref.lock();

		let bytes = match guard.read() {
			Ok(b) => b,
			Err(err) => {
				error!("Failed to read file: {err}");
				return;
			}
		};

		if let Ok(midi) = Smf::parse(&bytes) {
			let sf_path = PathBuf::from(self.gui.soundfont_buf.clone());

			if !sf_path.exists() {
				info!("The requested SoundFont was not found.");
				return;
			}

			let mut mdat = MidiData::new(midi, sf_path.clone(), MidiSettings::default());
			mdat.settings.volume = Volume::Amplitude(self.gui.volume);

			let res = self.start_sfx_midi(mdat, None, SoundSpace::Unsourced);

			match res {
				Ok(()) => {
					info!("Playing: {path} (volume {vol})", vol = self.gui.volume,);
				}
				Err(err) => {
					info!("Failed to play MIDI `{}` - {err}", sf_path.display());
				}
			}
		} else if let Ok(mut sdat) =
			super::sound_from_bytes(bytes.to_owned(), StaticSoundSettings::default())
		{
			sdat.settings.volume = tween::Value::Fixed(Volume::Amplitude(self.gui.volume));

			let res = self.start_sfx_wave(sdat, None, SoundSpace::Unsourced);

			match res {
				Ok(()) => {
					info!("Playing: {path} (volume {vol})", vol = self.gui.volume);
				}
				Err(err) => {
					info!("Failed to play `{path}` - {err}");
				}
			};
		} else {
			info!("Given file is neither waveform nor MIDI audio: `{path}`",);
		}
	}

	#[must_use]
	fn ui_label_song_counters(&self) -> String {
		let mut playing = 0;
		let mut paused = 0;

		for channel in self.sounds.iter() {
			let Some(sfx) = channel else {
				continue;
			};

			if sfx.state() == PlaybackState::Playing {
				playing += 1;
			}

			if sfx.state() == PlaybackState::Paused {
				paused += 1;
			}
		}

		format!(
			"{playing} playing, {paused} paused, {changing} changing, {total} total",
			changing = self.sounds.len() - (playing + paused),
			total = self.sounds.len(),
		)
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
}

impl Default for DevGui {
	fn default() -> Self {
		Self {
			id_buf: String::new(),
			soundfont_buf: String::new(),
			volume: 1.0,
		}
	}
}
